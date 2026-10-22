# Release notes

## v1.2.0 - Citrus (Windows)

Improves the in-app tools installer introduced in v1.1.0 so a multi-minute
install no longer blocks the screen and you can follow it package by package.

### What's new

- **Per-package step list.** During an install, each queued tool is its own row
  that updates live — **Waiting → Installing → Done/Failed** — with the current
  winget output line beneath. Applies to both the first-run banner and the
  Settings → Converter tools section.
- **Minimizable during install.** The borderless always-on-top card becomes a
  normal, minimizable window while installing: a **Minimize** button (next to
  Stop) tucks it into the taskbar and the install keeps running in the
  background; restore it from the taskbar to see live progress. When the install
  ends, the window returns to the floating card.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_1.2.0_x64-setup.exe` (NSIS installer) | 1,669,549 B (1.59 MiB) | `8a09d0df03252ca4416cbc71dae166e80490e0e7111a5392c1d07214aecac28c` |
| `citrus.exe` (portable, x64) | 5,155,840 B | `f59b262ecdd02342419372f081e3545005f81f1f78fdb09a033b24185f642227` |

### Verification

- 124/124 unit and integration tests pass; all frontend JS parses.
- The step list + Minimize flow verified live in the app's demo mode: packages
  advance Waiting → Installing → Done with the current winget line, and the
  Minimize button appears beside Stop.

## v1.1.0 - Citrus (Windows)

Quality release on top of v0.1.0: the app is now always closable, finds its converter
tools more reliably, no longer flashes a console window, uses the original Mandarina
artwork, and can install the tools it needs from within the app.

### What's new

- **First-run tools check + one-click installer.** On launch Citrus detects which
  external tools are present. A home-screen banner (and a new **Converter tools**
  section in Settings) lists what is missing and installs it with one click through
  `winget`, streaming progress with Stop and "Copy command" options. Ghostscript,
  which is not on winget, is offered as a manual download instead.
- **Converters found off PATH.** LibreOffice, 7-Zip and Ghostscript are located in
  their known install directories even when not on `PATH`, so document, archive and
  PDF conversions work when the app is launched from Explorer (not just a dev shell).
- **No console window.** Every child tool is spawned with `CREATE_NO_WINDOW`, so no
  black cmd box flashes during a conversion.
- **Always closable.** A system tray (Open / Quit), close-to-tray on the window's ✕,
  and Esc / Alt+M to hide or quit.
- **Original icon.** The app icon is the upstream Mandarina mark again (the product
  name stays Citrus).

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_1.1.0_x64-setup.exe` (NSIS installer) | 1,667,673 B (1.59 MiB) | `53708836bac0ebf7cdc07b703180d6ce9bcf8145693ed66383d5434d836f6b87` |
| `citrus.exe` (portable, x64) | 5,151,232 B | `11fe38639ccd6a1ed9a6e3e4cc087ca4ff2a235f511a306111a79a8c065524f1` |

Per-user install to `%LOCALAPPDATA%\Citrus\`, no admin; bundles the `LICENSE`, `NOTICE`
and `AUTHORS` files.

### Verification

- 124/124 unit and integration tests pass on Windows.
- Tools-installer flow verified live: the first-run banner appears when a tool is
  missing and disappears once it is detected; the Settings section shows per-tool
  status; a real conversion spawns its tool with no console window.

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

### Fixes bundled in this build

Beyond the initial cut, this artifact also carries:

- **Always closable:** a system-tray icon (Open / Quit), close-to-tray on the window's ✕,
  and Esc / Alt+M to hide or quit — the app can no longer get stuck with no way to close it.
- **Converters found off PATH:** LibreOffice and 7-Zip are located in their known install
  directories even when they are not on `PATH`, so document and archive conversions work when
  the app is launched from Explorer (not just from a dev shell).
- **No console window:** every child tool (ffmpeg, ImageMagick, 7-Zip, pandoc, LibreOffice,
  unzip) is spawned with `CREATE_NO_WINDOW`, so no black cmd box flashes during a conversion.
- **Original icon:** the app icon is the upstream Mandarina mark again.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_0.1.0_x64-setup.exe` (NSIS installer) | 1,727,031 B (1.65 MiB) | `e23921c9d8f0bdd8545b9604c393f82ea962100c5b0a05b4184be02ee02b7e90` |
| `citrus.exe` (portable, x64) | 5,179,392 B | `ae212964c366c44de32b7c3d9275c5c7f6024fe38f5beddce05a4f26fee9f195` |

Built with rustc 1.99.1 (MSVC), LTO release profile. The installer performs a per-user
install to `%LOCALAPPDATA%\Citrus\`, carries the app icon (the original Mandarina artwork —
the product name is Citrus, the mark stays upstream's), and bundles the `LICENSE`, `NOTICE`
and `AUTHORS` files so the MIT notice ships with the app.

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
