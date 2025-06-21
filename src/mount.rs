//! mount(2) and umount2 helpers.

use std::{ffi::CStr, marker::PhantomData};

pub fn mount(
    source: Option<&CStr>,
    target: Option<&CStr>,
    fs_type: Option<&CStr>,
    mount_flags: u64,
) -> Result<(), std::io::Error> {
    let result = unsafe {
        libc::mount(
            source.map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            target.map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            fs_type.map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            mount_flags,
            std::ptr::null(),
        )
    };
    if result != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub struct Mount<'a> {
    flags: u64,
    target: &'a CStr,
}

// ConfiguredMount markers
pub struct ActionSetPropagation;
pub struct ActionBind;
pub struct ActionCreate;

pub struct ConfiguredMount<'a, Action> {
    flags: u64,
    target: &'a CStr,
    source: Option<&'a CStr>,
    fs_type: Option<&'a CStr>,
    _action: PhantomData<Action>,
}

impl<'a> ConfiguredMount<'a, ActionSetPropagation> {
    /// Set the propagation type for `target`
    pub fn mount(self) -> Result<(), std::io::Error> {
        mount(None, Some(self.target), None, self.flags)
    }

    /// Recursively change the propagation type of all mounts in a subtree.
    pub fn recursive(mut self) -> Self {
        self.flags |= libc::MS_REC;
        self
    }
}

/// manual: the filesystemtype and data arguments are ignored.
impl<'a> ConfiguredMount<'a, ActionBind> {
    /// Bind `source` to `target`
    pub fn mount(self) -> Result<(), std::io::Error> {
        mount(self.source, Some(self.target), None, self.flags)
    }

    /// All submounts under the `source` subtree (other than unbindable mounts)
    /// are also bind mounted at the corresponding location in the `target`
    /// subtree.
    ///
    ///
    /// By default, when a directory is bind mounted, only that directory is
    /// mounted; if there are any submounts under the directory tree, they are
    /// not bind mounted.
    pub fn recursive(mut self) -> Self {
        self.flags |= libc::MS_REC;
        self
    }
}

impl<'a> ConfiguredMount<'a, ActionCreate> {
    /// Create a new mount.
    pub fn mount(self) -> Result<(), std::io::Error> {
        mount(self.source, Some(self.target), self.fs_type, self.flags)
    }
}

pub enum MountPropagation {
    Private = libc::MS_PRIVATE as isize,
    Shared = libc::MS_SHARED as isize,
    Slave = libc::MS_SLAVE as isize,
    Unbindable = libc::MS_UNBINDABLE as isize,
}

/// A mount builder that enforces correct usage of the mount syscall and
/// prevents invalid flag and argument combinations.
///
/// Note: The builder is not complete yet, I only add options and flags as I
/// need them, although it would be a good idea to support all flags and
/// options later.
///
/// # Signal Safety
/// This builder is async-signal-safe, it doesn't allocate and it expects the
/// arguments to be &CStr.
///
/// # Example
/// ```no_run
/// Mount::new("/")
///     .set_propagation(MountPropagation::Private)
///     .recursive()
///     .mount()
/// ```
impl<'a> Mount<'a> {
    pub fn new(target: &'a CStr) -> Self {
        Self { flags: 0, target }
    }

    /// Set the propagation type of an exsting mount
    pub fn set_propagation(
        self,
        propagation_type: MountPropagation,
    ) -> ConfiguredMount<'a, ActionSetPropagation> {
        ConfiguredMount {
            flags: self.flags | propagation_type as u64,
            target: self.target,
            source: None,
            fs_type: None,
            _action: PhantomData,
        }
    }

    pub fn bind(self, source: &'a CStr) -> ConfiguredMount<'a, ActionBind> {
        ConfiguredMount {
            flags: self.flags | libc::MS_BIND,
            target: self.target,
            source: Some(source),
            fs_type: None,
            _action: PhantomData,
        }
    }

    /// Create a new mount.
    pub fn create(
        self,
        fs_type: &'a CStr,
        source: &'a CStr,
    ) -> ConfiguredMount<'a, ActionCreate> {
        ConfiguredMount {
            flags: self.flags,
            target: self.target,
            source: Some(source),
            fs_type: Some(fs_type),
            _action: PhantomData,
        }
    }
}
