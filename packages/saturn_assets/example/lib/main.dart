import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:saturn_assets/saturn_assets.dart';
import 'package:shorebird_code_push/shorebird_code_push.dart';

/// The 32-byte Ed25519 public key used to verify signed asset manifests.
///
/// Compile your own key into the baseline release. Never fetch it from the
/// server. This all-zero placeholder rejects every manifest, so the app always
/// falls back to its packaged assets until you replace it.
final pinnedAssetPublicKey = Uint8List(32);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Select assets for the patch that is running now, not a downloaded patch
  // that will only run after the next restart.
  final updater = ShorebirdUpdater();
  final currentPatch = updater.isAvailable
      ? await updater.readCurrentPatch()
      : null;

  final assets = await SaturnAssets.load(
    server: Uri.parse('https://updates.example.com'),
    appId: 'YOUR_SATURN_APP_ID',
    releaseVersion: '1.0.0+1',
    channel: 'stable',
    platform: 'android',
    patchNumber: currentPatch?.number ?? 0,
    trustedPublicKey: pinnedAssetPublicKey,
  );

  runApp(ExampleApp(assets: assets));
}

/// Installs the resolved bundle above every route.
class ExampleApp extends StatelessWidget {
  /// Creates the example app for the resolved [assets].
  const ExampleApp({super.key, required this.assets});

  /// The asset overlay resolved at startup.
  final SaturnAssets assets;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'saturn_assets example',
      builder: (context, child) =>
          DefaultAssetBundle(bundle: assets.bundle, child: child!),
      home: Scaffold(
        appBar: AppBar(title: const Text('saturn_assets example')),
        body: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Patch ${assets.patchNumber}, assets: ${assets.source}'),
              const SizedBox(height: 16),
              const _Message(),
            ],
          ),
        ),
      ),
    );
  }
}

class _Message extends StatelessWidget {
  const _Message();

  @override
  Widget build(BuildContext context) {
    // Reads through the inherited bundle, so an overlaid file wins over the
    // packaged one. Calling rootBundle directly would bypass the overlay.
    return FutureBuilder<String>(
      future: DefaultAssetBundle.of(context).loadString('assets/message.txt'),
      builder: (context, snapshot) => Text(snapshot.data ?? 'Loading…'),
    );
  }
}
