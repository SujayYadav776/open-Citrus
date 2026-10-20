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
        const CREATE_SUSPENDED: u32 = 0x0000_0004;
        command.creation_flags(CREATE_SUSPENDED);
        let child = command.spawn()?;
        // Degraded case: if the job cannot be set up we resume the process anyway and
        // fall back to killing just the direct child.
        match unsafe { win::attach_job_and_resume(&child) } {
            Ok(job) => Ok(Proc { child, _job: job }),
            Err(_) => {
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
fn system_locale() -> Option<String> {
    None
}

#[cfg(windows)]
pub(crate) use win::JobHandle;

// Locale name ("es-ES") for the language detection fallback, when the whole env chain is
// empty (the usual case on Windows). None on Unix, where LANG & co already decide.
pub(crate) fn locale_fallback(name: &str) -> Option<String> {
    (name == "LANG").then(system_locale).flatten()
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
