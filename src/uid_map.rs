//! User and group mapping operations.

use std::{fs::OpenOptions, io::Write};

use crate::FixedBufferWriter;

/// Map a range of user IDs inside a namespace.
///
/// # Signal Safety
/// This function is signal safe.
fn map_uid_range(outside_uid: u32, inside_uid: u32, count: u32) {
    // check if the string to Path conversion is signal safe
    let mut uid_map_file = OpenOptions::new()
        .write(true)
        .open("/proc/self/uid_map")
        .expect("Should be able to open /proc/self/uid_map for writing");

    // 10 bytes for each 32 bit integer, and 3 for the spaces.
    let mut uid_map_line = FixedBufferWriter::<33>::new();

    write!(
        &mut uid_map_line,
        "{} {} {}",
        inside_uid, outside_uid, count
    )
    .expect("buffer size should be enough");

    // user_namespaces(7) says that the uid_map file may be written to only
    // **once**, or else the write will return `EPERM`. So in theory, a
    // single write should completely write the buffer.
    match uid_map_file.write(uid_map_line.buffer()) {
        Err(_) => panic!("writing to /proc/self/uid_map failed"),
        Ok(nbytes) if nbytes != uid_map_line.len() => {
            panic!("writing to /proc/self/uid_map failed")
        }
        Ok(_) => (),
    };
}

/// Map a single uid
pub fn map_uid(outside_uid: u32, inside_uid: u32) {
    map_uid_range(outside_uid, inside_uid, 1);
}
