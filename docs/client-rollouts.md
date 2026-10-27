# Flutter code updates and rollout identity

Saturn gradual rollouts use Shorebird's existing native installation identity.
A separate Saturn identity package or engine fork is not needed for an engine that
implements the protocol described here. Asset overlays remain optional and separate.

## Verified upstream contract

Source review of `shorebirdtech/updater` at commit
`d9da1d37116106830133fa332da27151478df37f` establishes:

- [Updater state](https://github.com/shorebirdtech/updater/blob/d9da1d37116106830133fa332da27151478df37f/library/src/cache/updater_state.rs) generates a random UUID v4, saves it in `state.json`, reloads it, and preserves it on release-version changes.
- [Patch-check requests](https://github.com/shorebirdtech/updater/blob/d9da1d37116106830133fa332da27151478df37f/library/src/network.rs) serialize that identity as `client_id` along with application, release, channel, platform, architecture, and the optional `current_patch_number`.
- [The updater](https://github.com/shorebirdtech/updater/blob/d9da1d37116106830133fa332da27151478df37f/library/src/updater.rs) obtains that identity from native state when making checks.

This is source verification, not a claim that every released engine uses that
revision. The Dart `shorebird_code_push` dependency wraps the engine embedded by
`shorebird release`; updating the Dart package alone does not update an installed
engine. Record the CLI, Flutter/engine, and Dart package versions you validate.
The existing Android validation predates rollout validation.

Do not generate another identity in Flutter preferences or attempt to set one
through `ShorebirdUpdater`: its public Dart API does not expose that parameter.
Do not edit native state files. Reinstalling, clearing app data, losing state,
or recovering from corrupt state can change the identifier. Identity stability
therefore depends on preserving the native updater's application storage.

## Reference application

The repository includes `examples/rollout/` with a minimal `pubspec.yaml`,
`shorebird.yaml`, and `lib/main.dart`. It uses the normal `ShorebirdUpdater` API;
there is no extra plugin or dependency beyond `shorebird_code_push`.

From that directory, with compatible Flutter and Shorebird tooling:

```bash
flutter create --platforms=android --project-name=saturn_rollout_example .
flutter pub get
```

Keep the provided `lib/main.dart`, dependency declarations, and `shorebird.yaml`.
Replace the placeholder app ID and HTTPS origin. Then follow the normal signed
baseline and patch publishing workflow in [Using Saturn](usage.md).
Include `shorebird.yaml` in Flutter assets before building the baseline.

The example disables native automatic updates with `auto_update: false` so its
button owns check/download timing. Both calls use the same `UpdateTrack.stable`.
For automatic background updates, use the normal engine configuration instead;
the native identity works for both flows. Do not add a Dart eligibility precheck
in front of an independently running auto-updater.

An excluded installation may see `upToDate`: that means no eligible update is
available to it, not that every installation is on the same patch. A successful
download applies after restart. The example handles no-update races and network
errors without blocking startup or forcing a restart.

If using [asset overlays](../packages/saturn_assets/README.md), load assets for
`readCurrentPatch()` on startup. Do not select assets with `readNextPatch()` or
independently assign an asset cohort. Pausing code delivery must not withhold
assets from a device already running that patch.

## Server smoke test

`scripts/check-rollout.py` sends native-shaped patch checks with stable simulated
UUIDs. It does not publish, download patches, or require management credentials.
Freeze publishing and rollout changes while it runs. Use a dedicated test app
and the exact active baseline, platform, architecture, and channel containing the
target patch. `--target-patch` is its patch number, not the database patch ID.

From the repository root:

```bash
python3 scripts/check-rollout.py \
  --server https://updates.example.com \
  --app-id YOUR_SATURN_APP_ID \
  --release-version '1.0.0+1' \
  --platform android --arch aarch64 --channel stable \
  --target-patch 2 --current-patch 1 \
  --cohort community-validation --installations 20 \
  --expect-percentage 25
```

Reuse the same cohort name to reuse the simulated installation identities.
The report gives the target offer count, observed percentage, repeated-check
stability, and whether an ID-less client received the target. Small samples need
not match the configured percentage exactly; this tool does not assert an exact
partial-rollout quota. It asserts that 0% never offers the target, 100% offers it
to every simulated installation and the ID-less client, and a partial rollout
excludes the ID-less client. A failed request or assertion returns nonzero.
The tool does not print installation identifiers or artifact capability URLs.

This checks the server contract, not whether an installed engine sends an ID.

## Device acceptance checklist

Before advertising compatibility with a particular engine version:

1. Build an Android baseline pointing to a disposable Saturn app. Record toolchain versions.
2. Publish two distinguishable patches. Leave the older patch at 100% and set the newer one to a partial percentage.
3. Through a private development request capture, confirm a nonempty `client_id` is sent by the native engine and stays unchanged across repeated checks and app restarts. Do not publish raw IDs or capability URLs.
4. Confirm included and excluded installations keep their assignments. Raise the percentage and confirm previously included installations remain included.
5. Set 0% and verify new checks stop offering the target. Confirm an already running patch keeps its matching assets.
6. Raise to 100% and confirm previously excluded installations receive the target. Restart to apply it.
7. Roll back and confirm rollback instructions are honored after checking and restarting. Restore and verify the saved percentage is retained.
8. Upgrade the baseline store release and verify identity persistence. Treat reinstall/data-clear as a potentially new installation.

Saturn's protocol/cohort tests and smoke tool are preparation for this validation.
They do not replace a device run. iOS remains unverified.
