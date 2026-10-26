import 'package:flutter/material.dart';
import 'package:saturn_assets/saturn_assets.dart';
import 'package:shorebird_code_push/shorebird_code_push.dart';

const marker = String.fromEnvironment('PROBE_MARKER', defaultValue: 'baseline');

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final updater = ShorebirdUpdater();
  var patch = (await updater.readCurrentPatch())?.number ?? 0;
  if (updater.isAvailable) {
    final status = await updater.checkForUpdate();
    if (status == UpdateStatus.outdated) {
      await updater.update();
      final next = (await updater.readNextPatch())?.number ?? 0;
      // ignore: avoid_print
      print('SATURN_ASSET_PROBE downloaded=$next restart_required=true');
    }
    patch = (await updater.readCurrentPatch())?.number ?? 0;
  }
  final assets = await SaturnAssets.load(
    server: Uri.parse('https://saturn.andka.id'),
    appId: 'cf60092f-662a-4c94-957d-e96f85fc7894',
    releaseVersion: '2.1.0+3',
    channel: 'stable',
    platform: 'android',
    patchNumber: patch,
  );
  if (patch >= 3) {
    await assets.loadFont(
      family: 'SaturnPatchText',
      asset: 'assets/RobotoSlab.ttf',
    );
    await assets.loadFont(
      family: 'SaturnPatchIcons',
      asset: 'assets/MaterialIcons.ttf',
    );
    // ignore: avoid_print
    print('SATURN_ASSET_PROBE fonts_loaded=true icon_font_loaded=true');
  }
  final message = await assets.bundle.loadString('assets/message.txt');
  // ignore: avoid_print
  print('SATURN_ASSET_PROBE marker=$marker patch=$patch source=${assets.source} asset=$message');
  runApp(AssetProbeApp(assets: assets, marker: marker));
}

class AssetProbeApp extends StatelessWidget {
  const AssetProbeApp({required this.assets, required this.marker, super.key});

  final SaturnAssets assets;
  final String marker;

  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: ThemeData.dark(useMaterial3: true),
    builder: (context, child) => DefaultAssetBundle(
      bundle: assets.bundle,
      child: child!,
    ),
    home: Scaffold(
      appBar: AppBar(title: const Text('Saturn asset probe')),
      body: Center(
        child: Builder(builder: (context) => FutureBuilder<String>(
          future: DefaultAssetBundle.of(context).loadString('assets/message.txt'),
          builder: (context, snapshot) => Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(marker, style: const TextStyle(fontSize: 24)),
              const SizedBox(height: 20),
              Text(snapshot.data ?? snapshot.error?.toString() ?? 'Loading asset',
                style: TextStyle(
                  fontSize: 32,
                  fontFamily: assets.patchNumber >= 3 ? 'SaturnPatchText' : null,
                )),
              if (assets.patchNumber >= 3) ...[
                const SizedBox(height: 12),
                Icon(
                  const IconData(
                    0xe159,
                    fontFamily: 'SaturnPatchIcons',
                  ),
                  size: 58,
                  color: Colors.lightBlueAccent,
                ),
                const Text(
                  'Downloaded font + icon font',
                  style: TextStyle(fontFamily: 'SaturnPatchText'),
                ),
              ],
              const SizedBox(height: 20),
              Image.asset(
                'assets/patched.png',
                width: 220,
                height: 180,
                fit: BoxFit.contain,
                errorBuilder: (context, error, stack) =>
                    Text('Image unavailable: $error'),
              ),
              const SizedBox(height: 20),
              Text('Patch ${assets.patchNumber} • ${assets.source}'),
            ],
          ),
        )),
      ),
    ),
  );
}
