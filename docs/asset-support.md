# Publish and inspect asset updates

Saturn asset updates are an extension to the normal Shorebird Dart-patch workflow. A code patch and an asset ZIP are separate uploads associated with the same patch. `shorebird patch` does not upload the ZIP automatically.

Start with [Using Saturn with Shorebird](usage.md) for server setup, CLI authentication, signed releases, and a comparison with hosted Shorebird. Integrate the [Flutter overlay package](../packages/saturn_assets/README.md) before building the store release that will receive asset updates.

## Before the first asset update

You need:

- An Android release built with Shorebird and containing `saturn_assets` and its startup integration.
- The exact baseline APK or AAB for that release, retained outside disposable build directories.
- A compatible Shorebird CLI and the same Flutter toolchain, signing key, flavor, release version, and build options used for the baseline.
- A reachable Saturn HTTPS origin, the matching application ID, and a management token available to the build machine.
- Python 3 for `scripts/publish-asset-bundle.py`; the publisher uses the Python standard library.

Adding `saturn_assets` to a Dart patch cannot retroactively add the native registration required by `path_provider` to an older store release. Ship a new store release first if the integration was missing.

## Publish an asset update

### 1. Build the Dart patch on staging

Make the asset and Dart changes together. Declare new asset keys in the Flutter application's `pubspec.yaml` before building. Keep the existing release version; the example below patches `1.0.0+1`:

```bash
shorebird patch \
  --platforms=android \
  --release-version=1.0.0+1 \
  --track=staging \
  --allow-asset-diffs \
  --public-key-path "$HOME/.config/saturn/signing/public.pem" \
  --private-key-path "$HOME/.config/saturn/signing/private.pem"
```

Run this from the Flutter application directory, with the [Saturn CLI environment](usage.md#2-configure-cli-authentication) set. Use `--allow-asset-diffs` only for assets supported by the integrated overlay. It does not enable native-code patching. The upstream [patch documentation](https://docs.shorebird.dev/code-push/patch/) describes the flag and standard asset restrictions.

Staging subscribers can receive this Dart patch before the bundle is uploaded. Keep test devices from updating during the upload window. Production devices should remain on stable until the matching bundle has been uploaded and checked. Saturn currently does not make the two uploads atomic or block promotion when an asset bundle is missing.

### 2. Find the server patch ID

The publisher needs the **patch ID**, not the displayed **Patch #number**. For example, `Patch #2` can have database ID `37`. The ID is not the release ID or application UUID.

Read the release and patch IDs from the authenticated management API. The following commands require `jq` on your build machine:

```bash
export SATURN_APP_ID='YOUR_SATURN_APP_ID'
export SATURN_RELEASE_VERSION='1.0.0+1'

curl --fail --silent --show-error \
  --header "Authorization: Bearer $SATURN_API_TOKEN" \
  "$SATURN_SERVER/api/v1/apps/$SATURN_APP_ID/releases" \
  | jq --arg version "$SATURN_RELEASE_VERSION" \
      '.releases[] | select(.version == $version) | {id, version}'
```

Set the numeric ID from that response, then list its patches:

```bash
export SATURN_RELEASE_ID='RELEASE_ID_FROM_RESPONSE'

curl --fail --silent --show-error \
  --header "Authorization: Bearer $SATURN_API_TOKEN" \
  "$SATURN_SERVER/api/v1/apps/$SATURN_APP_ID/releases/$SATURN_RELEASE_ID/patches" \
  | jq '.patches[] | {id, number, release_id, channel_ids}'

export SATURN_PATCH_ID='PATCH_ID_FROM_RESPONSE'
```

Choose the exact patch just built. Do not assume the largest global ID or the displayed patch number is its server ID.

### 3. Generate and review the ZIP locally

Run the publisher from the Saturn repository root. The `--base-apk` and `--patched-apk` options accept either APK or AAB inputs despite their names. Replace the absolute example paths with your build artifact paths:

```bash
python3 scripts/publish-asset-bundle.py \
  --base-apk /build-artifacts/1.0.0+1/baseline.apk \
  --patched-apk /workspace/my-app/build/app/outputs/bundle/release/app-release.aab \
  --app-id "$SATURN_APP_ID" \
  --patch-id "$SATURN_PATCH_ID" \
  --output /build-artifacts/1.0.0+1/patch-assets.zip
```

Without `--publish`, this only writes a local ZIP and prints its file list, size, and SHA-256. It does not contact the server. Review the printed files before continuing.

Use the new AAB produced by the patch build. A previous `app-release.apk` can remain in the build directory after patching; using it can accidentally compare the baseline with an old build.

The publisher includes every changed or new compiled asset relative to the store baseline, plus the patched `AssetManifest.bin`. Always compare each patch with the original baseline. If patch 2 still needs an image introduced by patch 1, patch 2's ZIP must contain that image as well.

### 4. Upload the bundle

Use the same artifact inputs and IDs, adding the server and publish options:

```bash
python3 scripts/publish-asset-bundle.py \
  --base-apk /build-artifacts/1.0.0+1/baseline.apk \
  --patched-apk /workspace/my-app/build/app/outputs/bundle/release/app-release.aab \
  --app-id "$SATURN_APP_ID" \
  --patch-id "$SATURN_PATCH_ID" \
  --server "$SATURN_SERVER" \
  --output /build-artifacts/1.0.0+1/patch-assets.zip \
  --publish
```

The script reads `SATURN_API_TOKEN`, registers an **Android** bundle, and uploads the raw ZIP. The server checks its declared size and SHA-256. One immutable bundle is allowed per patch and platform. The publisher currently has no automatic retry/resume feature.

If the only output file is `AssetManifest.bin`, publishing stops by default. Inspect your build inputs first. For an intentional manifest-only change, such as removing an asset from manifest discovery, add `--allow-manifest-only`. This does not physically remove the old asset from the installed store package; direct loads can still fall back to packaged files.

### 5. Inspect, test, and promote

1. Sign in to the Saturn dashboard and open **Patches**.
2. Select the application and locate the matching release and patch number.
3. In its **Assets** column, select **Inspect**.
4. Check the platform, **Uploaded** status, ZIP size, and SHA-256.
5. Search the file list. Select an image, text, or JSON file to preview it. Use **Download ZIP** to inspect originals locally.
6. Test on a device configured for staging in both the Shorebird updater and the asset loader. Let the code update download, then cold-start the application. Confirm the running patch number and the actual image/text/font behavior.
7. After validation, use **Promote**, select **stable**, and confirm. Stable devices can then receive the patch and its corresponding assets.

The console currently inspects bundles; it does not upload them. Continue using the publisher or management API for uploads.

## What the inspector shows

The file list represents the files included in this patch's overlay. It is not a full inventory of the installed application's assets or a before/after comparison with the baseline. Unchanged files remain in the store package and are not duplicated into the ZIP.

| File or state                                                                 | Dashboard behavior                                           |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------ |
| PNG, JPEG, GIF, WebP                                                          | Sanitized PNG thumbnail; animated images show a still frame  |
| UTF-8 `.txt`, `.json`, `.yaml`, `.yml`, `.xml`, `.csv`, `.md`, `.svg`, `.css` | Plain text; markup and scripts do not execute                |
| Fonts and compiled `AssetManifest.bin`                                        | Filename and size; no inline preview                         |
| Awaiting upload                                                               | Registration exists, but the ZIP has not completed uploading |
| No asset bundle                                                               | No bundle has been registered for this patch                 |
| Invalid ZIP or unsupported preview                                            | An error or explanation; the bundle and patch are unchanged  |

Inspection and ZIP download require administrator authentication. Archive files are not extracted to the server filesystem. Inspector limits are 4096 entries and 64 MiB expanded size. Text previews support up to 256 KiB. Image previews support up to 4 MiB and 2048 × 2048 pixels, resized to at most 768 × 768. The uploaded ZIP limit is 32 MiB.

## Runtime and rollback behavior

At startup, the Flutter integration reads the **currently running** Shorebird patch number, not the next downloaded patch number. `SaturnAssets.load` checks that exact patch, app, release version, platform, and channel with Saturn.

- Patch zero uses the packaged assets.
- A published, uploaded bundle is downloaded and checked against its SHA-256 before extraction. Files absent from the overlay fall back to the packaged assets.
- Each patch uses its own cache. Patches do not depend on the previous patch's ZIP.
- A successful server response saying that no bundle is available selects packaged assets.
- If a request fails and that running patch already has a cached overlay, the loader can reuse it. Without that cache it falls back to packaged assets. Code requiring a new asset must handle first-download failures rather than assuming the asset is always present.

**Roll back patch** stops the server from selecting its code and bundle. Code rollback takes effect after the updater processes it and the application restarts. A successful asset check for the rolled-back patch returns no bundle; an offline startup may still use that patch's cache. If the updater selects an earlier patch, its own bundle can be selected on the next startup. **Restore patch** re-enables the existing channel assignments and bundle selection. Neither action deletes the uploaded ZIP, so administrator inspection still works.

## Supported asset access

`DefaultAssetBundle.of(context)`, `Image.asset`, and `AssetImage` can use the overlay when installed as shown in the [package guide](../packages/saturn_assets/README.md). Direct `rootBundle` calls and engine/native resources still read the store package.

OTF/TTF text and icon fonts require explicit `SaturnAssets.loadFont` registration before they are used. Editing `pubspec.yaml`'s `fonts:` section alone does not register downloaded fonts.

Android has device validation. iOS is unverified; the bundled publisher is Android-only. Asset bundles have SHA-256 integrity checks but do not yet have independently signed metadata. Code-patch signing does not sign the separate ZIP. Use HTTPS and account for this security limitation when evaluating production use.

## Troubleshooting

| Symptom                                            | Check                                                                                                                                                                                             |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Shorebird blocks asset differences                 | Confirm the base release already contains the integration, then use the asset-specific patch command above                                                                                        |
| Publisher reports no changed asset file            | Use the fresh patched AAB and original baseline; reserve `--allow-manifest-only` for intentional manifest-only changes                                                                            |
| Registration returns a conflict                    | A bundle already exists for this patch/platform; the ZIP cannot be replaced. If retaining an original pending `upload_url`, retry that upload; otherwise publish a corrected new patch and bundle |
| Inspector stays on Awaiting upload                 | Registration completed but upload did not. The current publisher cannot automatically resume a pending registration                                                                               |
| Asset check reports unavailable                    | Confirm running patch number, exact release version, app ID, platform, active release, assigned channel, uploaded bundle, and rollback state                                                      |
| Code changes but image does not                    | Check the matching ZIP, `DefaultAssetBundle` integration, direct `rootBundle` usage, and restart/image cache behavior                                                                             |
| Downloaded font is not used                        | Register its logical asset key and family with `loadFont` before building the text or icon                                                                                                        |
| Bundle is visible in console but not on the device | Inspection does not promote patches or install updates. Check channel selection and cold-start the device after code download                                                                     |

## API reference

Management calls accept a bearer token or an authenticated administrator session. Cookie-authenticated mutations also require the configured same-origin request checks. The CLI/script uses the bearer token; the console uses its session cookie.

| Method and route                                                                     | Purpose                                                                                                          |
| ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| `POST /api/v1/apps/{app}/patches/{patch}/asset-bundle`                               | Register `{ "platform": "android", "sha256": "...", "size": 123 }` and receive `upload_url`                      |
| `POST /asset-uploads/{key}`                                                          | Upload raw ZIP bytes once using the returned capability URL                                                      |
| `GET /api/v1/apps/{app}/patches/{patch}/asset-bundles/{bundle}/files`                | Administrator file inventory                                                                                     |
| `GET /api/v1/apps/{app}/patches/{patch}/asset-bundles/{bundle}/files/{file}/preview` | Administrator text or sanitized image preview; `{file}` is the inventory's numeric index                         |
| `GET /api/v1/apps/{app}/patches/{patch}/asset-bundles/{bundle}/download`             | Administrator download of the original ZIP                                                                       |
| `GET /api/v1/assets/check`                                                           | Public device check with `app_id`, `release_version`, `channel`, `platform`, and `patch_number` query parameters |
| `GET /asset-objects/{key}`                                                           | Device ZIP download using the capability URL returned by a successful check                                      |

Capability URLs grant access to their object. Keep them out of shared screenshots and logs. The dashboard snapshot exposes the bundle hash but omits upload and download keys.
