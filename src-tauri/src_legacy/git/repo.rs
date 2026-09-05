use git2::Repository;
use std::path::Path;
use url::Url;

struct BcRepo {
    repo: Repository,
}

impl BcRepo {
    pub async fn clone_from_url(from_url: Url, into_folder: &Path) -> Result<Self, RepoError> {
        let repo = match Repository::clone(&from_url.to_string(), &into_folder) {
            Ok(repo) => repo,
            Err(e) => {
                return Err(RepoError::CloneFailed(
                    from_url.to_string(),
                    into_folder.display().to_string(),
                    e,
                ));
            }
        }; // TODO call as async operation // TODO map specific Error code for verbose output
        Ok(Self { repo: repo })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("could not clone repo {0} into {1}: {2}")]
    CloneFailed(String, String, git2::Error),
}
