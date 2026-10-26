# Android compatibility proof — 2026-10-26

Result: PASS on the Android Studio Medium_Phone emulator, Android 17 / API 37, x86_64 (16 KiB page system image).

The baseline APK embedded our RSA public key and `base_url: https://saturn.andka.id`. Its package was `id.andka.saturn_probe`, separate from the Quick GMJ customer application.

The actual Shorebird CLI published release `1.0.0+1` to our Rust backend. It built and signed patch 1, uploaded the 52,209-byte binary diff and promoted it to staging. Our verification script then promoted it to stable.

The device downloaded that diff over the Cloudflare HTTPS tunnel. Its logs explicitly reported `Verifying patch signature...` and `Patch signature is valid`, reconstructed the artifact and booted with `SATURN_PROBE marker=patched current=1 next=1 available=true`.

After a server rollback, the next check reported `restartRequired`. The subsequent boot reported `SATURN_PROBE marker=baseline current=0 next=0 available=true`. The entire sequence passed twice, including restoring the rolled-back patch for the second run. The final state leaves patch 1 rolled back.

The HTTPS analytics endpoint recorded the real download and first-launch events and one reporting install. These are bounded, client-reported metrics rather than active-device or total-adoption claims.

## Evidence

- [Patch download and signature verification](../artifacts/saturn/download.log)
- [Patched boot](../artifacts/saturn/patched.log)
- [Patched screen](../artifacts/saturn/patched.png)
- [Rollback boot](../artifacts/saturn/rolled-back.log)
- [Rollback screen](../artifacts/saturn/rolled-back.png)
- [REA runtime evidence](../artifacts/saturn/rea-device-evidence.json)

Evidence files are local and ignored by Git. REA read only the probe process's bounded logcat output. The executable-patch proof comes from the actual emulator runtime logs, not from the synthetic protocol fixtures.

## Other checks

All 12 Rust tests pass, including persistence, scoped selection, rollback, immutable uploads, hashes, byte ranges, authenticated management, analytics filtering, Dart's text/plain JSON wire format and rate limits. Clippy passes with warnings denied. The actual upstream Dart client also passed live response decoding and both multipart upload flows against the Docker service.

The dashboard's HTTPS response and JavaScript syntax were checked. No connected browser was available for interactive dashboard QA; its management actions are covered through the API tests.

## Scope

This proves this pinned Android engine, architecture and release/patch cycle. It does not prove iOS compatibility, all Flutter versions, all failure-recovery cases or production-scale operation. Compiler and engine artifacts still come from Shorebird. Production remains planned for `apps.gmjroaming.com`; it has not been deployed there.

Standalone probe analysis also passes with no issues. The final public HTTPS protocol check passed from the standalone project, and unauthenticated management requests returned HTTP 401.

