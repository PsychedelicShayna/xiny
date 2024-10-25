use std::{fs, path::PathBuf};

use crate::shell::shell;
use anyhow::{self as ah, Context};
use dirs;

pub struct Repo {
    pub clone_uri: String,
    pub git_dir: PathBuf,
    pub repo_dir: PathBuf,
    pub branch: String,
}

impl Repo {
    pub fn new(clone_uri: &String, branch: &String) -> ah::Result<Self> {
        let repo_dir = dirs::data_local_dir()
            .context("Repo::new finding local data directory via dirs::data_local_dir()")?
            .join("xiny")
            .join("repo");

        if !repo_dir.exists() {
            fs::create_dir_all(&repo_dir)
                .context("Repo::new creating local data directory via fs::create_dir_all()")?;
        }

        Ok(Self {
            clone_uri: clone_uri.to_owned(),
            git_dir: repo_dir.join(".git"),
            repo_dir,
            branch: branch.to_owned(),
        })
    }

    pub fn sync(&self, force: bool) -> ah::Result<bool> {
        if !self.git_dir.exists() {
            self.clone(true)?;
            return Ok(true);
        }

        if force || self.is_remote_ahead()? {
            self.pull()?;
            return Ok(true);
        }

        Ok(false)
    }

    // Checks if the latest commit hash of the local repository does not match
    // that of the remote repository for the branch specified in the config.
    pub fn is_remote_ahead(&self) -> ah::Result<bool> {
        if !self.git_dir.exists() {
            ah::bail!("Repo::check_behind_remote git directory does not exist yet.");
        }

        let (stdout, _) = shell(
            "git",
            vec![
                "--git-dir",
                self.git_dir.display().to_string().as_str(),
                "rev-parse",
                &self.branch,
            ],
        )?;

        let local_commit_hash: String = stdout
            .trim()
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .collect();

        let (stdout, stderr) = shell(
            "git",
            vec![
                "--git-dir",
                &self.git_dir.display().to_string(),
                "ls-remote",
                "origin",
                &self.branch,
            ],
        )?;

        if stderr.len() > 0 {
            ah::bail!("Repo::check_behind_remote ls-remote stderr: {}", stderr);
        }

        let remote_commit_hash: String = stdout
            .trim()
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .collect();

        Ok(local_commit_hash != remote_commit_hash)
    }

    pub fn clone(&self, clean: bool) -> ah::Result<()> {
        if clean && self.git_dir.exists() {
            fs::remove_dir_all(&self.repo_dir)
                .context("Repo::clone removing local path via fs::remove_dir_all()")?;

            fs::create_dir_all(&self.repo_dir)
                .context("Repo::clone recreating local path via fs::create_dir_all()")?;
        } else if self.git_dir.exists() {
            ah::bail!("Repo::clone git directory already exists.");
        }

        let _ = shell(
            "git",
            vec![
                "clone",
                "--depth",
                "1",
                "--branch",
                &self.branch,
                &self.clone_uri,
                &self.repo_dir.display().to_string(),
            ],
        )?;

        if !self.git_dir.exists() {
            ah::bail!("Repo::clone git directory does not exist after cloning.");
        }

        Ok(())
    }

    pub fn pull(&self) -> ah::Result<()> {
        let _ = shell(
            "git",
            vec![
                "--git-dir",
                &self.git_dir.display().to_string(),
                "fetch",
                "origin",
                &self.branch,
            ],
        )?;

        Ok(())
    }
}
