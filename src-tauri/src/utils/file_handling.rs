use std::path::Path;
use tar::Builder;
use tokio::fs;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};
use zip::ZipArchive;

/// # Example
///
/// Copies files recursivley from one directory to another
///
/// ```rust
/// Box::pin(copy_dir_all(&src, &dst)).await?;
/// ```
pub async fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), FileHandlingError> {
    fs::create_dir_all(&dst).await?;
    let mut src_entries = fs::read_dir(src).await?;
    while let Some(entry) = src_entries.next_entry().await? {
        let ty = entry.file_type().await?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name())).await?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name())).await?;
        }
    }
    Ok(())
    // IDEA just for fun implement a function that spawns a new copy all thread for all found subdirectories at once and compare speed. Think Tree
}

pub async fn compress(dir: &Path) -> Result<Vec<u8>, FileHandlingError> {
    let dir = dir.to_path_buf();

    let tar_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, std::io::Error> {
        let mut archive = Builder::new(Vec::new());
        archive.append_dir_all("", dir)?;
        archive.finish()?;

        let tar_data = archive.into_inner()?;
        Ok(tar_data)
    })
    .await??;

    Ok(tar_data)
}

pub async fn extract(zip_path: &Path, path: &Path) -> Result<(), FileHandlingError> {
    let temp_extract_path = path.with_extension("extracting");

    if temp_extract_path.try_exists()? {
        fs::remove_dir_all(&temp_extract_path).await?;
    }

    let zip_path_owned = zip_path.to_owned();
    let temp_extract_path_owned = temp_extract_path.clone();

    tokio::task::spawn_blocking(move || -> Result<(), FileHandlingError> {
        let file = std::fs::File::open(&zip_path_owned)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(&temp_extract_path_owned)?;

        Ok(())
    })
    .await??;

    fs::rename(&temp_extract_path, path).await?;

    fs::remove_file(zip_path).await?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum FileHandlingError {
    // TODO Improve
    #[error("io operation failed: {0}")]
    IoError(#[from] std::io::Error),
    #[error("tokio task join: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}
