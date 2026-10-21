# Release notes

## v0.1.0 - Citrus (Windows)

First Citrus release: the Mandarina radial-wheel file converter, ported to and shipped
for Windows. This is the release where the project identity became **Citrus** end to end.

### Identity (the rename)

The internal-name flip is complete; the app is now Citrus everywhere, not just in the
window title:

- Executable: `citrus.exe` (was `mandarina.exe`)
- Rust package / crate: `citrus` / `citrus_lib` (were `mandarina` / `mandarina_lib`)
- Bundle identifier: `com.citrus.converter` (was `com.mandarina.converter`)
- Config folder: `citrus` (was `mandarina`)
- Environment variables: `CITRUS_CONFIG_DIR`, `CITRUS_LANG`, `CITRUS_REAL`,
  `CITRUS_FORMAT` (were the `MANDARINA_*` equivalents; the old names are not accepted)

This was a clean break: settings written under the old `mandarina` config folder are not
migrated. Upstream attribution is preserved through the `repository` field in
`Cargo.toml`/`package.json` and the MIT `LICENSE` copyright.

### What Citrus does

Local file conversion driven by a radial wheel that appears at the cursor. Ten kinds and
250+ conversions (images, audio, video, PDF, documents, presentations, spreadsheets,
subtitles, archives, folders) plus tools (compress, strip metadata, trim, rotate, flip,
resize, remove audio). Nothing is uploaded; conversions run on your machine using
external command-line tools (ffmpeg, ImageMagick, pandoc, LibreOffice, poppler, 7-Zip,
zip/unzip, FLAC).

### Windows integration

- Explorer context menu: "Convert with Citrus" on files, folder icons, and folder
  background. Register with `packaging\register-explorer.ps1` (HKCU only, no admin;
  `-Remove` to undo).
- Alt+M global shortcut: summons or hides the drop card at the cursor while the app runs.
- Native cursor placement: the wheel appears exactly at the pointer (verified at 150% DPI);
  the Wayland enlargement workaround is kept only as a Linux fallback.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_0.1.0_x64-setup.exe` (NSIS installer) | 1.61 MiB | `c24baaf602bd4be4aaa4c9242c4918d290828cb2e8a2270946fa999c3a2f881f` |
| `citrus.exe` (portable, x64) | 5,051,392 B | `6344357411d1814b1c5a5de3284ad483295af08fb64dd0729af47b1f8933afde` |

Built with rustc 1.99.1 (MSVC), LTO release profile. The installer performs a per-user
install to `%LOCALAPPDATA%\Citrus\`, carries the Citrus icon, and bundles the `LICENSE`,
`NOTICE` and `AUTHORS` files so the MIT notice ships with the app.

### Verification

- 123/123 unit and integration tests pass on Windows, including a Job Object test proving
  the process-tree kill sweeps descendant processes (soffice children included).
- Conversion matrix reproduces the upstream `docs/Converters.md` for all ten kinds (16/16
  conversions) through the app's own `convert_blocking` path.
- Silent-install smoke test: the NSIS package installs `citrus.exe` and launches with the
  "Citrus" window.

### Known limitations

- Ghostscript is not available via winget, so the PDF size-reduction tools are unverified;
  every other conversion works without it.
- SVG center thumbnails fall back to ImageMagick's renderer (no `rsvg-convert` on winget);
  complex SVGs may render less faithfully.
- HEIC is input only and AVIF is read/write, matching the upstream ImageMagick build.
- Explorer multi-selection invokes the verb once per file, so each selected file opens its
  own wheel (the Linux Nautilus script received them together).
- The Linux `.deb` packaging still uses the old `mandarina` names; the port targets
  Windows, so it was intentionally left unchanged.

### Provenance

Citrus is a Windows port of [Mandarina](https://github.com/JorgeFTS/Mandarina) by JorgeFTS,
used here under the MIT license. The full upstream commit history is preserved in this
repository's git log.
