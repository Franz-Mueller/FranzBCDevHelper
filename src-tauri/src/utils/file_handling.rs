use std::path::Path;
use std::{fs, io};

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
    // TODO refactor with focus on concurrency/parallelism
    // - Files should be copied with tkoio
    // - What is faster, taking the extra steps to process all subdirectories with join or recursivly going thorugh them one by one?
}
