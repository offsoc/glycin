use std::fs::{canonicalize, DirEntry, File};
use std::io::{self, BufRead, BufReader};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, OnceLock};

use nix::sys::resource;

use crate::{Error, SandboxMechanism};

static SYSTEM_SETUP: OnceLock<io::Result<SystemSetup>> = OnceLock::new();

pub struct Sandbox {
    sandbox_mechanism: SandboxMechanism,
    command: PathBuf,
    stdin: UnixStream,
    ro_bind_extra: Vec<PathBuf>,
}

impl Sandbox {
    pub fn new(sandbox_mechanism: SandboxMechanism, command: PathBuf, stdin: UnixStream) -> Self {
        Self {
            sandbox_mechanism,
            command,
            stdin,
            ro_bind_extra: Vec::new(),
        }
    }

    pub fn add_ro_bind(&mut self, path: PathBuf) {
        self.ro_bind_extra.push(path);
    }

    pub async fn spawn(self) -> crate::Result<Child> {
        // Determine command line args
        let (bin, args, final_arg) = match self.sandbox_mechanism {
            SandboxMechanism::Bwrap => {
                ("bwrap".into(), self.bwrap_args().await?, Some(self.command))
            }
            SandboxMechanism::FlatpakSpawn => {
                (
                    "flatpak-spawn".into(),
                    vec![
                        "--sandbox".into(),
                        // die with parent
                        "--watch-bus".into(),
                        // change working directory to something that exists
                        "--directory=/".into(),
                    ],
                    Some(self.command),
                )
            }
            SandboxMechanism::NotSandboxed => {
                eprintln!("WARNING: Glycin running without sandbox.");
                (self.command, vec![], None)
            }
        };

        let mut command = Command::new(bin);

        command.stdin(OwnedFd::from(self.stdin));

        command.args(args);
        if let Some(arg) = final_arg {
            command.arg(arg);
        }

        // Set memory limit for sandbox
        unsafe {
            command.pre_exec(|| Ok(Self::set_memory_limit()));
        }

        let cmd_debug = format!("{:?}", command);
        let subprocess = command
            .spawn()
            .map_err(|err| Error::SpawnError(cmd_debug, Arc::new(err)))?;

        Ok(subprocess)
    }

    async fn bwrap_args(&self) -> crate::Result<Vec<PathBuf>> {
        let mut args: Vec<PathBuf> = Vec::new();

        args.extend(
            [
                "--unshare-all",
                "--die-with-parent",
                // change working directory to something that exists
                "--chdir",
                "/",
                // Make /usr available as read only
                "--ro-bind",
                "/usr",
                "/usr",
                // Make tmpfs dev available
                "--dev",
                "/dev",
            ]
            .iter()
            .map(|x| (*x).into())
            .collect::<Vec<_>>(),
        );

        let system = SystemSetup::cached().await.as_ref().unwrap();

        // Symlink paths like /usr/lib64 to /lib64
        for (dest, src) in &system.lib_symlinks {
            args.push("--symlink".into());
            args.push(src.clone());
            args.push(dest.clone());
        }

        // Mount paths like /lib64 if they exist
        for dir in &system.lib_dirs {
            args.push("--ro-bind".into());
            args.push(dir.clone());
            args.push(dir.clone());
        }

        // Make extra dirs available
        for dir in &self.ro_bind_extra {
            args.push("--ro-bind".into());
            args.push(dir.clone());
            args.push(dir.clone());
        }

        // Make loader binary available if not in /usr. This is useful for testing and
        // edding loaders in user space.
        if !self.command.starts_with("/usr") {
            args.push("--ro-bind".into());
            args.push(self.command.clone());
            args.push(self.command.clone());
        }

        Ok(args)
    }

    fn set_memory_limit() {
        // Default to 1 GB memory limit
        let mut limit: resource::rlim_t = 1024 * 1024 * 1024;

        // Lookup free memory
        if let Ok(file) = File::open("/proc/meminfo") {
            let meminfo = BufReader::new(file);

            for line in meminfo.lines() {
                if let Ok(line) = line {
                    if line.starts_with("MemAvailable:") {
                        if let Some(value) = line
                            .split(' ')
                            .filter(|x| !x.is_empty())
                            .nth(1)
                            .and_then(|x| x.parse::<resource::rlim_t>().ok())
                        {
                            limit = value.saturating_mul(1024);
                            // Keep 200 MB free
                            limit = limit.saturating_sub(1024 * 1024 * 200);
                        }
                    }
                }
            }
        }

        if let Err(err) = resource::setrlimit(resource::Resource::RLIMIT_AS, limit, limit) {
            eprintln!("Error setrlimit(RLIMIT_AS, {limit}): {err}");
        }
    }
}

#[derive(Debug, Default)]
struct SystemSetup {
    // Dirs that need to be symlinked (UsrMerge)
    lib_symlinks: Vec<(PathBuf, PathBuf)>,
    // Dirs that need mounting (not UsrMerged)
    lib_dirs: Vec<PathBuf>,
}

impl SystemSetup {
    async fn cached() -> &'static io::Result<SystemSetup> {
        if let Some(system) = SYSTEM_SETUP.get() {
            system
        } else {
            let system = Self::new().await;
            SYSTEM_SETUP.set(system).unwrap();
            SYSTEM_SETUP.get().unwrap()
        }
    }

    async fn new() -> io::Result<SystemSetup> {
        let mut system = SystemSetup::default();

        system.load_lib_dirs().await?;

        Ok(system)
    }

    async fn load_lib_dirs(&mut self) -> io::Result<()> {
        let dir_content: Result<std::fs::ReadDir, std::io::Error> = std::fs::read_dir("/");

        match dir_content {
            Ok(dir_content) => {
                for entry in dir_content {
                    if let Err(err) = self.add_dir(entry).await {
                        eprintln!("Unable to access entry in root directory (/): {err}");
                    }
                }
            }
            Err(err) => {
                eprintln!("Unable to list root directory (/) entries: {err}");
            }
        }

        Ok(())
    }

    async fn add_dir(&mut self, entry: io::Result<DirEntry>) -> io::Result<()> {
        let entry = entry?;
        let path = entry.path();

        if let Some(last_segment) = path.file_name() {
            if last_segment.as_encoded_bytes().starts_with(b"lib") {
                let metadata = entry.metadata()?;
                if metadata.is_dir() {
                    // Lib dirs like /lib
                    self.lib_dirs.push(entry.path());
                } else if metadata.is_symlink() {
                    // Symlinks like /lib -> /usr/lib
                    let target = canonicalize(&path)?;
                    let mut expected_target = PathBuf::from("/usr");
                    expected_target.push(last_segment);
                    if target == expected_target {
                        self.lib_symlinks.push((path, target));
                    }
                }
            }
        };

        Ok(())
    }
}
