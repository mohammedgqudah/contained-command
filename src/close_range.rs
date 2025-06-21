/// A simple* wrapper around close_range(2)
///
/// This is written for a single use case when spawning a container, that's why
/// it doesn't handle all errors and cases.
pub struct CloseRangeBuilder {
    flags: u32,
    first: u32,
    last: u32,
}

impl CloseRangeBuilder {
    pub fn new(first: u32, last: u32) -> Self {
        Self {
            flags: 0,
            first,
            last,
        }
    }

    /// Set the close-on-exec flag on the file descriptors, rather
    /// than immediately closing them.
    pub fn close_on_exec(mut self) -> Self {
        self.flags |= libc::CLOSE_RANGE_CLOEXEC;
        self
    }

    /// Unshare the specified file descriptors from any other processes before
    /// closing them, avoiding races with other threads sharing the file
    /// descriptor table.
    pub fn unshare_before_closing(mut self) -> Self {
        self.flags |= libc::CLOSE_RANGE_UNSHARE;
        self
    }

    /// Close the file descriptors from `first` to `last`
    pub fn close(&self) -> Result<(), ()> {
        let ret = unsafe {
            libc::syscall(
                libc::SYS_close_range,
                self.first,
                self.last,
                self.flags,
            )
        };

        match ret {
            0 => Ok(()),
            _ => Err(()),
        }
    }
}
