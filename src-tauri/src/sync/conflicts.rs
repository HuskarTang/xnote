use anyhow::Result;
use git2::Repository;
use std::path::Path;

use super::types::{GitConflictFile, GitConflictStatus};

#[allow(dead_code)]
pub fn extract_conflicts(repo: &Repository, _repo_path: &Path) -> Result<Vec<GitConflictFile>> {
    let index = repo.index()?;
    extract_conflicts_from_index(repo, index)
}

pub fn extract_conflicts_from_index(
    repo: &Repository,
    index: git2::Index,
) -> Result<Vec<GitConflictFile>> {
    let mut conflicts = Vec::new();

    if !index.has_conflicts() {
        return Ok(conflicts);
    }

    for conflict in index.conflicts()? {
        let conflict = conflict?;
        let path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .and_then(|entry| std::str::from_utf8(&entry.path).ok())
            .unwrap_or("unknown")
            .to_string();

        let local_content = conflict
            .their
            .as_ref()
            .and_then(|entry| repo.find_blob(entry.id).ok())
            .and_then(|blob| {
                std::str::from_utf8(blob.content())
                    .ok()
                    .map(|content| content.to_string())
            });
        let remote_content = conflict
            .our
            .as_ref()
            .and_then(|entry| repo.find_blob(entry.id).ok())
            .and_then(|blob| {
                std::str::from_utf8(blob.content())
                    .ok()
                    .map(|content| content.to_string())
            });

        let status = match (conflict.our.is_some(), conflict.their.is_some()) {
            (true, true) => GitConflictStatus::BothModified,
            (true, false) => GitConflictStatus::RemoteDeleted,
            (false, true) => GitConflictStatus::LocalDeleted,
            (false, false) => GitConflictStatus::Unsupported,
        };

        conflicts.push(GitConflictFile {
            file_path: path,
            status,
            local_content,
            remote_content,
            is_binary: false,
            diff_summary: None,
        });
    }

    Ok(conflicts)
}
