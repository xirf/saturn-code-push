# Asset overlays for Shorebird patches

Saturn now supports an Android proof path for Flutter asset changes. The server stores an immutable ZIP per app patch and platform. A Flutter package downloads that ZIP after Shorebird starts a patched app, verifies its SHA-256, and puts its files behind `DefaultAssetBundle`. The patch dashboard displays registered bundles.

This is a Saturn extension. Upstream `shorebird patch` still warns about changed assets. Use `--allow-asset-diffs` only when this package is present in the released app and a matching asset bundle has been published for the patch. If the patched Dart code requires a newly added asset, publish the asset bundle before promoting the patch to users.

## Build and publish

1. Add `packages/saturn_assets` as a path dependency to the Flutter app and wire `SaturnAssets.load` into startup as shown in [the package README](../packages/saturn_assets/README.md). Include this in a new store release; a Dart patch cannot add `path_provider` native registration to an older release.
2. Keep the baseline APK built by `shorebird release android --artifact=apk` and the patched AAB produced by `shorebird patch`. They must use the same app version and Flutter toolchain. Shorebird's patch command can leave an older APK in the output directory, so use `build/app/outputs/bundle/release/app-release.aab` as the patched input.
3. Generate the bundle from their compiled assets:

   ```powershell
   python scripts/publish-asset-bundle.py --base-apk baseline.apk --patched-apk build/app/outputs/bundle/release/app-release.aab --app-id APP_ID --patch-id PATCH_ID --output assets.zip
   ```

4. Review the listed files. To publish to the running Saturn server, set `SATURN_API_TOKEN` in the shell and add `--publish`. The script defaults to `http://127.0.0.1:3000`; use `--server` for another directly reachable Saturn instance.

The archive includes changed or new `flutter_assets` files and the patched `AssetManifest.bin`. Each patch bundle represents that patch's complete overlay relative to the store release. If patch 2 still needs an asset introduced in patch 1, patch 2's bundle must include that asset too.

## API

- `POST /api/v1/apps/{app}/patches/{patch}/asset-bundle` with bearer authentication and `{ "platform":"android", "sha256":"...", "size":123 }` registers one immutable bundle.
- `POST /asset-uploads/{key}` accepts raw ZIP bytes once and verifies their size and hash.
- `GET /api/v1/assets/check?app_id=...&release_version=...&channel=stable&platform=android&patch_number=1` selects a published bundle for the patch currently running on the device.
- `GET /asset-objects/{key}` downloads it. URLs are unguessable and tied to the registered bundle.

The Android path was verified on an Android 17 x86_64 emulator with Shorebird Flutter 3.47.8. Release `2.1.0+3` downloaded signed Dart patch 1, then loaded `PATCHED ASSET` from the associated Saturn bundle after a cold start. Rolling the patch back immediately selected the packaged `BASE ASSET`; the next cold start returned Dart to patch 0. Rollforward restored both the patch and downloaded asset.

Patch 2 added `assets/patched.png` to the compiled asset manifest and rendered it with `Image.asset`. The device downloaded patch 2, cold-started it, parsed the overlaid `AssetManifest.bin`, and displayed the new PNG. See [the emulator proof](../artifacts/saturn/asset-image-patched.png) and [runtime log](../artifacts/saturn/asset-image-patched.log).

Patch 4 downloaded two TTF files in the same overlay. `SaturnAssets.loadFont` registered Roboto Slab for ordinary text and Material Icons under a separate icon family. The emulator rendered both the changed text face and an `IconData` glyph from the downloaded font. See [the font and icon proof](../artifacts/saturn/asset-font-icon-patched-4.png) and [runtime log](../artifacts/saturn/asset-font-icon-patched-4.log).

The Flutter package supports assets loaded through `DefaultAssetBundle`, including `Image.asset` and `AssetImage`. OTF and TTF fonts are supported through explicit `SaturnAssets.loadFont` calls; this also supports icon fonts used by `IconData`. Direct calls to `rootBundle` and native or engine assets still read the store package. iOS remains unverified. The SHA-256 protects against corrupted bundles; production use should add a signature on the asset manifest so a compromised server cannot replace assets undetected.
