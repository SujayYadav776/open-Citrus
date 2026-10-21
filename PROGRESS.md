# open-Citrus - Progress Tracker

Project: Citrus - Windows port of Mandarina (MIT, upstream github.com/JorgeFTS/Mandarina).
Repo: github.com/SujayYadav776/open-Citrus (public, default branch `windows-port`).
Working copy: `E:\QwenWork\Mandarina` (remotes: `origin` = open-Citrus, `upstream` = JorgeFTS).
Last updated: 2026-10-21 (M5 close-out). Owner: QwenWork agent; keep this file current at every milestone step.

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

### M2 - Converter parity - DONE (commit 76a0b89, pushed)
- [x] Conversion matrix **16/16 PASS** via the repo's own `archivo_real` harness (app's `convert_blocking` path): image, audio flac+mp3, video webm+gif, pdf txt+jpg, docx->pdf, md->docx, csv->xlsx, pptx->pdf, srt vtt+txt, zip->tar, folder->zip, folder->tar.gz.
- [x] Tools installed via winget (8/9); 7-Zip + LibreOffice add no PATH entries (product must handle, logged for M4); Ghostscript absent from winget entirely - only the PDF-compression tools need it, open sourcing decision.
- [x] Fixes found by the matrix: `file_uri()` (LibreOffice profile URI was malformed on Windows -> all soffice conversions failed, one hung 750s); `-y` dropped from zip args on Windows (GnuWin32 lacks it; `--` verified supported).
- [x] Tests: full suite 122/122 with real tools, 0 warnings. Finding: `miniatura...nombres_raros` short-circuited while magick was absent and used NTFS-illegal names (`:`, `*`) - now platform-filtered.
- [x] Probes 1-3: HEIC read-only parity (libheif 1.23.6), Rar/Rar5 handlers in 7z, SVG falls back to ImageMagick when rsvg-convert absent (thumbnail degrades silently only).
- [x] RESOLVED in M5: descendant sweep proven by `spawn::job_tests::kill_tree_sweeps_the_whole_job` (grandchild heartbeat must freeze after kill_tree, direct child reaped within 5s; suite now 123/123). The earlier probe "failures" were rig bugs: cmd's `start /b` mangles quoted `%~1` args, and a shell-level crash test cannot drive the app's click-to-convert UI.
- [ ] Ghostscript-dependent PDF-compression tools remain unverified (no winget package; official installer is the open sourcing decision).

### M3 - Wheel-at-cursor UX - DONE (commit abad0b3, pushed)
- [x] `cursor_page_pos` command + `show_wheel(expand)` flag: native cursor query replaces the dance on Windows; Wayland/foreign-monitor errors fall back to the original enlarge-capture-shrink path bit-for-bit. Frontend places while hidden (no flash).
- [x] Live-verified at this machine's native 150% DPI: window center == cursor, offset (0,0); wheel screenshot shows full render (segments, thumbnail, size pill); free-mode "Drop a file here" card verified. Caveat: 100%/125% DPI not separately tested (same code path, logical sizes throughout).
- [x] pt/fr catalogs extended for the 4 new strings (completeness test green).

### M4 - Shell integration - DONE (commit c901462, pushed)
- [x] `packaging/register-explorer.ps1`: HKCU verbs for files (`*`), folder icons (`Directory`) and folder background (`%V`); verified registered; folder-arg launch renders the wheel. Multi-selection = one invocation per file (documented difference vs Nautilus).
- [x] Alt+M global shortcut (tauri-plugin-global-shortcut 2): toggle hide/show, summons at cursor on any monitor; live-verified: visible -> hidden -> visible at (1200,700) offset (0,0). Wayland registration failure degrades to a warning.
- [x] NSIS bundle target configured; installer produced by the M5 release build.
- [ ] Decision still open (by design): flip internal names (citrus.exe, com.citrus.converter, CITRUS_CONFIG_DIR) after upstream merges settle.

### M5 - Verification & close-out - DONE
- [x] Release build: `mandarina.exe` 4,976,640 B, SHA-256 `1b4dc2b9...a968cc88` (LTO; rustc 1.99.1 matches the compiler commit inferred from the upstream `.deb`).
- [x] NSIS installer `Citrus_0.1.0_x64-setup.exe` 1.54 MiB, SHA-256 `5a0e6ffb...b248f1`. Needed manual NSIS 3.13 staging into `%LOCALAPPDATA%\tauri\NSIS` (CLI's GitHub download timed out on this network) plus `nsis_tauri_utils-v0.5.3` dll.
- [x] Silent-install smoke test: `/S` installs to `%LOCALAPPDATA%\Citrus\`; installed exe launched with a file arg, window "Citrus" alive at t+3s.
- [x] Visual verification: wheel renders fully at cursor (m3-wheel.png: 7 segments, thumbnail, size pill); free-mode drop card (m3-freemode.png). Environment caveat: in this agent session app windows get reparented into a tabbed host, so HWND-geometry assertions are unreliable here; the offset (0,0) checks passed before hosting kicked in, screenshots are the ground truth.
- [x] Behavior comparison: the matrix exercises the identical `convert_blocking` code path with the same external tools; `docs/Converters.md` reproduced 16/16 on Windows. A physical Linux side-by-side run remains optional.
- [x] README rewritten for Citrus (MIT attribution, upstream link, requirements table, documented differences); PROGRESS.md committed per milestone.

## Known unknowns / risks
- Binary-to-source build equivalence: not established (no decompile needed; bounded by choice).
- WebView2 transparency edges at non-100% DPI (probe in M3).
- Multi-monitor mixed-DPI cursor placement (Wayland code only covered primary monitor; verify on Windows).

## Evidence quick-reference
- Release deb SHA-256: `44262a5516d595f3e2aa316dbed93d992d0a6f75c447607da95f193a97515541`
- Shipped ELF SHA-256: `fb2104abdfbd76d361a1775ef4c6b210b4949e382dfa0b16ee6876e0f32dd923` (6,550,896 B, x86_64)
- REA evidence ID: `ev_918f674d7a25951e721c4cf6b645c05a59291b9fba9ea2905295d11c31162404`
- Analyzed tags/commits: tag `v0.1.0-beta` = `628ac7f5`; port base = main `f2607ea`; rename = `849491f`
- Commit chain: rename `849491f` -> M1 `b5eb924` -> M2 `76a0b89` -> M3 `abad0b3` -> M4 `c901462` -> M5 (close-out)
- Windows release exe SHA-256: `1b4dc2b92da811c55e1d93cc7c397a6df696453288c92970f2ca8b17a968cc88` (4,976,640 B)
- NSIS installer SHA-256: `5a0e6ffbf5b8786d9d78f3517ddcc4c3efbbf224dc07d4a7550227f864b248f1` (Citrus_0.1.0_x64-setup.exe, 1.54 MiB)
