# Release notes

## v1.2.5 - Citrus (Windows)

Fixes the v1.2.4 click‑through (it had disabled the whole window, not just the empty ring)
and turns the first‑run tools banner into a proper setup checklist.

### What's new

- **Click‑through now only affects the empty ring.** The v1.2.4 feature compared a *global*
  screen cursor coordinate against the window's own size, so the computed distance was always
  outside the circle and the *entire* window - including the drop circle - ignored the mouse:
  you could not hover or drop files on it. The poller now converts the cursor to a
  window‑relative position first, so only the transparent area outside the circle falls
  through to the desktop, while the circle stays interactive.
- **First‑run setup checklist.** The tools banner is now a checklist of every required tool
  (FFmpeg, ImageMagick, pandoc, LibreOffice, 7‑Zip, Poppler, Ghostscript), each row showing a
  status mark - ✓ installed, ○ missing, … installing, ✕ failed - with a "X of N ready"
  counter on top. **Install all** runs the in‑app winget installer and ticks rows off live.
- **Dismissible and re‑openable.** "Not now" hides the checklist and leaves a small **Setup**
  pill on the home card to bring it back anytime. It only appears when at least one tool is
  missing, so a fully set‑up machine stays uncluttered.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_1.2.5_x64-setup.exe` (NSIS installer) | 1,692,415 B (1.61 MiB) | `acf81f1ec5e8ef384aac1ef5f397ae343d196e83e121364acbdffcaed8ed362b` |
| `citrus.exe` (portable, x64) | 5,216,256 B (4.97 MiB) | `18515fb78e3caf2840d8ac3d540ad58cae8a4c38c37bb3df3192bff70caca00b` |

### Verification

- Frontend only for the checklist; the click‑through fix is in `wheel.rs`.
- `cargo test --lib`: 131 passed, 0 failed, 1 ignored.
- GitHub Actions green on both ubuntu-24.04 and windows-latest.

## v1.2.4 - Citrus (Windows)

Fixes the drop card swallowing desktop clicks: the empty area around the circle is now
click‑through, so you can click whatever is behind Citrus without dismissing the card.

### What's new

- **The transparent area no longer blocks the desktop.** Citrus's window is a transparent
  460×460 rectangle, and WebView2 was capturing every mouse click inside it — so icons or
  taskbar items behind the empty ring around the "drop a file" circle couldn't be clicked.
  A lightweight poller now watches the global cursor and, while the pointer is outside the
  interactive circle, makes the window ignore mouse input so clicks fall straight through to
  the desktop. Move back over the circle and it becomes interactive again (within ~50 ms).
- **The circle itself still captures clicks.** Clicks and file drops inside the drop circle
  are handled by Citrus as before, and the conversion wheel, result cards, editor and
  settings stay fully interactive — only the empty surround is click‑through.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_1.2.4_x64-setup.exe` (NSIS installer) | 1,688,622 B (1.61 MiB) | `7702a0de4acb6a9a68828f3cf075b6ab3e5164907c6c5ff96750624a5ab1151a` |
| `citrus.exe` (portable, x64) | 5,209,600 B (4.97 MiB) | `530eacf40d00b650b527a549083f9e30b32be323aa05f9c77a2b8a3430f5e339` |

### Verification

- `cargo test --lib`: 131 passed, 0 failed, 1 ignored.
- GitHub Actions green on both ubuntu-24.04 and windows-latest.
- Click‑through verified working on a real Windows device.

## v1.2.3 - Citrus (Windows)

Second production-readiness hardening pass: robustness fixes across conversions,
the in-app tools installer, configuration and Windows shell integration. No
user-facing UI redesign (the README was rewritten for first-time users).

### What's new

- **A hung conversion can no longer run forever.** Every external tool run now has a
  generous watchdog (2 h). If a tool truly hangs - ffmpeg spinning on a corrupt file, a
  LibreOffice instance stuck on an invisible dialog - it is stopped and reported as
  "took too long and was stopped" instead of blocking the batch.
- **Packing a folder works on Windows.** "Compress" of a selection that included a
  folder used to fail (the staging step only handled files). Folders are now staged with
  a recursive copy that skips symlinks/junctions and caps depth, so it can't loop.
- **No memory blow-up on huge audio waveforms.** When a file's duration is unknown, the
  waveform now accumulates only a bounded number of samples while still draining the
  decoder, so a very long or streamed file can't exhaust memory.
- **Alt+M is debounce-safe.** A rapid double-press can no longer start two toggles that
  fight over the card's visibility while the ~200 ms resize is in flight.
- **Settings save is collision-proof.** The config is written to a uniquely named temp
  file (process id + counter) and renamed into place, and the temp is cleaned up on
  failure, so two savers can't clobber each other.
- **Single instance.** Launching Citrus again (e.g. right-clicking another file while it
  already runs) no longer spawns a duplicate tray icon or a second Alt+M; the running
  instance receives the new file paths and shows them instead.
- **Explorer right-click menu actually works.** The registration script was writing the
  command into a value literally named "(Default)" instead of the key's real default
  value, so the menu item appeared but did nothing. Fixed - "Convert with Citrus" now
  launches.
- **Screen-reader announcement.** The wheel's center pill is now a polite live region.
- **Docs & README.** The README was rewritten around a beginner-friendly 3-step quick
  start (install → one-click tools → convert) with screenshots and a troubleshooting
  section; the developer docs gained the installer commands/events, the timeout, the
  async file-info and the strengthened name validation.

### Deferred

- **Content-Security-Policy** was evaluated but left at `null` this release: it could not
  be runtime-verified in the build environment, and an incorrect CSP would silently blank
  the whole app, so it ships only after a device smoke test.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_1.2.3_x64-setup.exe` (NSIS installer) | 1,684,957 B (1.61 MiB) | `3573aa26d8620326c775add1d435cb8168d28e9b2f6f65b36d344f187fd06d89` |
| `citrus.exe` (portable, x64) | 5,197,824 B (4.96 MiB) | `13acbf583b961e8ddf91968fe36b6b932982c95b172a367a04ae074541f36e6a` |

### Verification

- `cargo test --lib`: 131 passed, 0 failed, 1 ignored.
- GitHub Actions green on both ubuntu-24.04 and windows-latest.
- Installer rebuilt from this tagged tree; the size/SHA-256 above are from that build.

## v1.2.2 - Citrus (Windows)

Hardening pass driven by a full production-readiness audit: path and URI safety, a
non-blocking file inspector, a single-flight tools installer, and CI that now compiles
the Windows-only code. No functional UI changes.

### What's new

- **No path-traversal or illegal names.** "Save as…" names are now validated: reserved
  Windows names (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`) and any
  `\ / : * ? " < > |` or control character are rejected, so a crafted name can no longer
  write outside the output folder.
- **LibreOffice conversions survive spaces in the temp path.** Paths handed to LibreOffice
  as `file://` URIs are now percent-encoded per RFC 3986, fixing failures whenever the
  profile or `%TEMP%` directory contained a space or other special character.
- **The wheel no longer freezes on big folders.** `file_info` (folder size and file
  metadata) runs off the main thread, and folder-size recursion is capped at 64 levels so a
  deep or symlink-heavy tree cannot hang the app.
- **One install at a time.** Starting a second tool install while one is already running is
  rejected ("An installation is already running") instead of corrupting the shared
  cancel/pid state.
- **Already-installed tools are skipped server-side.** The installer re-checks presence for
  each tool before invoking winget and reports it as done immediately, so even a stale key
  never spawns a redundant install.
- **CI now covers Windows.** The suite runs on both `ubuntu-24.04` and `windows-latest`, so
  the `#[cfg(windows)]` code (Job Objects, the tools installer, tray, taskbar window
  toggles) is compiled and tested on real Windows, not only Linux.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_1.2.2_x64-setup.exe` (NSIS installer) | 1,674,520 B (1.60 MiB) | `bb11e7007339a9f8b839bcc067f02f3f6c6c31376127483d2843cdd6fd82d032` |
| `citrus.exe` (portable, x64) | 5,171,712 B (4.93 MiB) | `31e9b7d1418fde11b6c928944b15c2758c9a6f3a2899a68b441efc23acf276df` |

### Verification

- `cargo test --lib`: 128 passed, 0 failed, 1 ignored.
- GitHub Actions green on both ubuntu-24.04 and windows-latest.
- Installer rebuilt from this tagged tree; the size/SHA-256 above are from that build.

## v1.2.1 - Citrus (Windows)

Patch release hardening the in-app tools installer (fixes installs that appear to hang at
"Preparing to install", e.g. LibreOffice).

### What's new

- **Elevation can proceed.** Dropped `--disable-interactivity` from the winget command, so
  machine-wide tools (LibreOffice, 7-Zip, ImageMagick, pandoc) can properly raise their
  admin/UAC prompt instead of stalling.
- **Per-tool admin hint.** Machine-wide tools now show a "Needs admin; may take a few
  minutes" hint in both the first-run banner and Settings → Converter tools, so a slow
  LibreOffice install is expected rather than mistaken for a freeze.
- **Timeout watchdog.** Each package has a time budget (30 min for the large MSIs, 15 min
  for the rest). If one runs past it, the in-flight installer is stopped and reported as
  **"Timed out — try installing it manually"** instead of hanging forever; the batch then
  continues.

### Release artifacts

| File | Size | SHA-256 |
|---|---|---|
| `Citrus_1.2.1_x64-setup.exe` (NSIS installer) | 1,671,661 B (1.59 MiB) | `f5c24edd0b41a4d06bddcb8ffc597ffa23904c318cdfebbb38eca54e1f17e129` |
| `citrus.exe` (portable, x64) | 5,158,912 B | `b01bf7ca82bfee267ebcf3913c0ea33267e1774a72f546e02559e02c27c829c1` |

### Verification

- 124/124 tests pass; all frontend JS parses.
- Timeout watchdog and admin hint verified to compile and render; the per-package step list
  now also shows the Timed-out state.

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
