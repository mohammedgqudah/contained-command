use std::{os::unix::fs::chroot, process::Command};

use crate::{
    clone3::{CloneResult, clone3},
    close_range::CloseRangeBuilder,
    error::Result,
    mount::{Mount, MountPropagation, mount},
};
pub struct Container {
    cmd: String,
    args: Vec<String>,
}

impl Container {
    pub fn new(cmd: String) -> Self {
        // TODO: should be a c os-string
        Self { cmd, args: vec![] }
    }

    pub fn spawn(&self) -> Result<()> {
        // SAFETY: The child will only run async-signal-safe functions
        // See: signal-safety(7)
        let clone = unsafe {
            let flags = libc::CLONE_CLEAR_SIGHAND
                | libc::CLONE_INTO_CGROUP
                | libc::CLONE_NEWIPC
                | libc::CLONE_NEWNET
                | libc::CLONE_NEWUTS
                | libc::CLONE_NEWNS
                | libc::CLONE_NEWPID
                | libc::CLONE_NEWUSER;

            clone3(flags as u64).unwrap()
        };

        match clone {
            CloneResult::Parent(_child) => {
                return Ok(());
            }
            CloneResult::Child => {
                // Ensure all file descriptors are closed when executing the
                // child process so they are not inherited by
                // the container.
                //
                // Note: From what I see, Rust opens all files with the
                // close-on-exec flag on linux, but doing this
                // here is just in case a file was opened outside of the std
                // lib.
                CloseRangeBuilder::new(3, u32::MAX)
                    .close_on_exec()
                    .close()
                    .expect("should close all file descriptors");

                // Make sure the new root mount in the namespace is not shared
                // with the host.
                // See: https://lwn.net/Articles/689856/
                Mount::new(c"/")
                    .set_propagation(MountPropagation::Private)
                    .recursive()
                    .mount()
                    .unwrap();

                // Make the container root a mount.
                Mount::new(c"/tmp/bbox").bind(c"/tmp/bbox").mount().unwrap();

                Mount::new(c"/tmp/bbox/proc")
                    .no_dev()
                    .no_suid()
                    .no_exec()
                    .create(c"proc", c"proc")
                    .mount()
                    .unwrap();

                Mount::new(c"/tmp/bbox/sys")
                    .readonly()
                    .no_dev()
                    .no_suid()
                    .no_exec()
                    .create(c"sysfs", c"sys")
                    .mount()
                    .unwrap();

                chroot("/").unwrap();
                self.do_exec()?;
            }
        };
        Ok(())
    }

    fn do_exec(&self) -> Result<()> {
        todo!()
    }
}
