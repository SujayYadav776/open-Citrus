// Spawning external tools in a unit that can be killed together with everything
// the tool itself launches.
//
// - Unix (as upstream): the child gets its own process group (`process_group(0)`) and
//   killing sends SIGKILL to the whole group.
// - Windows: the child is created suspended and assigned to a private Job Object with
//   KILL_ON_JOB_CLOSE before its first thread runs, so no descendant can escape the job
//   (children inherit it and break-away is not allowed). Killing the tree is one call,
//   and closing the handle by accident (the parent dying) also takes the tree down.
use std::ops::{Deref, DerefMut};
use std::process::{Child, Command};

// Windows process creation flags. Citrus is a GUI (windows-subsystem) app, so it has no console
// to hand down; launching a console tool (ffmpeg, ffprobe, 7z, unzip, pandoc, magick...) without
// CREATE_NO_WINDOW makes Windows allocate a fresh console for it, which flashes a black box on
// screen even when its stdio is redirected to a pipe. Suppressing the window is cosmetic only:
// the tools still run and still write to the pipes we read progress from.
#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(windows)]
const CREATE_SUSPENDED: u32 = 0x0000_0004; // lets us place the child in the Job Object before it runs

// Suppresses the console window for a child launched directly (e.g. via `.output()`). Callers that
// go through `spawn_killing_tree` get this for free. No-op on Unix, which has no consoles to allocate.
pub(crate) fn hide_console(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = command;
    }
}

pub(crate) struct Proc {
    child: Child,
    #[cfg(windows)]
    _job: JobHandle,
}

impl Deref for Proc {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.child
    }
}

impl DerefMut for Proc {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}

pub(crate) fn spawn_killing_tree(mut command: Command) -> Result<Proc, std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
        Ok(Proc { child: command.spawn()? })
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_SUSPENDED | CREATE_NO_WINDOW);
        let child = command.spawn()?;
        // Degraded case: if the job cannot be set up we resume the process anyway and
        // fall back to killing just the direct child.
        match unsafe { win::attach_job_and_resume(&child) } {
            Ok(job) => Ok(Proc { child, _job: job }),
            Err(e) => {
                if std::env::var_os("CITRUS_DEBUG").is_some() {
                    eprintln!("[citrus] job attach failed, kill degrades to direct child: {e:?}");
                }
                win::resume_all_threads(&child);
                Ok(Proc { child, _job: JobHandle::none() })
            }
        }
    }
}

// Kills everything the child launched. On Windows this terminates the job (all its
// processes at once); the handle itself is swept by KILL_ON_JOB_CLOSE when the Proc drops.
pub(crate) fn kill_tree(proc: &Proc) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", proc.child.id())])
            .stderr(std::process::Stdio::null())
            .status();
    }
    #[cfg(windows)]
    {
        proc._job.terminate();
    }
}

#[cfg(windows)]
mod win {
    use super::Child;
    use std::os::windows::io::AsRawHandle;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject, TerminateJobObject,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation,
    };
    use windows::Win32::System::Threading::{
        OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };
    use windows::Win32::Globalization::GetUserDefaultLocaleName;

    // A job handle that closes itself; KILL_ON_JOB_CLOSE sweeps whatever is still running.
    pub struct JobHandle(Option<HANDLE>);

    impl JobHandle {
        pub fn none() -> Self {
            JobHandle(None)
        }
        pub fn terminate(&self) {
            if let Some(job) = self.0 {
                unsafe {
                    let _ = TerminateJobObject(job, 1);
                }
            }
        }
    }

    impl Drop for JobHandle {
        fn drop(&mut self) {
            if let Some(job) = self.0 {
                unsafe {
                    let _ = CloseHandle(job);
                }
            }
        }
    }

    // Safety: the handle is a plain kernel object owned by this JobHandle and only used
    // from the thread that owns the Proc (never sent across threads).
    unsafe impl Send for JobHandle {}

    pub(crate) unsafe fn attach_job_and_resume(child: &Child) -> Result<JobHandle, windows::core::Error> {
        let process: HANDLE = HANDLE(child.as_raw_handle());
        let job = unsafe { CreateJobObjectW(None, PCWSTR::null()) }?;
        let created = JobHandle(Some(job));
        // Any failure below: the JobHandle drop closes the job, and the process (still
        // suspended) is resumed or killed by its owner.
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )?;
            AssignProcessToJobObject(job, process)?;
        }
        // The process was created suspended: resume its threads now that it is inside the job.
        resume_all_threads(child);
        Ok(created)
    }

    // Resume the (single) thread of a freshly spawned suspended process.
    pub(crate) fn resume_all_threads(child: &Child) {
        let pid = child.id();
        let Ok(snap) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, pid) }) else {
            return;
        };
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        unsafe {
            if Thread32First(snap, &mut entry).is_ok() {
                loop {
                    if entry.th32OwnerProcessID == pid {
                        if let Ok(thread) = OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID) {
                            ResumeThread(thread);
                            let _ = CloseHandle(thread);
                        }
                        break; // a freshly spawned suspended process has exactly one thread
                    }
                    if Thread32Next(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);
        }
    }

    // "es-ES", "en-US"... or nothing (then the env chain decides, as on Unix).
    // Reached only via locale_fallback, whose caller is #[cfg(not(test))].
    #[cfg_attr(test, allow(dead_code))]
    pub fn system_locale() -> Option<String> {
        const MAX: usize = 85; // LOCALE_NAME_MAX_LENGTH
        let mut buf = [0u16; MAX];
        let len = unsafe { GetUserDefaultLocaleName(&mut buf) };
        if len <= 1 {
            return None;
        }
        let text = String::from_utf16_lossy(&buf[..(len - 1) as usize]);
        (!text.is_empty()).then_some(text)
    }
}

#[cfg(windows)]
use win::system_locale;
#[cfg(not(windows))]
// Only reached from lang::detect_now, which is #[cfg(not(test))]
#[cfg_attr(test, allow(dead_code))]
fn system_locale() -> Option<String> {
    None
}

#[cfg(windows)]
pub(crate) use win::JobHandle;

// Locale name ("es-ES") for the language detection fallback, when the whole env chain is
// empty (the usual case on Windows). None on Unix, where LANG & co already decide.
// The only caller (lang::detect_now) is #[cfg(not(test))], so it is dead in the test build.
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn locale_fallback(name: &str) -> Option<String> {
    (name == "LANG").then(system_locale).flatten()
}

// The cancel path (kill_tree) is exercised by converters/tools tests, but those use the
// Unix group-kill or short-lived children. This Windows test proves the Job Object
// reaches DESCENDANTS: parent.bat idles while a `start /b` grandchild appends to a
// heartbeat file. After kill_tree the direct child must be reaped AND the heartbeat
// frozen. If the heartbeat keeps growing, the job is not covering descendants.
#[cfg(all(test, windows))]
mod job_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn kill_tree_sweeps_the_whole_job() {
        let dir = std::env::temp_dir().join(format!("citrus_job_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let hb = dir.join("hb.txt");
        let gc = dir.join("gc.bat");
        let parent = dir.join("parent.bat");
        std::fs::write(
            &gc,
            "@echo off\r\n:loop\r\necho x>>\"%~1\"\r\nping -n 2 127.0.0.1 >nul\r\ngoto loop\r\n",
        )
        .unwrap();
        std::fs::write(
            &parent,
            "@echo off\r\nstart \"\" /b %~1 %~2\r\nping -n 600 127.0.0.1 >nul\r\n",
        )
        .unwrap();

        let mut command = Command::new("cmd.exe");
        command
            .args(["/c", "parent.bat", "gc.bat", "hb.txt"])
            .current_dir(&dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let mut proc = spawn_killing_tree(command).expect("spawn");

        let deadline = Instant::now() + Duration::from_secs(20);
        while std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0) == 0 {
            assert!(Instant::now() < deadline, "grandchild never started: inconclusive");
            std::thread::sleep(Duration::from_millis(100));
        }

        kill_tree(&proc);
        // bounded reap check: if the job never attached, the parent idles for 600s and
        // this must fail fast instead of hanging the suite
        let reaped = loop {
            match proc.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => break None,
            }
            if Instant::now() > deadline {
                break None;
            }
        };
        assert!(reaped.is_some(), "direct child not reaped after kill_tree (job missing?)");

        std::thread::sleep(Duration::from_millis(800));
        let before = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
        std::thread::sleep(Duration::from_millis(3000));
        let after = std::fs::metadata(&hb).map(|m| m.len()).unwrap_or(0);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(before, after, "grandchild survived kill_tree: heartbeat grew {before}->{after}");
    }
}

// The install hint that goes with a missing tool: the apt line (upstream) on Unix, a
// winget equivalent where a well-known package exists on Windows.
pub(crate) fn install_hint(apt_names: &str) -> String {
    #[cfg(not(windows))]
    {
        format!("sudo apt install {apt_names}")
    }
    #[cfg(windows)]
    {
        let mut known: Vec<&str> = Vec::new();
        let mut rest: Vec<&str> = Vec::new();
        for name in apt_names.split_whitespace() {
            match name {
                "ffmpeg" => known.push("winget install Gyan.FFmpeg"),
                "imagemagick" => known.push("winget install ImageMagick.ImageMagick"),
                "pandoc" => known.push("winget install JohnMacFarlane.Pandoc"),
                "libreoffice" => known.push("winget install TheDocumentFoundation.LibreOffice"),
                "ghostscript" => known.push("winget install ArtifexSoftware.GhostScript"),
                "7zip" | "p7zip-full" => known.push("winget install OpenSource.7-Zip"),
                other => rest.push(other),
            }
        }
        let mut parts: Vec<String> = known.iter().map(|s| s.to_string()).collect();
        if !rest.is_empty() {
            parts.push(format!(
                "install {} and put it on your PATH",
                rest.join(", ")
            ));
        }
        parts.join(" ; or ")
    }
}

// Known install directories that winget/MSI leave OUT of PATH (LibreOffice, 7-Zip). Without
// this, an app launched by Explorer - which sees the real system PATH, not a dev shell - cannot
// find soffice/7z even though they are installed, so those conversions silently vanish.
#[cfg(windows)]
fn known_install_paths(name: &str) -> Vec<std::path::PathBuf> {
    let bases: &[&str] = match name {
        "soffice" | "libreoffice" => &[
            r"C:\Program Files\LibreOffice\program",
            r"C:\Program Files (x86)\LibreOffice\program",
        ],
        "7z" | "7za" | "7zz" => &[
            r"C:\Program Files\7-Zip",
            r"C:\Program Files (x86)\7-Zip",
        ],
        _ => &[],
    };
    bases
        .iter()
        .map(|b| std::path::PathBuf::from(b).join(format!("{name}.exe")))
        .collect()
}

#[cfg(not(windows))]
fn known_install_paths(_name: &str) -> Vec<std::path::PathBuf> {
    Vec::new()
}

// Is the bare tool present on PATH? (Windows: also try name.exe.)
fn on_path(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path) {
        #[cfg(not(windows))]
        {
            let cand = dir.join(name);
            if cand.is_file() {
                return Some(cand);
            }
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{name}.exe"));
            if exe.is_file() {
                return Some(exe);
            }
        }
    }
    None
}

// Resolve a bare tool name to an existing executable: PATH first, then the known install
// dirs. Returns None when nothing is found.
pub(crate) fn find_program(name: &str) -> Option<std::path::PathBuf> {
    if name.contains('/') || name.contains('\\') {
        let p = std::path::PathBuf::from(name);
        return if p.is_file() { Some(p) } else { None };
    }
    on_path(name).or_else(|| known_install_paths(name).into_iter().find(|p| p.is_file()))
}

// The executable to launch. Anything already on PATH (or already a path) is returned
// unchanged so CreateProcess resolves it exactly as before - crucially, this never swaps in
// an extensionless MSYS binary (e.g. /usr/bin/tar) in a dev shell. Only tools that are NOT
// on PATH fall back to a known install dir (soffice, 7-Zip), which is what makes an
// Explorer-launched app find them.
pub(crate) fn resolve_program(name: &str) -> String {
    if name.contains('/') || name.contains('\\') || on_path(name).is_some() {
        return name.to_string();
    }
    known_install_paths(name)
        .into_iter()
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_string())
}
