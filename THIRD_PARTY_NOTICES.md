# Third-party notices

Saturn's original source is licensed under [LGPL-3.0-or-later](LICENSE). Dependencies and bundled third-party assets retain their respective licenses.

## Bundled console assets

- Generated shadcn-vue components and the vendored base stylesheet use the MIT license. See [shadcn-vue notice](console/public/licenses/shadcn-vue.txt).
- DM Sans and Manrope are bundled locally through Fontsource. Their font license texts are retained in `console/public/licenses/`.
- Other files in `console/public/licenses/` retain their existing attribution notices, including assets used by earlier builds.

The production console exposes these notices under `/licenses/`. Package lockfiles and `server/Cargo.lock` identify dependency versions. Consult each dependency's distribution for its license terms.

## Shorebird compatibility

Saturn is an independent server implementation of a supported subset of the public Shorebird protocol. It uses Shorebird-distributed tooling to build Flutter releases and patches. Shorebird and Flutter projects retain their own licenses and trademarks; the Saturn license does not relicense those projects or imply their endorsement.
