# Saturn assets: Flutter integration

`saturn_assets` overlays Flutter assets for the Shorebird patch currently running on an Android device. It supplies a patched `AssetManifest.bin` and changed/new files through `DefaultAssetBundle`. Shorebird still handles the Dart patch; this package handles the separate asset ZIP served by Saturn.

For the complete build, upload, inspect, and promotion sequence, use the [asset update guide](../../docs/asset-support.md). For the differences from hosted Shorebird, start with [Using Saturn with Shorebird](../../docs/usage.md).

## Installation

Add the package **before building the baseline store release**. It uses `path_provider`, so adding it to a Dart patch is not sufficient for releases missing the native plugin registration.

For an application beside this Saturn checkout, merge these dependencies into its `pubspec.yaml` and adjust the relative path as needed:

```yaml
dependencies:
  flutter:
    sdk: flutter
  shorebird_code_push: ^2.0.7
  saturn_assets:
    path: ../saturn-code-push/packages/saturn_assets
```

For an application that does not have this checkout available, the same package can be fetched from the [Shorebird fork PR](https://github.com/xirf/shorebird/pull/1). Replace the path dependency with a Git dependency pinned to the reviewed commit:

```yaml
saturn_assets:
  git:
    url: https://github.com/xirf/shorebird.git
    ref: 1625363cc53c4091f88d5a3daaa8de9b12fb5c6d
    path: packages/saturn_assets
```

Use one dependency source, not both. The PR adds a package; it does not change CLI patch behavior. No forked CLI is required for the overlay itself. Upgrading this dependency does not automatically update already installed store releases.

Fetch dependencies with a Flutter SDK compatible with the application’s SDK constraints:

```bash
flutter pub get
```

Also include `shorebird.yaml` and your application assets in the application's Flutter asset declaration:

```yaml
flutter:
  assets:
    - shorebird.yaml
    - assets/
```

The logical asset key used at runtime must match the key in the compiled build, for example `assets/banner.png`. Do not add the ZIP's Android packaging prefix such as `base/assets/flutter_assets/` to that key.

## Load the running patch's assets

Initialize Flutter bindings, read the **current** patch, and load its bundle before building the application. Adapt this startup example to your existing home widget:

```dart
import 'package:flutter/material.dart';
import 'package:saturn_assets/saturn_assets.dart';
import 'package:shorebird_code_push/shorebird_code_push.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

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
    trustedPublicKey: pinnedAssetPublicKey, // 32 Ed25519 bytes compiled into the baseline
  );

  runApp(
    MaterialApp(
      builder: (context, child) {
        return DefaultAssetBundle(
          bundle: assets.bundle,
          child: child!,
        );
      },
      home: const Scaffold(
        body: Center(
          child: Text('Saturn asset integration is ready'),
        ),
      ),
    ),
  );
}
```

Use the same server, app ID, exact store release version, and channel as the running app's code updater. Replace the example values; `1.0.0+1` is the baseline version, not a new version for every patch. `readNextPatch()` refers to downloaded code for a future launch and must not select assets for code that is still running from an older patch.

The example only loads assets. Keep your normal code-update check/download flow. A background download becomes the running patch after restart; run this loader again on that startup. Network failure selects packaged assets, so code depending on a newly introduced file must provide an appropriate error/fallback UI.

## Read assets in widgets

Widgets below `MaterialApp.builder` can use the overlaid asset bundle:

```dart
Image.asset(
  'assets/banner.png',
  fit: BoxFit.contain,
)
```

For text or binary data, use the inherited bundle from a descendant widget's `BuildContext`:

```dart
final bundle = DefaultAssetBundle.of(context);
final message = await bundle.loadString('assets/message.txt');
```

`AssetImage` also uses the inherited bundle. Calls made outside that widget tree can explicitly use `assets.bundle.load(...)` or `assets.bundle.loadString(...)`.

Direct `rootBundle` calls bypass the overlay. Native resources and engine resources remain part of the store package. Images decoded before selecting the bundle may require cache eviction; loading the bundle before `runApp` and cold-starting after a code update avoids that ordering issue.

## Register text and icon fonts

Before `runApp`, register downloaded OTF/TTF data under the family used by your widgets:

```dart
await assets.loadFont(
  family: 'PatchedText',
  asset: 'assets/patched-text.ttf',
);

await assets.loadFont(
  family: 'PatchedIcons',
  asset: 'assets/patched-icons.ttf',
);
```

Use `fontFamily: 'PatchedText'` in a `TextStyle`, or `fontFamily: 'PatchedIcons'` when constructing `IconData`. Register only files available for the running patch, or handle their absence. A new font introduced by a later patch is not present in the baseline.

Changing the `fonts:` section in `pubspec.yaml` does not itself register a downloaded font at runtime. Include the compiled font file in the overlay and call `loadFont` before using the family.

## Staging and update checks

Stable is the default Shorebird channel. To test a staging patch, configure manual update checks as described in the [upstream tracks guide](https://docs.shorebird.dev/code-push/tracks/): set `auto_update: false` in the test app's `shorebird.yaml`, and pass the selected track to both code update calls.

For example, invoke this from your application's update UI or nonblocking update flow:

```dart
const track = UpdateTrack.staging;
final updater = ShorebirdUpdater();

if (updater.isAvailable) {
  final status = await updater.checkForUpdate(track: track);
  if (status == UpdateStatus.outdated) {
    await updater.update(track: track);
  }
}
```

Use `channel: track.name` in the corresponding `SaturnAssets.load` configuration. Ensure the same track is selected again after restarting the test app. A stable code check paired with a staging asset check can select inconsistent data or no bundle.

The loader does not switch the Shorebird updater's channel, download Dart patches, or restart the app. Keep channel selection in your application's existing configuration and pass it to both components.

## Cache, limits, and security

The loader verifies the manifest's Ed25519 signature against the baseline-pinned key, then verifies the downloaded ZIP's SHA-256 before extraction. It rejects unsafe archive paths, requires `AssetManifest.bin`, and limits ZIP download size to 32 MiB and expanded assets to 64 MiB/4096 entries. The cache belongs to the app/release/platform/running patch. Files not present in the overlay fall back to packaged files.

A successful server response reporting no bundle uses packaged assets. A failed request selects packaged assets. The loader never selects extracted cached overlays offline.

Use HTTPS. Asset metadata is not independently signed yet: SHA-256 detects a mismatch against that metadata, but does not protect against a compromised server replacing both. Signing Dart code patches does not sign the asset ZIP.

Android has recorded device validation. iOS remains unverified. See [supported access and limitations](../../docs/asset-support.md#supported-asset-access).

## License

LGPL-3.0-or-later. See the repository [license](../../LICENSE) and [third-party notices](../../THIRD_PARTY_NOTICES.md).

### Signed manifests and offline policy

`trustedPublicKey` is required. Define `pinnedAssetPublicKey` as a 32-byte `Uint8List` compiled into the app, never fetched from Saturn. See [signed publishing](../../docs/asset-support.md#signed-asset-publishing) for key provisioning and publisher flags. This is a breaking API change requiring a new baseline release. Every startup downloads and verifies an Ed25519 manifest and ZIP; unsigned metadata, network errors and rollback responses select packaged assets. Previously extracted files are not reused offline. Handle missing new assets before enabling features that require them. Signed content does not prove freshness against a compromised server replaying old responses.
