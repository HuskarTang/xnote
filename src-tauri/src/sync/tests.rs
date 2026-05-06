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
