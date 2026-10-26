# Saturn Code Push

Independent Rust update API, dashboard and patch analytics.

Development: https://saturn.andka.id. Planned production: https://apps.gmjroaming.com.

See [server setup](server/README.md) and [Android probe](examples/saturn_probe/README.md). Quick GMJ is a separate application.

Run: `.\server\start.ps1`.

[Validated Android patch and rollback proof](docs/android-validation.md).

Install/recreate tooling: `.\scripts\bootstrap-saturn.ps1`. `server/.env` contains the local management token; it is ignored by Git.


[Console and username/password login validation](docs/console-login-validation.md).

Reset dashboard credentials: `.\server\set-login.ps1 -Username admin`.

[Flutter asset overlay and Android validation](docs/asset-support.md).
