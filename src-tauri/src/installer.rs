// Detecting and installing the external command-line tools Citrus shells out to.
//
// Citrus is only a driver: the actual conversions are done by ffmpeg, ImageMagick, pandoc,
// LibreOffice, 7-Zip, poppler and ghostscript. On a fresh machine those are missing, so this
// module powers the "Tools" section of the settings and the first-run banner: it reports which
// tools are present and, on Windows, installs the missing ones silently through `winget`,
// streaming progress and allowing the in-flight installer to be stopped.
//
// A tool counts as present when any of its program names resolves (PATH first, then the known
// install directories - see spawn::find_program), which is the same rule the converters use.
use crate::spawn::find_program;
use crate::tr;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tauri::{AppHandle, Emitter, Manager};

// One external tool: the programs that count as it, its winget package id, and whether it is
// essential (without it even the most common conversions - images, audio, video - do not work).
pub struct ToolSpec {
    key: &'static str,
    // apt package names identifying this tool. The converters ask for a missing tool by its
    // Debian name, and this is how that name finds its entry here (see hint_for_apt). Tools
    // that come with the system (zip, tar, unzip) are not listed: there is nothing to offer.
    apt_names: &'static [&'static str],
    // winget package id, or None when the tool is not distributed through winget
    winget: Option<&'static str>,
    // official download page, for tools that must be installed manually
    manual_url: Option<&'static str>,
    programs: &'static [&'static str],
    essential: bool,
    // machine-wide MSI: winget will request elevation (UAC) and it can take several minutes
    needs_admin: bool,
    // per-package watchdog budget; if exceeded the in-flight installer is stopped and reported
    timeout_secs: u64,
}

// Order matters: the essentials first, then the ones that unlock extra formats.
const TOOLS: [ToolSpec; 8] = [
    ToolSpec { key: "ffmpeg", apt_names: &["ffmpeg"], winget: Some("Gyan.FFmpeg"), manual_url: None, programs: &["ffmpeg", "ffprobe"], essential: true, needs_admin: false, timeout_secs: 900 },
    ToolSpec { key: "imagemagick", apt_names: &["imagemagick"], winget: Some("ImageMagick.ImageMagick"), manual_url: None, programs: &["magick", "convert"], essential: true, needs_admin: true, timeout_secs: 1800 },
    ToolSpec { key: "pandoc", apt_names: &["pandoc"], winget: Some("JohnMacFarlane.Pandoc"), manual_url: None, programs: &["pandoc"], essential: false, needs_admin: true, timeout_secs: 1800 },
    ToolSpec { key: "libreoffice", apt_names: &["libreoffice"], winget: Some("TheDocumentFoundation.LibreOffice"), manual_url: None, programs: &["soffice", "libreoffice"], essential: false, needs_admin: true, timeout_secs: 1800 },
    ToolSpec { key: "sevenzip", apt_names: &["7zip", "p7zip-full"], winget: Some("7zip.7zip"), manual_url: None, programs: &["7z", "7za", "7zz"], essential: false, needs_admin: true, timeout_secs: 1800 },
    ToolSpec { key: "poppler", apt_names: &["poppler-utils"], winget: Some("oschwartz10612.Poppler"), manual_url: None, programs: &["pdftoppm", "pdftotext"], essential: false, needs_admin: false, timeout_secs: 900 },
    // Ghostscript is not on winget; it must be installed from the official page.
    ToolSpec { key: "ghostscript", apt_names: &["ghostscript"], winget: None, manual_url: Some("https://ghostscript.com/releases/gsdnld.html"), programs: &["gs", "gswin64c"], essential: false, needs_admin: false, timeout_secs: 900 },
    // MiKTeX compiles LaTeX (.tex/.latex) to PDF; the net installer downloads packages on demand.
    ToolSpec { key: "miktex", apt_names: &["miktex"], winget: Some("MiKTeX.MiKTeX"), manual_url: None, programs: &["pdflatex", "xelatex", "texify"], essential: false, needs_admin: false, timeout_secs: 1800 },
];

// The install instruction for a missing tool, taken from the table above: the winget command
// when the tool is distributed through winget, the official download page when it is not, and a
// generic PATH message for the plain system tools this list does not cover. Kept here so the
// message and the installer can never disagree, which is exactly what had drifted for poppler
// (no arm at all), 7-Zip (an id that does not exist) and ghostscript (not on winget).
pub(crate) fn hint_for_apt(apt_names: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut unknown: Vec<&str> = Vec::new();
    for name in apt_names.split_whitespace() {
        match TOOLS.iter().find(|t| t.apt_names.contains(&name)) {
            Some(t) => {
                let one = match (t.winget, t.manual_url) {
                    (Some(id), _) => format!("winget install {id}"),
                    (None, Some(url)) => url.to_string(),
                    (None, None) => name.to_string(),
                };
                if !parts.contains(&one) {
                    parts.push(one);
                }
            }
            None => unknown.push(name),
        }
    }
    if !unknown.is_empty() {
        parts.push(format!("install {} and put it on your PATH", unknown.join(", ")));
    }
    parts.join(" ; or ")
}

// Which tool a program name belongs to, using the same rule as detection: any of its program
// names counts. The converters need this to put the code of the tool that is actually missing in
// front of the error, so the interface can offer its one-click install.
pub(crate) fn tool_key_for_program(program: &str) -> Option<&'static str> {
    TOOLS.iter().find(|t| t.programs.contains(&program)).map(|t| t.key)
}

#[derive(Serialize)]
pub struct ToolStatus {
    key: &'static str,
    winget: Option<&'static str>,
    manual_url: Option<&'static str>,
    present: bool,
    essential: bool,
    needs_admin: bool,
}

// The payload streamed to the interface while installing: which tool, what phase, and a line of
// installer output (for "log"). index/total let the interface show "2 of 5".
#[derive(Clone, Serialize)]
struct InstallEvent {
    key: String,
    phase: String,
    line: String,
    index: usize,
    total: usize,
}

fn present(programs: &[&str]) -> bool {
    programs.iter().any(|p| find_program(p).is_some())
}

// Whether the tool of this key is resolvable, with the same rule as detection: any of its program
// names counts. converters::missing_tools() asks this instead of reading the table itself. An
// unknown key is not present, so a typo can never report a tool as missing.
pub(crate) fn is_present(key: &str) -> bool {
    TOOLS.iter().find(|t| t.key == key).is_some_and(|t| present(t.programs))
}

// The program names this tool answers to. Anything that runs a tool asks for this list instead of
// repeating one name: a Windows Ghostscript install ships gswin64c.exe and no gs.exe, so the PDF
// compressor, hard-coded to "gs", failed on machines where this same table reported the tool
// installed. An unknown key gives an empty list, which resolves to nothing, so a typo can never
// run a program by accident.
pub(crate) fn programs_for(key: &str) -> &'static [&'static str] {
    TOOLS.iter().find(|t| t.key == key).map(|t| t.programs).unwrap_or(&[])
}

#[tauri::command]
pub fn get_tool_status() -> Vec<ToolStatus> {
    TOOLS
        .iter()
        .map(|t| ToolStatus { key: t.key, winget: t.winget, manual_url: t.manual_url, present: present(t.programs), essential: t.essential, needs_admin: t.needs_admin })
        .collect()
}

// Cancel request for the running batch, and the pid of the winget process currently in flight (so
// "Stop" can terminate just that process, letting any installer it already launched finish).
static INSTALL_CANCEL: AtomicBool = AtomicBool::new(false);
static INSTALL_PID: AtomicU32 = AtomicU32::new(0);
// One install at a time: the atomics above are process-global, so a second concurrent batch would
// clobber the cancel flag and the tracked pid. INSTALL_RUNNING serialises them.
static INSTALL_RUNNING: AtomicBool = AtomicBool::new(false);

// Clears INSTALL_RUNNING when it goes out of scope, on any return path (including a panic unwind).
struct InstallGuard;
impl Drop for InstallGuard {
    fn drop(&mut self) {
        INSTALL_RUNNING.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
pub fn cancel_install() {
    INSTALL_CANCEL.store(true, Ordering::SeqCst);
    let pid = INSTALL_PID.load(Ordering::SeqCst);
    if pid != 0 {
        kill_pid(pid);
    }
}

// Kills a single process by pid (no /T: child installers already started are left to finish).
#[cfg(windows)]
fn kill_pid(pid: u32) {
    use std::process::{Command, Stdio};
    let mut command = Command::new("taskkill");
    command.args(["/F", "/PID", &pid.to_string()]).stdout(Stdio::null()).stderr(Stdio::null());
    crate::spawn::hide_console(&mut command);
    let _ = command.status();
}

#[cfg(not(windows))]
fn kill_pid(_pid: u32) {}

#[tauri::command]
pub async fn install_tools(app: AppHandle, keys: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || install_blocking(app, keys))
        .await
        .map_err(|e| tr!("The installer stopped unexpectedly: {error}", "El instalador se detuvo inesperadamente: {error}", error = e))?
}

fn install_blocking(app: AppHandle, keys: Vec<String>) -> Result<(), String> {
    // One batch at a time: the cancel/pid atomics are process-global, so a second concurrent
    // install would corrupt the first. Claim the slot; if it is taken, reject rather than interleave.
    if INSTALL_RUNNING.swap(true, Ordering::SeqCst) {
        return Err(tr!("An installation is already running", "Ya hay una instalación en curso"));
    }
    let _guard = InstallGuard;
    #[cfg(not(windows))]
    {
        let _ = (app, keys);
        Err(tr!("Automatic install is only available on Windows", "La instalación automática solo está disponible en Windows"))
    }
    #[cfg(windows)]
    {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        use std::sync::Arc;
        use std::time::{Duration, Instant};

        let winget = find_program("winget").ok_or_else(|| tr!("winget is not available on this system", "winget no está disponible en este sistema"))?;
        let specs: Vec<&ToolSpec> = TOOLS.iter().filter(|t| t.winget.is_some() && keys.iter().any(|k| k == t.key)).collect();
        let total = specs.len();
        INSTALL_CANCEL.store(false, Ordering::SeqCst);
        INSTALL_PID.store(0, Ordering::SeqCst);

        for (index, spec) in specs.into_iter().enumerate() {
            if INSTALL_CANCEL.load(Ordering::SeqCst) {
                break;
            }
            // Defense in depth: the interface filters to missing tools and re-checks right before
            // installing, but a stale key could still reach here. If any of this tool's programs now
            // resolves, there is nothing to install - report it as done and skip the winget call.
            if present(spec.programs) {
                emit(&app, spec.key, "done", "", index, total);
                continue;
            }
            emit(&app, spec.key, "start", "", index, total);

            let timeout = spec.timeout_secs;
            let mut command = Command::new(&winget);
            command
                .args(["install", "--id", spec.winget.unwrap(), "--exact", "--silent", "--accept-package-agreements", "--accept-source-agreements"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            crate::spawn::hide_console(&mut command);
            let mut child = match command.spawn() {
                Ok(c) => c,
                Err(e) => {
                    emit(&app, spec.key, "error", &tr!("Could not start the installer: {error}", "No se pudo iniciar el instalador: {error}", error = e), index, total);
                    continue;
                }
            };
            let pid = child.id();
            INSTALL_PID.store(pid, Ordering::SeqCst);

            // Watchdog: if this package runs past its budget, stop the in-flight winget so the
            // batch cannot hang forever on a stalled download or an unanswered elevation prompt.
            let finished = Arc::new(AtomicBool::new(false));
            let timed_out = Arc::new(AtomicBool::new(false));
            {
                let (finished, timed_out) = (Arc::clone(&finished), Arc::clone(&timed_out));
                std::thread::spawn(move || {
                    let deadline = Instant::now() + Duration::from_secs(timeout);
                    while !finished.load(Ordering::SeqCst) && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(500));
                    }
                    if !finished.load(Ordering::SeqCst) {
                        timed_out.store(true, Ordering::SeqCst);
                        kill_pid(pid);
                    }
                });
            }

            // winget writes progress to stdout and occasional warnings to stderr; drain both at the
            // same time (stderr on its own thread) so neither pipe can fill and block the child.
            let err_thread = child.stderr.take().map(|err| {
                let app = app.clone();
                let key = spec.key.to_string();
                std::thread::spawn(move || {
                    for line in BufReader::new(err).lines().map_while(Result::ok) {
                        let line = line.trim();
                        if !line.is_empty() {
                            emit(&app, &key, "log", line, index, total);
                        }
                    }
                })
            });
            if let Some(out) = child.stdout.take() {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    let line = line.trim();
                    if !line.is_empty() {
                        emit(&app, spec.key, "log", line, index, total);
                    }
                    if INSTALL_CANCEL.load(Ordering::SeqCst) {
                        kill_pid(pid);
                        break;
                    }
                }
            }
            let status = child.wait();
            finished.store(true, Ordering::SeqCst);
            if let Some(t) = err_thread {
                let _ = t.join();
            }
            INSTALL_PID.store(0, Ordering::SeqCst);

            let ok = matches!(&status, Ok(s) if s.success());
            if INSTALL_CANCEL.load(Ordering::SeqCst) {
                emit(&app, spec.key, "cancelled", "", index, total);
                break;
            }
            if timed_out.load(Ordering::SeqCst) && !ok {
                emit(&app, spec.key, "timeout", "", index, total);
                continue;
            }
            emit(&app, spec.key, if ok { "done" } else { "error" }, "", index, total);
        }
        Ok(())
    }
}

fn emit(app: &AppHandle, key: &str, phase: &str, line: &str, index: usize, total: usize) {
    let _ = app.emit(
        "install-progress",
        InstallEvent { key: key.to_string(), phase: phase.to_string(), line: line.to_string(), index, total },
    );
}

// Minimizes the main window. Used by the installer's "Minimize" button so a multi-minute
// winget install can be tucked into the taskbar while it keeps running in the background.
#[tauri::command]
pub fn minimize_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

// While installing, turn the borderless always-on-top card into a normal, minimizable window
// that shows in the taskbar (so the user can minimize/restore it). When off, restore the card.
#[tauri::command]
pub fn set_install_taskbar(app: AppHandle, on: bool) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_skip_taskbar(!on);
        let _ = window.set_always_on_top(!on);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_lista_de_herramientas_es_coherente() {
        assert_eq!(get_tool_status().len(), TOOLS.len());
        // every tool has at least one program to look for and a way to get it (winget or manual)
        for t in &TOOLS {
            assert!(t.winget.is_some() || t.manual_url.is_some(), "{} has no install path", t.key);
            assert!(!t.programs.is_empty(), "{}", t.key);
        }
        // keys are unique and the two essentials come first (ffmpeg, imagemagick)
        let mut keys: Vec<&str> = TOOLS.iter().map(|t| t.key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n, "duplicated tool key");
        assert!(TOOLS.iter().filter(|t| t.essential).count() >= 2);
    }

    // converters::tools_for_kind() is the coarse "what may a file of this kind need" table behind
    // the wheel warning. It is allowed to be imprecise, it is not allowed to name a tool that does
    // not exist here, and no kind may end up claiming that it needs nothing.
    #[test]
    fn las_herramientas_por_tipo_de_archivo_son_reales() {
        use crate::converters::{tools_for_kind, Kind};
        const TODOS: [Kind; 10] = [
            Kind::Image, Kind::Audio, Kind::Video, Kind::Pdf, Kind::Document,
            Kind::Slides, Kind::Spreadsheet, Kind::Subtitle, Kind::Archive, Kind::Folder,
        ];
        for kind in TODOS {
            let herramientas = tools_for_kind(kind);
            assert!(!herramientas.is_empty(), "{kind:?} says it needs no tool");
            for key in herramientas {
                let spec = TOOLS.iter().find(|t| t.key == *key)
                    .unwrap_or_else(|| panic!("{kind:?} names an unknown tool: {key}"));
                assert!(
                    spec.winget.is_some() || spec.manual_url.is_some(),
                    "{kind:?} names {key}, which has no install path"
                );
                assert!(!spec.programs.is_empty(), "{key} has no program names");
            }
        }
    }

    #[test]
    fn la_pista_de_instalacion_sale_de_la_lista() {
        // poppler: the old hand-written hint had no arm for it, so a user was told to put
        // "poppler-utils" on their PATH while the app knew the winget package all along
        assert_eq!(hint_for_apt("poppler-utils"), "winget install oschwartz10612.Poppler");
        // 7-Zip: the old hint used `OpenSource.7-Zip`, which is not a package on winget
        assert_eq!(hint_for_apt("7zip"), "winget install 7zip.7zip");
        assert_eq!(hint_for_apt("p7zip-full"), "winget install 7zip.7zip");
        // ghostscript is genuinely not on winget, so it must never be offered as a winget command
        let gs = hint_for_apt("ghostscript");
        assert!(!gs.contains("winget"), "ghostscript is not on winget: {gs}");
        assert!(gs.contains("ghostscript.com"), "{gs}");
        // tools the table does not cover keep the generic message
        assert_eq!(hint_for_apt("librsvg2-bin"), "install librsvg2-bin and put it on your PATH");
        // a mixed list offers both, without repeating anything
        let mix = hint_for_apt("unzip 7zip");
        assert!(mix.contains("winget install 7zip.7zip") && mix.contains("unzip"), "{mix}");
        assert_eq!(hint_for_apt("ffmpeg ffmpeg"), "winget install Gyan.FFmpeg");
    }

    // The program lists are what several callers rely on: compress_pdf(), the settings and the
    // test guards. Pinning the names here is what stops a list being narrowed back to one entry
    // and silently breaking a platform again.
    #[test]
    fn los_nombres_de_cada_herramienta_se_mantienen() {
        // Windows ships gswin64c.exe and no gs.exe; with only "gs" in the list the PDF compressor
        // failed there while this same table reported the tool installed.
        let gs = programs_for("ghostscript");
        assert!(gs.contains(&"gswin64c"), "windows name missing: {gs:?}");
        assert_eq!(gs.first().copied(), Some("gs"), "the unix name must stay first");
        // the names the converters pass to need() have to be in the lists that detect the tool
        assert!(programs_for("imagemagick").contains(&"convert"));
        assert!(programs_for("imagemagick").contains(&"magick"));
        assert!(programs_for("libreoffice").contains(&"soffice"));
        assert!(programs_for("sevenzip").contains(&"7z"));
        assert!(programs_for("poppler").contains(&"pdftoppm"));
        // a key that is not in the table resolves to nothing, never to some other program
        assert!(programs_for("no-existe").is_empty());
        assert!(!is_present("no-existe"));
    }

    // Every entry must be able to recommend itself, and no apt name may belong to two tools:
    // that is what keeps this table the only place where the install guidance lives.
    #[test]
    fn cada_herramienta_recomienda_su_propio_instalador() {
        let mut todas_las_apt: Vec<&str> = Vec::new();
        for t in &TOOLS {
            assert!(!t.apt_names.is_empty(), "{} has no apt name to match a hint", t.key);
            for apt in t.apt_names {
                let h = hint_for_apt(apt);
                match (t.winget, t.manual_url) {
                    (Some(id), _) => assert_eq!(h, format!("winget install {id}"), "apt {apt}"),
                    (None, Some(url)) => assert_eq!(h, url.to_string(), "apt {apt}"),
                    (None, None) => panic!("{} has no install path", t.key),
                }
                assert!(!todas_las_apt.contains(apt), "apt name {apt} used by two tools");
                todas_las_apt.push(apt);
            }
        }
    }
}
