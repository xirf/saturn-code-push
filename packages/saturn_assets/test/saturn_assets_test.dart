import 'dart:convert';
import 'dart:io';

import 'package:archive/archive.dart';
import 'package:crypto/crypto.dart';
import 'package:cryptography/cryptography.dart' as signing;
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:saturn_assets/saturn_assets.dart';

class BaseBundle extends CachingAssetBundle {
  @override
  Future<ByteData> load(String key) async => ByteData.sublistView(utf8.encode('base:$key'));
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('downloads an overlay, reads manifest, and requires signatures and fails closed on offline or tampered metadata', () async {
    final manifest = const StandardMessageCodec().encodeMessage({
      'assets/new.txt': [
        {'asset': 'assets/new.txt', 'dpr': null},
      ],
    })!;
    final archive = Archive()
      ..addFile(ArchiveFile.bytes('AssetManifest.bin', Uint8List.sublistView(manifest)))
      ..addFile(ArchiveFile.string('assets/new.txt', 'patched'));
    final zip = ZipEncoder().encodeBytes(archive);
    final hash = sha256.convert(zip).toString();
    final algorithm = signing.Ed25519();
    final keyPair = await algorithm.newKeyPair();
    final publicKey = await keyPair.extractPublicKey();
    final manifestJson = jsonEncode(['saturn-assets-v1', 'app', '1.0.0+1', 1, 'android', hash, zip.length]);
    final signature = await algorithm.sign(utf8.encode(manifestJson), keyPair: keyPair);
    var downloads = 0;
    var offline = false;
    var tampered = false;
    Future<Uint8List> fetch(Uri uri, int maxBytes) async {
      if (offline) throw const SocketException('offline');
      if (uri.path == '/api/v1/assets/check') {
        return Uint8List.fromList(utf8.encode(jsonEncode({
          'asset_bundle_available': true,
          'manifest': manifestJson,
          'signature': base64Encode(tampered ? List<int>.filled(64, 0) : signature.bytes),
          'sha256': hash,
          'size': zip.length,
          'download_url': 'https://saturn.example/download',
        })));
      }
      downloads++;
      return zip;
    }
    final temp = await Directory.systemTemp.createTemp('saturn-asset-test');
    try {
      Future<SaturnAssets> load() => SaturnAssets.load(
        server: Uri.parse('https://saturn.example'),
        appId: 'app', releaseVersion: '1.0.0+1', channel: 'stable',
        platform: 'android', patchNumber: 1, base: BaseBundle(),
        trustedPublicKey: Uint8List.fromList(publicKey.bytes),
        cacheDirectory: temp,
        fetch: fetch,
      );
      final first = await load();
      expect(first.source, 'downloaded');
      expect(await first.bundle.loadString('assets/new.txt'), 'patched');
      expect(await first.bundle.loadString('assets/old.txt'), 'base:assets/old.txt');
      final parsed = await AssetManifest.loadFromAssetBundle(first.bundle);
      expect(parsed.listAssets(), contains('assets/new.txt'));
      final second = await load();
      expect(second.source, 'downloaded');
      expect(downloads, 2);
      offline = true;
      expect((await load()).source, 'base');
      offline = false;
      tampered = true;
      expect((await load()).source, 'base');
    } finally {
      await temp.delete(recursive: true);
    }
  });
}
