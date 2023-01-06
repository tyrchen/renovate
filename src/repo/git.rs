#![allow(clippy::unwrap_used)]
use git2::{Error, IndexAddOption, Object, ObjectType, Oid, Repository, Signature};
use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct GitRepo(Arc<Repository>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpVersion {
    Major,
    Minor,
    Patch,
}

const IGNORE_RULES: &str = "dist\nnode_modules\n";

impl GitRepo {
    pub fn init(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let repo = if let Ok(repo) = Repository::discover(path) {
            repo
        } else {
            Repository::init(path)?
        };

        let ignore = path.join(".gitignore");
        if !ignore.exists() {
            fs::write(ignore, IGNORE_RULES).expect("should write");
        }
        Ok(GitRepo(Arc::new(repo)))
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let repo = Repository::discover(path)?;
        Ok(GitRepo(Arc::new(repo)))
    }

    pub fn is_current_dir(&self) -> bool {
        let path = env::current_dir().unwrap();
        self.get_root_path() == path
    }

    pub fn get_root_path(&self) -> &Path {
        self.0.path().parent().unwrap()
    }

    pub fn get_relative_dir(&self) -> PathBuf {
        let path = env::current_dir().unwrap();
        let repo_path = self.get_root_path();
        path.strip_prefix(repo_path).unwrap().to_path_buf()
    }

    pub fn is_dirty(&self) -> bool {
        // let mut status = Default::default();
        let statuses = self.0.statuses(None).unwrap();
        let mut filtered = statuses
            .iter()
            .filter(|s| !s.status().is_ignored())
            .peekable();

        filtered.peek().is_some()
    }

    pub fn commit(&self, message: impl AsRef<str>) -> Result<Oid, Error> {
        let mut index = self.0.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        let oid = index.write_tree()?;
        index.write()?;
        let sig = Signature::now("Bot", "bot@renovate.tools")?;
        let parent_commit = self
            .find_last_commit()
            .ok()
            .and_then(|o| o.into_commit().ok());

        let parents = if let Some(c) = parent_commit.as_ref() {
            vec![c]
        } else {
            vec![]
        };

        let tree = self.0.find_tree(oid)?;
        self.0
            .commit(Some("HEAD"), &sig, &sig, message.as_ref(), &tree, &parents)
    }

    pub fn tag(&self, name: impl AsRef<str>, message: impl AsRef<str>) -> Result<Oid, Error> {
        let sig = Signature::now("Bot", "bot@cellacloud.com")?;
        let head_object = self.find_last_commit()?;
        self.0
            .tag(name.as_ref(), &head_object, &sig, message.as_ref(), false)
    }

    pub fn get_last_commit_id(&self) -> Result<String, Error> {
        let commit = self.find_last_commit()?;
        let sid = commit.short_id()?.as_str().unwrap().to_owned();
        Ok(sid)
    }

    pub fn checkout(&self, refname: &str) -> Result<String, Error> {
        let old_ref = self.0.head()?.shorthand().unwrap().to_owned();
        let (object, reference) = self.0.revparse_ext(refname)?;
        self.0.checkout_tree(&object, None)?;

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => self.0.set_head(gref.name().unwrap())?,
            // this is a commit, not a reference
            None => self.0.set_head_detached(object.id())?,
        };

        Ok(old_ref)
    }

    pub fn find_last_commit(&self) -> Result<Object, Error> {
        self.0.head()?.resolve()?.peel(ObjectType::Commit)
    }

    pub fn list_tags(&self, n: usize, prefix: Option<String>) -> Result<Vec<String>, Error> {
        let tags = self
            .0
            .tag_names(None)?
            .into_iter()
            .rev()
            .filter(|t| t.is_some())
            .map(|t| t.unwrap().to_owned())
            .filter(|t| {
                if let Some(p) = prefix.as_ref() {
                    t.starts_with(p)
                } else {
                    true
                }
            })
            .take(n)
            .collect();

        Ok(tags)
    }

    pub fn get_prefix_name(&self) -> Option<String> {
        if !self.is_current_dir() {
            let path = env::current_dir().ok();
            path.as_ref()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .map(|s| s.to_owned())
        } else {
            None
        }
    }
}

impl fmt::Debug for GitRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GitRepo({:?})", self.get_root_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn git_repo_should_work() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();
        let repo = GitRepo::init(root).unwrap();
        fs::write(root.join("file.txt"), "Hello World")
            .await
            .unwrap();
        repo.commit("Initial commit").unwrap();
        repo.tag("v1.0.0", "Initial tag").unwrap();
        let id = repo.get_last_commit_id().unwrap();
        assert_eq!(id.len(), 7);
        fs::write(root.join("file.txt"), "Hello Tyr").await.unwrap();
        repo.commit("2nd commit").unwrap();
        repo.tag("v2.0.0", "2nd tag").unwrap();
        let old_ref = repo.checkout("v1.0.0").unwrap();
        assert_eq!(old_ref, "master");
        repo.checkout(&old_ref).unwrap();
    }
}
