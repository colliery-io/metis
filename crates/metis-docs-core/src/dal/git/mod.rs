use git2::{Repository, TreeWalkMode, TreeWalkResult};
use std::path::Path;

/// Git repository utilities for branch detection and main branch resolution.
pub struct GitRepo {
    repo: Repository,
    main_branch: String,
}

impl GitRepo {
    /// Attempt to open a git repository from a workspace path.
    /// Returns `None` if the path is not inside a git repository.
    pub fn discover<P: AsRef<Path>>(workspace_path: P) -> Option<Self> {
        let repo = Repository::discover(workspace_path).ok()?;
        let main_branch = Self::resolve_main_branch_name(&repo)?;
        Some(Self { repo, main_branch })
    }

    /// Returns the name of the main branch ("main" or "master").
    pub fn main_branch(&self) -> &str {
        &self.main_branch
    }

    /// Returns the current HEAD branch name, or `None` if HEAD is detached.
    pub fn current_branch(&self) -> Option<String> {
        let head = self.repo.head().ok()?;
        if head.is_branch() {
            head.shorthand().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Returns `true` if the current branch is main/master.
    pub fn is_on_main(&self) -> bool {
        self.current_branch()
            .map(|b| b == self.main_branch)
            .unwrap_or(false)
    }

    /// Get a reference to the underlying git2 repository.
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    /// Get the full reference name for the main branch (e.g., "refs/heads/main").
    pub fn main_ref(&self) -> String {
        format!("refs/heads/{}", self.main_branch)
    }

    /// Read a file's content from main's committed tree.
    /// `tree_path` should be relative to the repo root (e.g., ".metis/initiatives/FOO/initiative.md").
    pub fn read_blob(&self, tree_path: &str) -> Option<String> {
        let reference = self.repo.find_reference(&self.main_ref()).ok()?;
        let commit = reference.peel_to_commit().ok()?;
        let tree = commit.tree().ok()?;
        let entry = tree.get_path(Path::new(tree_path)).ok()?;
        let blob = self.repo.find_blob(entry.id()).ok()?;
        std::str::from_utf8(blob.content()).ok().map(|s| s.to_string())
    }

    /// Check if a file exists in main's committed tree.
    pub fn blob_exists(&self, tree_path: &str) -> bool {
        let reference = match self.repo.find_reference(&self.main_ref()) {
            Ok(r) => r,
            Err(_) => return false,
        };
        let commit = match reference.peel_to_commit() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => return false,
        };
        tree.get_path(Path::new(tree_path)).is_ok()
    }

    /// List all markdown files under a given directory prefix in main's tree.
    /// Returns paths relative to the repo root.
    pub fn list_markdown_files(&self, dir_prefix: &str) -> Vec<String> {
        let mut files = Vec::new();

        let reference = match self.repo.find_reference(&self.main_ref()) {
            Ok(r) => r,
            Err(_) => return files,
        };
        let commit = match reference.peel_to_commit() {
            Ok(c) => c,
            Err(_) => return files,
        };
        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => return files,
        };

        let _ = tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            if let Some(name) = entry.name() {
                let full_path = if root.is_empty() {
                    name.to_string()
                } else {
                    format!("{}{}", root, name)
                };

                if full_path.starts_with(dir_prefix)
                    && full_path.ends_with(".md")
                    && name != "code-index.md"
                {
                    files.push(full_path);
                }
            }
            TreeWalkResult::Ok
        });

        files
    }

    /// Get the commit time of main's HEAD as a Unix timestamp (seconds).
    pub fn main_head_commit_time(&self) -> Option<f64> {
        let reference = self.repo.find_reference(&self.main_ref()).ok()?;
        let commit = reference.peel_to_commit().ok()?;
        Some(commit.time().seconds() as f64)
    }

    /// Get the path to the repository's working directory root.
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    /// Resolve whether the repository uses "main" or "master" as its primary branch.
    fn resolve_main_branch_name(repo: &Repository) -> Option<String> {
        // Check local branches first
        for name in &["main", "master"] {
            let refname = format!("refs/heads/{}", name);
            if repo.find_reference(&refname).is_ok() {
                return Some(name.to_string());
            }
        }

        // Fall back to remote tracking branches
        for name in &["main", "master"] {
            let refname = format!("refs/remotes/origin/{}", name);
            if repo.find_reference(&refname).is_ok() {
                return Some(name.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Signature;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_repo(branch_name: &str) -> (tempfile::TempDir, Repository) {
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Create an initial commit so HEAD exists
        let sig = Signature::now("Test", "test@test.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            let test_file = dir.path().join("README.md");
            fs::write(&test_file, "# Test").unwrap();
            index.add_path(Path::new("README.md")).unwrap();
            index.write_tree().unwrap()
        };
        {
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
                .unwrap();
        }

        // Rename the default branch if needed
        let current = {
            let head = repo.head().unwrap();
            head.shorthand().unwrap().to_string()
        };
        if current != branch_name {
            let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
            repo.branch(branch_name, &head_commit, true).unwrap();
            repo.set_head(&format!("refs/heads/{}", branch_name))
                .unwrap();
        }

        (dir, repo)
    }

    #[test]
    fn test_discover_git_repo() {
        let (dir, _repo) = create_test_repo("main");
        let git_repo = GitRepo::discover(dir.path());
        assert!(git_repo.is_some());
    }

    #[test]
    fn test_discover_non_git_dir() {
        let dir = tempdir().unwrap();
        // No git init — should return None
        let git_repo = GitRepo::discover(dir.path());
        assert!(git_repo.is_none());
    }

    #[test]
    fn test_main_branch_detection() {
        let (dir, _repo) = create_test_repo("main");
        let git_repo = GitRepo::discover(dir.path()).unwrap();
        assert_eq!(git_repo.main_branch(), "main");
    }

    #[test]
    fn test_master_branch_detection() {
        let (dir, repo) = create_test_repo("master");
        // Delete the "main" ref if it exists (git init may create it by default)
        if let Ok(mut reference) = repo.find_reference("refs/heads/main") {
            reference.delete().unwrap();
        }
        let git_repo = GitRepo::discover(dir.path()).unwrap();
        assert_eq!(git_repo.main_branch(), "master");
    }

    #[test]
    fn test_current_branch() {
        let (dir, _repo) = create_test_repo("main");
        let git_repo = GitRepo::discover(dir.path()).unwrap();
        assert_eq!(git_repo.current_branch(), Some("main".to_string()));
    }

    #[test]
    fn test_is_on_main() {
        let (dir, _repo) = create_test_repo("main");
        let git_repo = GitRepo::discover(dir.path()).unwrap();
        assert!(git_repo.is_on_main());
    }

    #[test]
    fn test_is_not_on_main_feature_branch() {
        let (dir, repo) = create_test_repo("main");

        // Create and checkout a feature branch
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        repo.branch("feature/test", &commit, false).unwrap();
        repo.set_head("refs/heads/feature/test").unwrap();

        let git_repo = GitRepo::discover(dir.path()).unwrap();
        assert!(!git_repo.is_on_main());
        assert_eq!(
            git_repo.current_branch(),
            Some("feature/test".to_string())
        );
    }

    #[test]
    fn test_detached_head_is_not_main() {
        let (dir, repo) = create_test_repo("main");

        // Detach HEAD
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        repo.set_head_detached(commit.id()).unwrap();

        let git_repo = GitRepo::discover(dir.path()).unwrap();
        assert!(!git_repo.is_on_main());
        assert_eq!(git_repo.current_branch(), None);
    }

    #[test]
    fn test_main_ref() {
        let (dir, _repo) = create_test_repo("main");
        let git_repo = GitRepo::discover(dir.path()).unwrap();
        assert_eq!(git_repo.main_ref(), "refs/heads/main");
    }

    /// Helper to create a repo with .metis/ files committed on main, then switch to a feature branch.
    fn create_repo_with_metis_files() -> (tempfile::TempDir, Repository) {
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Create .metis/ files
        let metis_dir = dir.path().join(".metis");
        fs::create_dir_all(metis_dir.join("initiatives/METIS-I-0001")).unwrap();
        fs::write(
            metis_dir.join("initiatives/METIS-I-0001/initiative.md"),
            "# Test Initiative\n\nContent here.",
        )
        .unwrap();
        fs::write(metis_dir.join("config.toml"), "[project]\nprefix = \"METIS\"")
            .unwrap();

        // Commit everything on main
        let sig = Signature::now("Test", "test@test.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            index.write().unwrap();
            index.write_tree().unwrap()
        };
        {
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "add metis files", &tree, &[])
                .unwrap();
        }

        // Create and switch to a feature branch
        {
            let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
            repo.branch("feature/test", &head_commit, false).unwrap();
        }
        repo.set_head("refs/heads/feature/test").unwrap();

        (dir, repo)
    }

    #[test]
    fn test_read_blob_from_main() {
        let (dir, _repo) = create_repo_with_metis_files();
        let git_repo = GitRepo::discover(dir.path()).unwrap();

        assert!(!git_repo.is_on_main());

        let content = git_repo
            .read_blob(".metis/initiatives/METIS-I-0001/initiative.md")
            .unwrap();
        assert!(content.contains("Test Initiative"));
    }

    #[test]
    fn test_read_blob_nonexistent() {
        let (dir, _repo) = create_repo_with_metis_files();
        let git_repo = GitRepo::discover(dir.path()).unwrap();

        let result = git_repo.read_blob(".metis/nonexistent.md");
        assert!(result.is_none());
    }

    #[test]
    fn test_blob_exists() {
        let (dir, _repo) = create_repo_with_metis_files();
        let git_repo = GitRepo::discover(dir.path()).unwrap();

        assert!(git_repo.blob_exists(".metis/config.toml"));
        assert!(git_repo.blob_exists(".metis/initiatives/METIS-I-0001/initiative.md"));
        assert!(!git_repo.blob_exists(".metis/nonexistent.md"));
    }

    #[test]
    fn test_list_markdown_files() {
        let (dir, _repo) = create_repo_with_metis_files();
        let git_repo = GitRepo::discover(dir.path()).unwrap();

        let files = git_repo.list_markdown_files(".metis/");
        assert_eq!(files.len(), 1); // Only the .md file, not config.toml
        assert!(files[0].contains("initiative.md"));
    }

    #[test]
    fn test_main_head_commit_time() {
        let (dir, _repo) = create_repo_with_metis_files();
        let git_repo = GitRepo::discover(dir.path()).unwrap();

        let time = git_repo.main_head_commit_time().unwrap();
        assert!(time > 0.0);
    }
}
