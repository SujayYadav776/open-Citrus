# Citrus

A local file converter with a floating radial wheel, for **Windows**. Select files in
Explorer, right click "Convert with Citrus" (or press Alt+M and drop a file), and pick a
format from the wheel that appears at your cursor. Everything runs on your machine;
nothing is uploaded.

Citrus is the Windows port of [Mandarina](https://github.com/JorgeFTS/Mandarina), an MIT
licensed project by JorgeFTS. The full upstream history is preserved in this repository's
git log, and the Linux app remains the source of truth for behavior. This fork uses a full
Citrus identity: the binary is `citrus.exe`, the config folder is `citrus`, and the bundle
id is `com.citrus.converter`. Upstream attribution is kept via the `repository` field and
the MIT `LICENSE` copyright; the internal-name flip means future upstream merges must be
replayed onto the Citrus names rather than applied verbatim.

## What it converts

Ten kinds and 250+ conversions, all local: images, audio, video, PDF, documents,
presentations, spreadsheets, subtitles, archives, and folders, plus tools (compress,
strip metadata, trim, rotate, flip, resize, remove audio). The conversion recipes and
the format matrix are inherited from upstream; see
[docs/Converters.md](docs/Converters.md).

## Requirements

The app itself needs no installer privileges, but conversions are performed by external
command line tools that must be on your PATH. Citrus detects what is installed and only
offers what works (hints name the winget package when a tool is missing):

| Tool | Needed for | winget id |
|---|---|---|
| ffmpeg | audio, video, subtitles, probes | `Gyan.FFmpeg` |
| ImageMagick | images, PDF from images | `ImageMagick.ImageMagick` |
| pandoc | markdown/HTML/epub document paths | `JohnMacFarlane.Pandoc` |
| LibreOffice | docx/pdf/pptx/xlsx conversions | `TheDocumentFoundation.LibreOffice` |
| poppler (pdftoppm, pdftotext) | PDF to text/images | `oschwartz10612.Poppler` |
| 7-Zip | rar input, zip fallback | `7zip.7zip` |
| zip / unzip (Info-ZIP) | archive convert/create | `GnuWin32.Zip`, `GnuWin32.UnZip` |
| FLAC | flac audio | `Xiph.FLAC` |
| Ghostscript | PDF size-reduction tools only | not on winget (official installer) |

Note: the 7-Zip and LibreOffice installers do not add themselves to PATH; if the app
does not find them, add `C:\Program Files\7-Zip` and `C:\Program Files\LibreOffice\program`
to your PATH.

## Install

Grab the NSIS installer from the latest release, or run the portable exe from
`src-tauri/target/release/`. Then wire the shell integration:

```powershell
powershell -File packaging\register-explorer.ps1          # add context menu verbs (HKCU)
powershell -File packaging\register-explorer.ps1 -Remove  # undo
```

Set `CITRUS_BIN` to point the verbs at your installed `citrus.exe` path. Alt+M works
while the app is running (it summons or hides the drop card at the cursor; drag files
onto it).

## Differences from the Linux original

- Alt+M summons the free "drop a file here" card instead of converting a file manager
  selection (Explorer gives no global selection API; the context menu is the selection
  path).
- Multi-selection in Explorer invokes the verb once per file, so each selection opens
  its own wheel; the Linux script received them together.
- SVG center thumbnails fall back to ImageMagick's renderer when `rsvg-convert` is
  absent (it has no winget package); complex SVGs may render less faithfully.
- HEIC is input only and AVIF is read/write, matching the upstream ImageMagick build.
- The wheel appears exactly at the cursor via a native pointer query; the Wayland
  enlargement workaround is kept as fallback and remains the Linux path.

## Development

Rust stable (MSVC), Node 24+, and the tool table above. From `src-tauri/`:

```
cargo test          # 127 pass + 1 ignored (archivo_real is a manual probe)
cargo tauri dev     # hot reload
cargo tauri build   # release exe + NSIS installer (target: nsis)
```

The port was planned and verified milestone by milestone; the progress ledger lives at
`PROGRESS.md` and the reverse-engineering findings behind it in `docs/` (upstream) plus
the commit history.

## License

MIT. The underlying Mandarina work is Copyright (c) 2026 JorgeFTS; the Citrus Windows
port and branding are Copyright (c) 2026 SujayYadav776. Both are distributed under the
same MIT license, and the original copyright and permission notice are retained as MIT
requires. See [LICENSE](LICENSE), [NOTICE](NOTICE), and [AUTHORS](AUTHORS) for the full
attribution terms. Citrus is an independent derivative and is not endorsed by or
affiliated with the original author.
