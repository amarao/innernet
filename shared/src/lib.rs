pub use anyhow::Error;
use lazy_static::lazy_static;
use std::{
    fs::{self, File},
    io,
    os::unix::fs::PermissionsExt,
    path::Path,
    time::Duration,
};

pub mod interface_config;
#[cfg(target_os = "linux")]
mod netlink;
pub mod prompts;
pub mod types;
pub mod wg;

pub use types::*;

lazy_static! {
    pub static ref CLIENT_CONFIG_DIR: &'static Path = Path::new("/etc/innernet");
    pub static ref CLIENT_DATA_DIR: &'static Path = Path::new("/var/lib/innernet");
    pub static ref SERVER_CONFIG_DIR: &'static Path = Path::new("/etc/innernet-server");
    pub static ref SERVER_DATABASE_DIR: &'static Path = Path::new("/var/lib/innernet-server");
    pub static ref REDEEM_TRANSITION_WAIT: Duration = Duration::from_secs(5);
}

pub const PERSISTENT_KEEPALIVE_INTERVAL_SECS: u16 = 25;
pub const INNERNET_PUBKEY_HEADER: &str = "X-Innernet-Server-Key";

pub fn ensure_dirs_exist(dirs: &[&Path]) -> Result<(), WrappedIoError> {
    for dir in dirs {
        match fs::create_dir(dir).with_path(dir) {
            Err(e) if e.kind() != io::ErrorKind::AlreadyExists => {
                return Err(e);
            },
            _ => {
                warn_on_dangerous_mode(dir).with_path(dir)?;
            },
        }
    }
    Ok(())
}

pub fn warn_on_dangerous_mode(path: &Path) -> Result<(), io::Error> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let permissions = metadata.permissions();
    let mode = permissions.mode() & 0o777;

    if mode & 0o007 != 0 {
        log::warn!(
            "{} is world-accessible (mode is {:#05o}). This is probably not what you want.",
            path.to_string_lossy(),
            mode
        );
    }
    Ok(())
}

/// Updates the permissions of a file or directory. Returns `Ok(true)` if
/// permissions had to be changed, `Ok(false)` if permissions were already
/// correct.
pub fn chmod(file: &File, new_mode: u32) -> Result<bool, io::Error> {
    let metadata = file.metadata()?;
    let mut permissions = metadata.permissions();
    let mode = permissions.mode() & 0o777;
    let updated = if mode != new_mode {
        permissions.set_mode(new_mode);
        file.set_permissions(permissions)?;
        true
    } else {
        false
    };

    Ok(updated)
}
