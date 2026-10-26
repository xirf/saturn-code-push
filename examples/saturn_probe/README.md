# Saturn Android probe

A separate test application (`id.andka.saturn_probe`) for the self-hosted Saturn API. This is not the Quick GMJ customer app.

The app displays a compile-time `PROBE_MARKER` and its running/next patch numbers. On startup it checks for patches and downloads an available update without blocking the first screen. Logs begin with `SATURN_PROBE` so the verification script can collect evidence.

With the Android Studio emulator running and Saturn listening on port 3000:

```powershell
.\scripts\saturn-probe.ps1 -Phase prepare
.\scripts\saturn-probe.ps1 -Phase release
.\scripts\saturn-probe.ps1 -Phase patch
.\scripts\saturn-probe.ps1 -Phase verify
```

Run these commands from the repository root. Preparation creates a dedicated app on our server and a local RSA signing key in ignored `.tooling/saturn-signing`. Release builds and installs a baseline APK. Patch builds a signed variant with marker `patched` and publishes it to staging. Verify promotes it to stable, checks the downloaded and running patch, requests rollback, and checks a return to baseline. Verification leaves that patch rolled back; the dashboard can restore it.

Evidence is written to ignored `artifacts/saturn/*.log`. The source and scripts remain reviewable; private signing keys and generated build outputs are not committed.

The current test uses HTTPS at `saturn.andka.id`. A loopback-only cleartext exception also permits an explicit local `adb reverse` test; it applies solely to this test app. The later production hostname is `apps.gmjroaming.com`.

The pinned Shorebird and REA toolkits are installed under `%LOCALAPPDATA%\SaturnCodePush` on C:. Their paths are recorded in ignored `.tooling/shorebird-path.txt` and `.tooling/rea-path.txt`. Run `scripts/bootstrap-saturn.ps1` to recreate them. Build outputs are disposable; they are not part of this repository.
