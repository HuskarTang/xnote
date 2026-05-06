use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::types::PendingSyncState;

const STATE_FILE_NAME: &str = "xnote-sync-state.json";

fn state_path(repo_path: &Path) -> PathBuf {
    repo_path.join(".git").join(STATE_FILE_NAME)
}

pub fn read_pending_state(repo_path: &Path) -> Result<Option<PendingSyncState>> {
    let path = state_path(repo_path);
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read pending sync state: {}", path.display()))?;
    let state = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse pending sync state: {}", path.display()))?;
    Ok(Some(state))
}

pub fn write_pending_state(repo_path: &Path, state: &PendingSyncState) -> Result<()> {
    let path = state_path(repo_path);
    let content =
        serde_json::to_string_pretty(state).context("Failed to serialize pending sync state")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write pending sync state: {}", path.display()))
}

pub fn clear_pending_state(repo_path: &Path) -> Result<()> {
    let path = state_path(repo_path);
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to remove pending sync state: {}", path.display()))?;
    }
    Ok(())
}
