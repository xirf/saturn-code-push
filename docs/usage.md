# Using Saturn with Shorebird

Saturn provides the update server and dashboard. The Shorebird CLI still builds the enabled Flutter release and its Dart patches. Saturn implements a tested subset of the protocol; it is not an official self-hosted Shorebird distribution or a complete replacement for every hosted feature.

For asset changes, Saturn adds a separate ZIP publisher and the `saturn_assets` Flutter package. Follow the [asset update guide](asset-support.md) after the initial setup below.

## What differs from regular Shorebird?

| Area                    | Regular hosted Shorebird                                   | This Saturn implementation                                                                            |
| ----------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Build commands          | `shorebird release` and `shorebird patch`                  | The same CLI command families, directed at Saturn                                                     |
| Management server       | Shorebird's hosted service                                 | Your Saturn origin                                                                                    |
| CLI credentials         | Your Shorebird credentials/API key                         | An app-scoped Publish API key, supplied as `SHOREBIRD_TOKEN`                                                 |
| Dashboard access        | Hosted account                                             | Saturn administrator username/password                                                                |
| Dart patches            | Built by Shorebird tooling                                 | Built by Shorebird tooling and served by Saturn                                                       |
| Asset changes           | Asset patching is not supported by the standard patch flow | A separate overlay package and ZIP publishing step                                                    |
| Images, text, and fonts | Packaged in the store release                              | Supported overlays require `saturn_assets` in the store release; fonts also need runtime registration |
| Patch promotion         | Hosted tracks                                              | Saturn channels, managed from the dashboard                                                           |
| Rollback scope          | Track-specific behavior is documented upstream             | Saturn marks a patch rolled back across all of its assigned channels                                  |
| Hosted features         | The upstream product's feature set                         | Multiple accounts and roles; no hosted billing, OAuth, or managed signing        |

The upstream [patch guide](https://docs.shorebird.dev/code-push/patch/) explains the normal Dart-patch flow and its asset restrictions. `--allow-asset-diffs` bypasses a check; it does not publish asset bytes. Saturn's extra upload is still required.

The [Saturn package PR](https://github.com/xirf/shorebird/pull/1) adds `packages/saturn_assets` to the fork. The reviewed commit is `1625363cc53c4091f88d5a3daaa8de9b12fb5c6d`. Its changes do not modify the CLI, compiler, or engine, and do not automate bundle publishing. The package is also present in this repository. See [package installation](../packages/saturn_assets/README.md#installation).

## 1. Set up the server and application

Deploy Saturn using the [Linux/Podman server guide](../server/README.md#start). Use Docker for the container development environment. The commands below are Bash commands for the machine or CI runner that builds your Flutter application; they do not require Flutter tooling inside the deployed server container.

Sign in to the dashboard and select **New application**. Saturn opens the new application, whose getting-started checklist and **Settings → General** page show its application ID and a ready-made `shorebird.yaml`. Use that ID in your Flutter project's `shorebird.yaml`:

```yaml
app_id: YOUR_SATURN_APP_ID
base_url: https://updates.example.com
```

Replace the example origin with the HTTPS origin configured as Saturn's `PUBLIC_BASE_URL`. Include `shorebird.yaml` in your Flutter assets. Keep the app's `pubspec.yaml` version, for example `1.0.0+1`, fixed when making patches for that release.

Existing ordinary Flutter installations need a new store release built with Shorebird before receiving Dart patches. Existing Shorebird releases without `saturn_assets` need a new store release containing that package before using Saturn asset overlays.

Changing `base_url` on disk does not redirect an already installed release. Its bundled configuration must already point to Saturn.

## 2. Configure CLI authentication

Use a compatible Shorebird CLI/toolchain. The recorded Android validation used CLI `1.6.133` and Shorebird Flutter `3.47.8`; see [validation details](../server/README.md#validation). Recheck compatibility when upgrading the CLI. Saturn still relies on Shorebird-distributed build tooling.

In the Bash session that runs your build:

```bash
export SATURN_SERVER='https://updates.example.com'

read -r -s -p 'Saturn Publish API key: ' SATURN_API_TOKEN
printf '\n'
export SATURN_API_TOKEN

export SHOREBIRD_HOSTED_URL="$SATURN_SERVER"
export SHOREBIRD_TOKEN="$SATURN_API_TOKEN"
```

`SHOREBIRD_HOSTED_URL` selects the CLI's server. The app uses the `base_url` bundled in `shorebird.yaml`. Configure both for the same deployment. The CLI's endpoint override is visible in [upstream environment configuration](https://github.com/shorebirdtech/shorebird/blob/main/packages/shorebird_cli/lib/src/shorebird_env.dart).

The asset publisher reads `SATURN_API_TOKEN`. The CLI reads `SHOREBIRD_TOKEN`. Both values are the same Saturn management token in this setup. In CI, inject them from your secret store instead of using an interactive prompt.

Do not put these tokens in Flutter code, `shorebird.yaml`, `pubspec.yaml`, a `--dart-define`, screenshots, or Git. Devices use the public update-check API; they do not need an administrator token. The dashboard uses its own HTTP-only session cookie, rather than these environment variables.

## 3. Create a signed baseline release

Keep the signing private key outside the application repository and server image. If you already have a signing key for this application, reuse it. For a new application, generate a pair once:

```bash
umask 077
mkdir -p "$HOME/.config/saturn/signing"

openssl genpkey \
  -algorithm RSA \
  -pkeyopt rsa_keygen_bits:2048 \
  -out "$HOME/.config/saturn/signing/private.pem"

openssl pkey \
  -in "$HOME/.config/saturn/signing/private.pem" \
  -pubout \
  -out "$HOME/.config/saturn/signing/public.pem"
```

Run from your Flutter application directory:

```bash
shorebird release android \
  --artifact=apk \
  --public-key-path "$HOME/.config/saturn/signing/public.pem"
```

Keep the exact baseline build as a CI artifact, with the application ID, release version, Flutter revision, and build options. Asset publishing compares the patched build to this baseline, not to the previous patch. For an unflavored APK build:

```bash
mkdir -p saturn-baselines/1.0.0+1
cp build/app/outputs/flutter-apk/app-release.apk \
  saturn-baselines/1.0.0+1/baseline.apk
```

Keep generated baselines and signing keys out of Git. An Android AAB can also be used by the asset publisher. Adjust build paths for flavors and your CI artifact layout. Distribute the real release through your normal store or test-device process; the Saturn dashboard does not install the baseline app for you.

If this release must support asset updates, integrate the [Flutter package](../packages/saturn_assets/README.md) **before** building this baseline.

## 4. Publish a Dart-only patch

Keep the same release version, Flutter toolchain, public key, flavor, and relevant build options as the baseline. Change the Dart code, then publish to staging:

```bash
shorebird patch \
  --platforms=android \
  --release-version=1.0.0+1 \
  --track=staging \
  --public-key-path "$HOME/.config/saturn/signing/public.pem" \
  --private-key-path "$HOME/.config/saturn/signing/private.pem"
```

For a patch with changed assets, use the [asset workflow](asset-support.md#publish-an-asset-update) instead; it adds `--allow-asset-diffs` and the ZIP upload.

In Saturn, open the application and go to **Releases** or **Patches**. Confirm the patch's release and uploaded artifacts. Use **Promote** to make the patch available on the selected channel. A patch uploaded to staging is already eligible for devices subscribed to staging; it is not available to stable devices until promoted there.

Staging test devices must ask for the same channel in both the Shorebird updater and `SaturnAssets.load`. [Channel setup](../packages/saturn_assets/README.md#staging-and-update-checks) explains the manual updater calls. A downloaded code patch takes effect after an app restart; inspecting its files in the dashboard does not apply it to a device.

## 5. Roll back or restore

In **Patches**, open the patch's action menu and select **Roll back patch**. Confirm the action. The server stops selecting the patch and reports its rollback to update checks. Devices use a previous eligible patch or the baseline after the updater processes the rollback and the app restarts.

Saturn's rollback flag applies to the patch across all assigned channels. It is not a per-channel rollback switch. **Restore patch** makes it eligible again on its existing channels.

For asset bundles, the next successful asset check stops selecting the rolled-back bundle. Cached assets may still be used during a network failure. The bundle remains available for administrator inspection and is not deleted. See [asset runtime behavior](asset-support.md#runtime-and-rollback-behavior).

## Continue

- [Publish and inspect asset updates](asset-support.md)
- [Integrate the Flutter overlay package](../packages/saturn_assets/README.md)
- [Server setup, credentials, and deployment](../server/README.md)
- [Security and current limitations](../SECURITY.md)


## Gradual patch rollouts

In **Patches**, choose **Promote**, select a channel, and set its rollout percentage.
Start at 5%, monitor launch failures, then promote the same patch to 25%, 50%, and
100%. Each channel has its own setting. Existing patches default to 100%.
The channel badges show partial percentages and paused delivery.

0% pauses new delivery; it does not uninstall downloaded or running patches.
Reducing a percentage also does not uninstall patches. Use **Roll back** to remove
a faulty patch on subsequent updater checks and restarts. Restore preserves rollout settings.
These controls are manual; there is no automatic failure threshold or scheduled expansion.

Partial rollouts require the updater's patch-check request to include a nonempty,
persistent `client_id` (at most 256 bytes). Shorebird's upstream native updater already generates, persists, and sends this field.
See [Flutter code updates and rollout identity](client-rollouts.md) for the verified
source revision, reference application, smoke tool, and device validation steps.
Saturn does not modify the upstream engine or inject an installation ID. Clients without a valid ID remain on the previous
eligible patch until rollout reaches 100%. Do not use an IP address or a random
per-request value as the identity.

Saturn hashes the patch, channel, and client ID into a stable bucket. Increasing the
percentage preserves existing cohort membership. This is an approximate percentage
of identifiers, not a fixed device quota or an A/B experiment. An excluded device can
receive an older eligible patch; explicit rollback instructions remain independent
of cohort selection. Assets remain bound to the running patch identity.

CI can use the existing authenticated promotion endpoint:

```json
{"patch_id": 123, "channel_id": 1, "rollout_percentage": 5}
```

`rollout_percentage` must be an integer from 0 through 100. Omitting it preserves
that channel's current setting, defaulting to 100% for a new promotion. These settings
use the existing publisher permissions and audit trail. Device IDs are client claims,
so cohorts are a delivery mechanism, not an authorization boundary.
