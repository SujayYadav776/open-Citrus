# Citrus

**A fast, private file converter for Windows.** Pick a file, choose a format from a
wheel that pops up right at your cursor, done. Everything happens on your own computer —
nothing is ever uploaded.

![Citrus demo](docs/images/mandarina-demo.gif)

- 🖱️ Convert straight from the right‑click menu, or press **Alt+M** and drop a file
- 🎨 Images · 🎵 Audio · 🎬 Video · 📄 PDF · 📝 Documents · 📊 Spreadsheets · 🗜️ Archives
- 🔒 100% local and offline — no account, no cloud, no tracking
- 🛠️ Extras: compress, trim, rotate, flip, resize, strip metadata, remove audio

---

## Quick start (3 steps)

### 1. Install the app

Download the latest **`Citrus_x.x.x_x64-setup.exe`** from the
[Releases](https://github.com/SujayYadav776/open-Citrus/releases) page and run it. It
installs for your user only — no administrator rights needed. When it's done, Citrus
lives quietly in your system tray.

### 2. Install the converter tools (one click)

Citrus does the converting through well‑known free command‑line tools. On first run it
shows a small banner listing whatever is missing, with an **Install all** button that
sets them up for you automatically (you'll see per‑tool progress, and you can minimize
it while it works). You can also open **Settings → Converter tools** at any time.

> Prefer to do it yourself, or need a tool the in‑app installer doesn't cover? See the
> [full tool list](#the-tools-behind-it) below.

### 3. Convert something

Two ways, whichever you like:

- **Right‑click** a file in Explorer → **Convert with Citrus**, then click a format on
  the wheel. (Add the menu item once with the command in
  [Shell integration](#shell-integration-right-click-menu).)
- **Press Alt+M** to open the drop card at your cursor, drag a file onto it, and pick a
  format.

The converted file appears next to the original with the new extension. If a name is
taken, Citrus adds "2", "3", … — it never overwrites your files.

![The conversion wheel](docs/images/wheel.png)

---

## What can it convert?

Ten kinds and 250+ conversions, all local: images, audio, video, PDF, documents,
presentations, spreadsheets, subtitles, archives and folders — plus the editing tools
above. The full format matrix lives in
[docs/Converters.md](docs/Converters.md).

![Tools wheel](docs/images/tools-wheel.png)

---

## Shell integration (right‑click menu)

To add "Convert with Citrus" to the Explorer right‑click menu, run this once in PowerShell
(it only writes to your own user settings — no admin, and easy to undo):

```powershell
powershell -File packaging\register-explorer.ps1          # add the menu items
powershell -File packaging\register-explorer.ps1 -Remove  # undo
```

If you installed Citrus somewhere unusual, point the menu at your `citrus.exe` first:

```powershell
$env:CITRUS_BIN = "C:\Users\you\AppData\Local\Citrus\citrus.exe"
```

If the menu doesn't appear immediately, restart Explorer (or sign out and back in).
**Alt+M works without any of this** — it just needs Citrus to be running.

---

## The tools behind it

Citrus detects what's installed and only offers what actually works; when a tool is
missing it names the exact `winget` package to get. The in‑app installer covers the
first six; the last few are manual.

| Tool | Needed for | How to get it |
|---|---|---|
| ffmpeg | audio, video, subtitles, probing | in‑app installer / `winget install Gyan.FFmpeg` |
| ImageMagick | images, PDF from images | in‑app installer / `winget install ImageMagick.ImageMagick` |
| pandoc | markdown / HTML / epub | in‑app installer / `winget install JohnMacFarlane.Pandoc` |
| LibreOffice | docx / pdf / pptx / xlsx | in‑app installer / `winget install TheDocumentFoundation.LibreOffice` |
| 7‑Zip | rar input, zip fallback | in‑app installer / `winget install 7zip.7zip` |
| poppler | PDF to text / images | in‑app installer / `winget install oschwartz10612.Poppler` |
| Ghostscript | PDF "reduce size" tools | manual: [official installer](https://ghostscript.com/releases/gsdnld.html) |
| zip / unzip (Info‑ZIP) | archive convert / create | manual (`winget install GnuWin32.Zip`, `GnuWin32.UnZip`) |
| FLAC | flac audio | manual (`winget install Xiph.FLAC`) |

> **Note:** the 7‑Zip and LibreOffice installers don't add themselves to your PATH. If
> Citrus can't find them, add `C:\Program Files\7-Zip` and
> `C:\Program Files\LibreOffice\program` to your PATH.

---

## Troubleshooting

- **The wheel looks empty / a format is missing** → that tool isn't installed. Open
  **Settings → Converter tools** and install it.
- **Right‑click menu doesn't show** → run the registration command above, then restart
  Explorer.
- **Alt+M does nothing** → make sure Citrus is running (look for its icon in the tray).
- **A conversion fails** → the card tells you which tool is missing and how to get it.

---

## For developers

Needs Rust (MSVC toolchain) and Node 24+, plus the tools above. From `src-tauri/`:

```
cargo test          # unit + integration tests (1 ignored: a manual real-file probe)
cargo tauri dev     # hot reload
cargo tauri build   # release exe + NSIS installer (target: nsis)
```

The app is a Tauri 2 desktop app: a Rust backend (`src-tauri/`) and a plain
HTML/CSS/JS frontend (`src/`, no framework, no bundler). The design notes, per‑module
docs and the milestone progress ledger live in [`docs/`](docs/) and `PROGRESS.md`.

---

## Differences from the Linux original

Citrus is a Windows port of the Linux app. A few things work differently here:

- **Alt+M** opens a free "drop a file here" card instead of converting a file‑manager
  selection (Explorer has no global selection API — the right‑click menu is the path).
- A **multi‑selection** in Explorer opens one wheel per file, not a combined one.
- **SVG** thumbnails fall back to ImageMagick when `rsvg-convert` isn't present.
- **HEIC** is read‑only and **AVIF** reads/writes, matching the upstream ImageMagick build.

---

## About & license

Citrus is the Windows port of [Mandarina](https://github.com/JorgeFTS/Mandarina), an MIT‑licensed
project by JorgeFTS; the Linux app remains the source of truth for behavior, and the full
upstream history is preserved in this repository's git log. This fork uses its own Citrus
identity (binary `citrus.exe`, config folder `citrus`, bundle id `com.citrus.converter`).

MIT licensed. Mandarina is Copyright (c) 2026 JorgeFTS; the Citrus port and branding are
Copyright (c) 2026 SujayYadav776 — both under the same MIT license, with the original
notice retained as MIT requires. See [LICENSE](LICENSE), [NOTICE](NOTICE) and
[AUTHORS](AUTHORS). Citrus is an independent derivative and is not endorsed by or
affiliated with the original author.
