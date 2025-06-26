use std::{
    ffi::{CString, OsStr},
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::Duration,
};

use crate::{
    clone3::{CloneResult, clone3},
    close_range::CloseRangeBuilder,
    error::Result,
    mount::{Mount, MountPropagation},
};

pub struct Container {
    cmd: CString,
    args: Vec<CString>,
    env: Vec<CString>,
}

impl Container {
    pub fn new<C: AsRef<OsStr>>(cmd: C) -> Self {
        let cmd = CString::new(cmd.as_ref().as_encoded_bytes())
            .expect("Null in the command");
        let mut args = Vec::with_capacity(2);
        args.push(cmd.clone());

        Self {
            cmd,
            args,
            env: vec![],
        }
    }

    pub fn arg<C: AsRef<OsStr>>(mut self, arg: C) -> Self {
        let arg = CString::new(arg.as_ref().as_encoded_bytes())
            .expect("Null in the arg");

        self.args.push(arg);
        self
    }

    pub fn env<C: AsRef<OsStr>>(mut self, env: C) -> Self {
        let arg = CString::new(env.as_ref().as_encoded_bytes())
            .expect("Null in the env");

        self.env.push(arg);
        self
    }

    /// Return a c-style null-terminated array for self.args
    fn get_argv(&self) -> Vec<*const i8> {
        let mut argv: Vec<*const i8> =
            self.args.iter().map(|arg| arg.as_ptr()).collect();

        argv.push(std::ptr::null());
        argv
    }

    /// Return a c-style null-terminated array for self.env
    fn get_envp(&self) -> Vec<*const i8> {
        let mut envp: Vec<*const i8> =
            self.env.iter().map(|env| env.as_ptr()).collect();

        envp.push(std::ptr::null());
        envp
    }

    pub fn spawn(&mut self) -> Result<()> {
        let argv = self.get_argv();
        let envp = self.get_envp();

        let (mut parent_sock, mut child_sock) = match UnixStream::pair() {
            Ok((sock1, sock2)) => (sock1, sock2),
            Err(e) => {
                panic!("Couldn't create a pair of sockets: {e:?}");
            }
        };

        child_sock
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();

        let mut read_buf = [0];

        // SAFETY: The child will only run async-signal-safe functions
        // See: signal-safety(7)
        let clone = unsafe {
            let flags = libc::CLONE_CLEAR_SIGHAND
                | libc::CLONE_INTO_CGROUP
                | libc::CLONE_NEWIPC
                | libc::CLONE_NEWNET
                | libc::CLONE_NEWUTS
                | libc::CLONE_NEWNS
                //| libc::CLONE_NEWUSER
                | libc::CLONE_NEWPID;

            clone3(flags as u64).expect("clone failed")
        };

        match clone {
            CloneResult::Parent(child) => {
                drop(child_sock);
                //map_uid(format!("/proc/{}/uid_map", child.pid), 0, 0)
                //    .unwrap();

                parent_sock.write_all(&[1]).unwrap(); // wake child
                drop(parent_sock);
                unsafe {
                    libc::waitpid(child.pid as i32, std::ptr::null_mut(), 0)
                };
                return Ok(());
            }
            CloneResult::Child => {
                drop(parent_sock);
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

                match child_sock.read(&mut read_buf) {
                    Ok(0) => panic!("Parent failed to initialize container"),
                    Ok(_) => (),
                    Err(_) => panic!("Error reading pipe"),
                }

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

                unsafe {
                    libc::syscall(
                        libc::SYS_pivot_root,
                        c"/tmp/bbox".as_ptr(),
                        c"/tmp/bbox/old_root".as_ptr(),
                    );
                    // TODO: detach /old_root & rm old_root
                    libc::chdir(c"/".as_ptr());
                };
                self.do_exec(argv.as_ptr(), envp.as_ptr()).unwrap();

                println!("exec failed");
            }
        };
        Ok(())
    }

    fn do_exec(
        &self,
        argv: *const *const i8,
        envp: *const *const i8,
    ) -> std::result::Result<(), std::io::Error> {
        let result = unsafe { libc::execve(self.cmd.as_ptr(), argv, envp) };
        if result != 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
