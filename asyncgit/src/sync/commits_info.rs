use super::utils::repo;
use crate::error::Result;
use git2::{Commit, Error, Oid};
use scopetime::scope_time;

/// identifies a single commit
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CommitId(Oid);

impl CommitId {
    /// create new CommitId
    pub fn new(id: Oid) -> Self {
        Self(id)
    }

    ///
    pub(crate) fn get_oid(self) -> Oid {
        self.0
    }
}

impl ToString for CommitId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

///
#[derive(Debug)]
pub struct CommitInfo {
    ///
    pub message: String,
    ///
    pub time: i64,
    ///
    pub author: String,
    ///
    pub id: CommitId,
}

///
pub fn get_commits_info(
    repo_path: &str,
    ids: &[Oid],
    message_length_limit: usize,
) -> Result<Vec<CommitInfo>> {
    scope_time!("get_commits_info");

    let repo = repo(repo_path)?;

    let commits = ids
        .iter()
        .map(|id| repo.find_commit(*id))
        .collect::<std::result::Result<Vec<Commit>, Error>>()?
        .into_iter();

    let res = commits
        .map(|c: Commit| {
            let message = get_message(&c, message_length_limit);
            let author = if let Some(name) = c.author().name() {
                String::from(name)
            } else {
                String::from("<unknown>")
            };
            CommitInfo {
                message,
                author,
                time: c.time().seconds(),
                id: CommitId(c.id()),
            }
        })
        .collect::<Vec<_>>();

    Ok(res)
}

fn get_message(c: &Commit, message_length_limit: usize) -> String {
    if let Some(msg) = c.message() {
        limit_str(msg, message_length_limit)
    } else {
        String::from("<unknown>")
    }
}

fn limit_str(s: &str, limit: usize) -> String {
    if let Some(first) = s.lines().next() {
        first.chars().take(limit).collect::<String>()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {

    use super::get_commits_info;
    use crate::error::Result;
    use crate::sync::{
        commit, stage_add_file, tests::repo_init_empty,
    };
    use std::{fs::File, io::Write, path::Path};

    #[test]
    fn test_log() -> Result<()> {
        let file_path = Path::new("foo");
        let (_td, repo) = repo_init_empty().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path = root.as_os_str().to_str().unwrap();

        File::create(&root.join(file_path))?.write_all(b"a")?;
        stage_add_file(repo_path, file_path).unwrap();
        let c1 = commit(repo_path, "commit1").unwrap();
        File::create(&root.join(file_path))?.write_all(b"a")?;
        stage_add_file(repo_path, file_path).unwrap();
        let c2 = commit(repo_path, "commit2").unwrap();

        let res =
            get_commits_info(repo_path, &vec![c2, c1], 50).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0].message.as_str(), "commit2");
        assert_eq!(res[0].author.as_str(), "name");
        assert_eq!(res[1].message.as_str(), "commit1");

        Ok(())
    }
}
