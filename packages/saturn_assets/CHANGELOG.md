## 0.1.0

- Initial release.
- `SaturnAssets.load` resolves the asset overlay for the running Shorebird
  patch, verifies the Ed25519-signed manifest against a baseline-pinned key,
  checks the ZIP's SHA-256, and extracts it with path, size, and entry limits.
- `SaturnAssetBundle` serves overlaid files and falls back to packaged assets.
- `SaturnAssets.loadFont` registers downloaded OTF/TTF fonts at runtime.
- Fails closed: network errors, unsigned metadata, and rollbacks select
  packaged assets.
