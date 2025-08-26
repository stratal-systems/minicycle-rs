
// BEGIN CHATGPT CODE

use std::fs;
use std::io;
use std::os::unix::fs as unix_fs;
use std::path::Path;

pub fn force_symlink<T: AsRef<Path>, U: AsRef<Path>>(target: T, link_path: U) -> io::Result<()> {
    let link_path = link_path.as_ref();

    match fs::symlink_metadata(link_path) {
        Ok(_) => fs::remove_file(link_path)?, // safely removes symlinks and files
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }

    unix_fs::symlink(target, link_path)?;
    Ok(())
}

// END CHATGPT CODE

