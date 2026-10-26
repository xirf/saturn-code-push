# Console and password login validation — 2026-10-26

The dashboard now uses React, Vite, Radix Themes, Lucide icons and Recharts. It includes responsive application navigation, releases and patches, channel promotion/rollback dialogs, analytics, settings and a dedicated username/password sign-in page. Fonts are bundled locally. Design references included shadcn dashboard blocks (https://ui.shadcn.com/blocks?category=dashboard) and Radix Themes (https://www.radix-ui.com/themes/docs/theme/overview).

Seventeen Rust tests pass and Clippy passes with warnings denied. New tests cover Argon2id password verification, login origin/attempt limits, secure session-cookie flags, authenticated mutation origin checks, logout revocation, session expiry, persistence across SQLite reopen, and session invalidation after password changes.

Live HTTPS checks passed using PowerShell's web client: incorrect passwords are rejected; correct credentials create a Secure, HttpOnly, SameSite=Strict eight-hour session; existing app data is accessible; cross-origin and origin-less cookie mutations are rejected; logout revokes the session; the CLI bearer token remains valid; production assets are served and a same-origin content policy is applied. The live service was not restarted during these checks to avoid interrupting the user's login attempt. Restart persistence is covered by the Rust tests.

The Python HTTP test client was rejected by Cloudflare error 1010 before reaching the app. The PowerShell client completed the API checks without changing Cloudflare configuration.

The production TypeScript/Vite build passes. The password-reset helper's PowerShell syntax and Docker stdin hashing command were verified without changing the live password. Environment files, the local credential handoff file, node_modules and build outputs are excluded from Git.

Interactive visual QA is incomplete: Windows Computer Use stopped because it could not verify Chrome's current URL. No alternative browser automation was used to bypass that restriction. The development-only preview at http://127.0.0.1:5173/?preview=1 is explicitly labelled as illustrative data; production uses authenticated API data.

See local evidence at ../artifacts/saturn/login-validation.json. Credentials are in ignored .tooling/dashboard-login.txt; server/.env stores the username and password hash, not a plaintext password. Use server/set-login.ps1 to reset credentials.
