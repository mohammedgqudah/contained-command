use std::{io::Error, mem};

pub struct Child {
    pub tid: u64,
    pub pid: i64,
}

pub struct Clone3 {}
pub enum CloneResult {
    Child,
    Parent(Child),
}

/// A tiny wrapper around the clone3 syscall.
///
/// # Safety
/// The clone syscall is inherently unsafe in a multithreaded program, you must
/// only call async-signal safe functions before you `exec`
///
/// # Example
///
/// ```no_run
/// unsafe {
///     let result = clone3(libc::CLONE_VM | libc::SIGCHLD).unwrap();
///     match result {
///         CloneResult::Child => {
///             println!("In the child");
///         }
///         CloneResult::Parent(child) => {
///             println!("Spawned pid={} tid={}", child.pid, child.tid);
///         }
///     }
/// }
/// ```
pub unsafe fn clone3(flags: u64) -> Result<CloneResult, std::io::Error> {
    let flags = flags | libc::CLONE_PARENT_SETTID as u64;
    let mut child_tid: mem::MaybeUninit<u64> = std::mem::MaybeUninit::uninit();

    let clone_args = libc::clone_args {
        flags,
        pidfd: 0,
        child_tid: 0,
        parent_tid: child_tid.as_mut_ptr() as u64,
        exit_signal: libc::SIGCHLD as u64,
        stack: 0,
        stack_size: 0,
        tls: 0,
        set_tid: 0,
        set_tid_size: 0,
        cgroup: 0,
    };

    // SAFETY: is the caller’s responsibility.
    let pid = unsafe {
        libc::syscall(
            libc::SYS_clone3,
            &clone_args as *const libc::clone_args,
            size_of::<libc::clone_args>(),
        )
    };

    if pid < 0 {
        return Err(Error::last_os_error());
    };

    // SAFETY: The clone syscall finished successfuly and it initilized the
    // variable.
    let child_tid = unsafe { child_tid.assume_init() };

    Ok(match pid {
        0 => CloneResult::Child,
        _ => CloneResult::Parent(Child {
            tid: child_tid,
            pid,
        }),
    })
}
