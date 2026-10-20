# open-Citrus - Progress Tracker

Project: Citrus - Windows port of Mandarina (MIT, upstream github.com/JorgeFTS/Mandarina).
Repo: github.com/SujayYadav776/open-Citrus (public, default branch `windows-port`).
Working copy: `E:\QwenWork\Mandarina` (remotes: `origin` = open-Citrus, `upstream` = JorgeFTS).
Last updated: 2026-10-21 (session 2). Owner: QwenWork agent; keep this file current at every milestone step.

## Done

- [x] REA installed and connected to QwenWork (rea-agents 6.7.0; MCP connector + skill; 200+ tools live).
- [x] Phase-by-phase RE plan approved (goal: full Windows port).
- [x] Phase 0 - clone at `v0.1.0-beta`, SHAs recorded, evidence ledger opened.
- [x] Phase 1 - architecture extracted: 26 IPC commands, SVG wheel, 10 converter kinds, ~20 external CLI tools, argv-only shell handoff, cancel via process-group kill, Wayland cursor dance, XDG/env config + locale.
- [x] Phase 2 - `.deb` verified: SHA-256 matches release `SHA256SUMS` (`44262a55...`); shipped scripts byte-identical to repo; binary `fb2104ab...` inspected (REA evidence `ev_918f674d...`; no cargo-auditable section; rustc commit `58cf8f9c3...` inferred).
- [x] Findings report + portability matrix + M0-M5 milestone plan delivered (`outputs/Mandarina-RE-findings-and-windows-port-plan.md`).
- [x] Repo setup on account: open-Citrus created, full upstream history pushed (attribution preserved), `windows-port` default branch.
- [x] Citrus rename (visible identity only; internals stay upstream for clean merges): commit `849491f`, pushed. 7 strings / 5 files: productName, window title, bundle description, EN/ES/PT/FR app.title, HTML title.
- [x] M0a - toolchain part 1: rustup + stable-msvc installed (rustc 1.99.1). Nice corroboration: the locally installed 1.99.1 commit `58cf8f9c3` equals the compiler commit REA inferred from the shipped `.deb` binary; the maintainer builds on this same stable.
- [x] M0a - Node v24.13.1 and WebView2 155.0.4283.62 verified present.
- [x] M0a - `cargo fetch` done (all Windows deps resolved from Cargo.lock, incl. windows 0.62, webview2-com-sys 0.39).
- [x] M0b - VS 2022 Build Tools installed via winget (job `b4lucso8s`, exit 0); MSVC 14.44.35207 verified. (Note: the earlier "Windows SDK verified" claim was wrong - SDK was absent; corrected via M0c attempt 4.)
- [x] M0c - first Windows compile reached Citrus code (after SDK fix): all ~500 deps compile; exactly 5 errors, all in the predicted Unix systems layer. Scorecard: `process_group` in `run_progress` (converters.rs:335) HIT; in `capture_with` (thumbnail.rs:62) HIT; `os::unix` CommandExt imports (converters.rs:3, thumbnail.rs:5) HIT; `kill -KILL` compile-clean (runtime-only, still M1); wheel.rs test symlink correctly absent from plain check (returns under `cargo test`, M2). NEW FIND, not in the matrix: `converters.rs:916` stages multi-file archive inputs via `std::os::unix::fs::symlink` into a TempDir to avoid copying big files; Windows plan: `std::fs::hard_link` on NTFS same-volume with copy fallback.

### M1 - Runtime layer (Windows systems code) - DONE
- [x] Process-tree kill via `crate::spawn`: Windows spawns suspended into a private Job Object with `KILL_ON_JOB_CLOSE` (assigned before first thread runs), Unix keeps `process_group(0)`. Call sites in `converters.rs::run_progress` + `thumbnail.rs::capture_with` now use `spawn_killing_tree`/`kill_tree`; `kill_group` removed. Cancel semantics (`[cancelled]`, 50 ms poll, ffmpeg progress) unchanged (verified by existing cancel tests passing).
- [x] Archive staging: `stage_input` = symlink (Unix) / `std::fs::hard_link` with copy fallback (Windows).
- [x] Config dir: added `APPDATA` fallback after `HOME` (Windows has no HOME/XDG on Explorer launch).
- [x] Locale: `lang.rs::detect` now falls through to `GetUserDefaultLocaleName` in the `LANG` slot (lowest priority; env + saved-setting order preserved).
- [x] Frontend: `editor.js` `validarNombre` rejects `\ / : * ? " < > |` and Windows reserved names (`CON/PRN/AUX/NUL/COMn/LPTn`); 4 locales updated.
- [x] OS-aware install hints: `spawn::install_hint()` maps apt names to winget packages (ffmpeg/imagemagick/pandoc/libreoffice/ghostscript/7zip) with a PATH fallback for the rest; all 27 `sudo apt install` sites converted (2 of them were inline `tr()` with no `{hint}` slot - had to switch to the `tr!("...{hint}", ..., hint = ...)` form because the catalog completeness test rejects orphaned keys and placeholder mismatch). pt/fr backend catalogs updated to the new `{hint}` keys.
- [x] `cargo check` clean and `cargo test` = 122 passed / 0 failed / 1 ignored on Windows (x86_64-pc-windows-msvc). Correction after M2 check: only ffmpeg was actually on PATH; the magick-using tool tests pass against stubbed programs, so they never proved ImageMagick was installed.

## Remaining

### M2 - Converter parity
STATUS (2026-10-21): tools installing via winget (background job `baao6cg2g`: 7zip, GnuWin32 zip/unzip, ImageMagick, pandoc, LibreOffice, Ghostscript, poppler, FLAC). Findings: `magick` was NOT on PATH during M1's test run (image tests pass via stubbed runs, so it was never proven); `unzip`/`tar` were only Git-Bash binaries (System32 bsdtar covers tar; unzip must come from GnuWin32/7z); rsvg-convert has no winget package (document as limitation; probe ImageMagick internal SVG renderer as practical fallback). Matrix driver `m2_matrix.sh` ready: 15 conversions over all 10 kinds + folder, via the repo's own `archivo_real` harness (the app's `convert_blocking` path). Code-review watch-out: `convert_folder` uses `zip -r -q -y out -- name`; GnuWin32 zip may reject `--` - if folder-zip fails there, apply a minimal cfg(windows) arg tweak.
- [ ] Locate/install remaining Windows builds (only ffmpeg was present): ImageMagick (magick), pandoc, LibreOffice (soffice), poppler (pdftoppm/pdftotext), ghostscript, 7zip, zip/unzip, flac; tar OK via System32 bsdtar (winget; detect, never auto-install in the product).
- [x] (spilled from M1) `cargo test` already green on Windows: 122/122; no path-separator fallout found yet.
- [ ] Gate: conversion matrix reproduces `docs/Converters.md` for all 10 kinds (needs the tools above; then live-run the wheel).
- [ ] Probe unknowns: HEIC support in installed ImageMagick build; rar read via 7z; cancel a soffice-spawning conversion to prove the Job Object tree-kill live.

### M3 - Wheel-at-cursor UX
- [ ] Native cursor position query replaces the Wayland enlarge-capture-shrink dance (keep `clamp_to_monitor`).
- [ ] Verify transparent, undecorated, always-on-top, skip-taskbar window on WebView2 at 100/125/150% DPI.
- [ ] Drag-and-drop free mode verified.

### M4 - Shell integration (Windows equivalents of Nautilus script)
- [ ] Explorer context menu "Convert with Citrus" for files and background (registry handlers passing selected paths as argv - the app already reads its own argv).
- [ ] Alt+M global hotkey via tauri-plugin-global-shortcut (parity with the Nautilus scripts-accels feature).
- [ ] NSIS installer (`tauri bundle` target) + icon `.ico` (already in repo).
- [ ] Decision deferred: flip internal names (binary citrus.exe, identifier com.citrus.converter, CITRUS_CONFIG_DIR) at end of port.

### M5 - Verification & close-out
- [ ] End-to-end behavior comparison Citrus vs Linux Mandarina on identical sample files.
- [ ] Findings ledger closed; limitations documented; README rewrite for Citrus (keep MIT attribution + upstream link).

## Known unknowns / risks
- Binary-to-source build equivalence: not established (no decompile needed; bounded by choice).
- WebView2 transparency edges at non-100% DPI (probe in M3).
- Multi-monitor mixed-DPI cursor placement (Wayland code only covered primary monitor; verify on Windows).

## Evidence quick-reference
- Release deb SHA-256: `44262a5516d595f3e2aa316dbed93d992d0a6f75c447607da95f193a97515541`
- Shipped ELF SHA-256: `fb2104abdfbd76d361a1775ef4c6b210b4949e382dfa0b16ee6876e0f32dd923` (6,550,896 B, x86_64)
- REA evidence ID: `ev_918f674d7a25951e721c4cf6b645c05a59291b9fba9ea2905295d11c31162404`
- Analyzed tags/commits: tag `v0.1.0-beta` = `628ac7f5`; port base = main `f2607ea`; rename = `849491f`
