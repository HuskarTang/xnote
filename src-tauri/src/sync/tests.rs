use tempfile::TempDir;

fn init_repo_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    git2::Repository::init(dir.path()).unwrap();
    dir
}

#[test]
fn pending_state_round_trips_inside_git_dir() {
    let dir = init_repo_dir();
    let state = crate::sync::types::PendingSyncState {
        transaction_id: "sync-1".to_string(),
        transaction_type: crate::sync::types::SyncTransactionType::Sync,
        phase: crate::sync::types::SyncPhase::Conflicted,
        target_branch: "main".to_string(),
        temporary_branch: "xnote/local-sync/1".to_string(),
        remote_url: "https://example.invalid/repo.git".to_string(),
        pre_transaction_head: None,
        conflicts: vec![crate::sync::types::GitConflictFile {
            file_path: "note.md".to_string(),
            status: crate::sync::types::GitConflictStatus::BothModified,
            local_content: Some("local".to_string()),
            remote_content: Some("remote".to_string()),
            is_binary: false,
            diff_summary: None,
        }],
    };

    crate::sync::state::write_pending_state(dir.path(), &state).unwrap();
    let loaded = crate::sync::state::read_pending_state(dir.path())
        .unwrap()
        .unwrap();
    assert_eq!(loaded.transaction_id, "sync-1");
    assert_eq!(loaded.target_branch, "main");
    assert_eq!(loaded.conflicts[0].file_path, "note.md");

    crate::sync::state::clear_pending_state(dir.path()).unwrap();
    assert!(crate::sync::state::read_pending_state(dir.path())
        .unwrap()
        .is_none());
}

#[test]
fn safe_remote_label_masks_token_in_url() {
    let label = crate::sync::auth::safe_remote_label("https://user:token@example.com/repo.git");
    assert_eq!(label, "https://example.com/repo.git");
}

#[test]
fn connection_test_reports_missing_url_without_network() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = crate::config::GitSyncConfig {
        enabled: true,
        repository_url: "".to_string(),
        branch: "".to_string(),
        username: None,
        password: None,
        ssh_key_path: None,
        auth_type: "none".to_string(),
    };
    let manager = crate::sync::GitSyncManager::new(dir.path().to_path_buf(), config);
    let result = manager.test_connection().unwrap();

    assert!(!result.success);
    assert!(!result.repository_reachable);
    assert_eq!(result.message, "Git repository URL is required");
}

#[test]
fn connection_test_reports_missing_ssh_key_without_network() {
    let dir = tempfile::TempDir::new().unwrap();
    let missing_key = dir.path().join("missing-key");
    let config = crate::config::GitSyncConfig {
        enabled: true,
        repository_url: "git@example.com:owner/repo.git".to_string(),
        branch: "main".to_string(),
        username: None,
        password: None,
        ssh_key_path: Some(missing_key.to_string_lossy().to_string()),
        auth_type: "ssh".to_string(),
    };
    let manager = crate::sync::GitSyncManager::new(dir.path().to_path_buf(), config);
    let result = manager.test_connection().unwrap();

    assert!(!result.success);
    assert!(!result.repository_reachable);
    assert!(!result.auth_success);
    assert!(result.message.contains("SSH key file does not exist"));
}

#[test]
fn sanitized_connection_error_does_not_leak_url_credentials() {
    let repository_url = "https://user:secret@example.invalid/repo.git";
    let err = git2::Error::from_str(
        "failed to resolve address for https://user:secret@example.invalid/repo.git",
    );
    let message = super::sanitized_connection_error_message(repository_url, &err);

    assert!(!message.contains("secret"));
    assert!(!message.contains("user:secret"));
    assert!(message.contains("https://example.invalid/repo.git"));
}

fn write_file(dir: &std::path::Path, name: &str, content: &str) {
    std::fs::write(dir.join(name), content).unwrap();
}

fn commit_all(repo: &git2::Repository, message: &str) -> git2::Oid {
    let sig = git2::Signature::now("Test User", "test@example.invalid").unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let parents = match repo.head().ok().and_then(|head| head.peel_to_commit().ok()) {
        Some(parent) => vec![parent],
        None => vec![],
    };
    let parent_refs = parents.iter().collect::<Vec<_>>();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .unwrap()
}

fn basic_config(remote: &std::path::Path, branch: &str) -> crate::config::GitSyncConfig {
    crate::config::GitSyncConfig {
        enabled: true,
        repository_url: remote.to_string_lossy().to_string(),
        branch: branch.to_string(),
        username: None,
        password: None,
        ssh_key_path: None,
        auth_type: "none".to_string(),
    }
}

#[test]
fn setup_git_sync_handles_empty_data_directory() {
    let local = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );
    let result = manager.setup_git_sync().unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    assert!(local.path().join(".git").exists());
}

#[test]
fn setup_git_sync_pushes_existing_local_notes_to_empty_remote() {
    let local = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();
    write_file(local.path(), "local.md", "# Local\n");

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );
    let result = manager.setup_git_sync().unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    assert!(local.path().join(".git").exists());
    assert!(local.path().join("local.md").exists());
}

#[test]
fn setup_git_sync_creates_configured_branch_when_remote_missing() {
    let local = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();
    write_file(local.path(), "topic.md", "# Topic\n");

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "topic"),
    );
    let result = manager.setup_git_sync().unwrap();
    let remote_repo = git2::Repository::open_bare(remote.path()).unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    assert!(remote_repo.find_reference("refs/heads/topic").is_ok());
}

#[test]
fn setup_git_sync_detects_same_file_conflict() {
    let local = tempfile::TempDir::new().unwrap();
    let remote_work = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();

    let remote_repo =
        git2::Repository::clone(remote.path().to_str().unwrap(), remote_work.path()).unwrap();
    write_file(remote_work.path(), "note.md", "remote\n");
    commit_all(&remote_repo, "remote note");
    remote_repo
        .find_remote("origin")
        .unwrap()
        .push(&["refs/heads/master:refs/heads/main"], None)
        .unwrap();

    write_file(local.path(), "note.md", "local\n");
    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );
    let result = manager.setup_git_sync().unwrap();

    assert_eq!(
        result.outcome,
        crate::sync::types::GitSyncOutcome::Conflicted
    );
    assert_eq!(result.conflicts[0].file_path, "note.md");
    assert!(crate::sync::state::read_pending_state(local.path())
        .unwrap()
        .is_some());
}
