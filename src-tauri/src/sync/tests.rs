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
fn default_ssh_key_path_prefers_id_rsa_from_user_ssh_directory() {
    let home = tempfile::TempDir::new().unwrap();
    let ssh_dir = home.path().join(".ssh");
    std::fs::create_dir_all(&ssh_dir).unwrap();
    let key_path = ssh_dir.join("id_rsa");
    std::fs::write(&key_path, "private key").unwrap();

    assert_eq!(
        crate::sync::auth::default_ssh_key_path_in(home.path()),
        Some(key_path)
    );
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

#[test]
fn missing_remote_branch_detection_checks_wrapped_error_sources() {
    let err = anyhow::anyhow!("couldn't find remote ref refs/heads/main")
        .context("Failed to fetch branch main");

    assert!(super::is_missing_remote_branch_error(&err));
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

fn local_branch_names(repo: &git2::Repository) -> Vec<String> {
    repo.branches(Some(git2::BranchType::Local))
        .unwrap()
        .map(|branch| {
            let (branch, _) = branch.unwrap();
            branch.name().unwrap().unwrap().to_string()
        })
        .collect()
}

fn temporary_branch_names(repo: &git2::Repository) -> Vec<String> {
    local_branch_names(repo)
        .into_iter()
        .filter(|name| {
            name.starts_with("xnote/local-bootstrap/") || name.starts_with("xnote/local-sync/")
        })
        .collect()
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
fn setup_git_sync_ignores_logs_directory_in_data_repo() {
    let local = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();
    write_file(local.path(), "note.md", "# Note\n");

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );
    let result = manager.setup_git_sync().unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    let gitignore = std::fs::read_to_string(local.path().join(".gitignore")).unwrap();
    assert!(gitignore.lines().any(|line| line.trim() == "logs/"));
}

#[test]
fn fetch_branch_updates_remote_tracking_ref() {
    let local = init_repo_dir();
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

    let local_repo = git2::Repository::open(local.path()).unwrap();
    local_repo
        .remote("origin", remote.path().to_str().unwrap())
        .unwrap();
    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );

    manager.fetch_branch(&local_repo, "main").unwrap();

    assert!(local_repo
        .find_reference("refs/remotes/origin/main")
        .is_ok());
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

#[test]
fn continue_git_sync_uses_resolved_local_content() {
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
    let setup_result = manager.setup_git_sync().unwrap();
    assert_eq!(
        setup_result.outcome,
        crate::sync::types::GitSyncOutcome::Conflicted
    );
    assert!(manager.get_pending_git_sync().unwrap().is_some());

    let result = manager
        .continue_git_sync(vec![crate::sync::types::ResolvedConflictFile {
            file_path: "note.md".to_string(),
            final_content: Some("final local content\n".to_string()),
        }])
        .unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    assert_eq!(
        std::fs::read_to_string(local.path().join("note.md")).unwrap(),
        "final local content\n"
    );
    assert!(crate::sync::state::read_pending_state(local.path())
        .unwrap()
        .is_none());
}

#[test]
fn abort_git_sync_clears_pending_state() {
    let local = init_repo_dir();
    let pending = crate::sync::types::PendingSyncState {
        transaction_id: "abort".to_string(),
        transaction_type: crate::sync::types::SyncTransactionType::Sync,
        phase: crate::sync::types::SyncPhase::Conflicted,
        target_branch: "main".to_string(),
        temporary_branch: "xnote/local-sync/abort".to_string(),
        remote_url: "https://example.invalid/repo.git".to_string(),
        pre_transaction_head: None,
        conflicts: vec![],
    };
    crate::sync::state::write_pending_state(local.path(), &pending).unwrap();
    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(std::path::Path::new("/tmp/nonexistent.git"), "main"),
    );

    let result = manager.abort_git_sync().unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    assert!(crate::sync::state::read_pending_state(local.path())
        .unwrap()
        .is_none());
}

#[test]
fn manual_sync_pushes_local_change() {
    let local = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();
    write_file(local.path(), "local.md", "initial\n");

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );
    assert_eq!(
        manager.setup_git_sync().unwrap().outcome,
        crate::sync::types::GitSyncOutcome::Success
    );

    write_file(local.path(), "local.md", "changed\n");
    let result = manager.perform_sync().unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    assert!(result.pushed);
}

#[test]
fn manual_sync_removes_reachable_temporary_branches_after_success() {
    let local = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();
    write_file(local.path(), "local.md", "initial\n");

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );
    assert_eq!(
        manager.setup_git_sync().unwrap().outcome,
        crate::sync::types::GitSyncOutcome::Success
    );

    let repo = git2::Repository::open(local.path()).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch("xnote/local-bootstrap/stale", &head, false).unwrap();
    repo.branch("xnote/local-sync/stale", &head, false).unwrap();
    write_file(local.path(), "local.md", "changed\n");

    let result = manager.perform_sync().unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Success);
    assert!(temporary_branch_names(&repo).is_empty());
}

#[test]
fn manual_sync_blocks_when_pending_state_exists() {
    let local = init_repo_dir();
    let pending = crate::sync::types::PendingSyncState {
        transaction_id: "pending".to_string(),
        transaction_type: crate::sync::types::SyncTransactionType::Sync,
        phase: crate::sync::types::SyncPhase::Conflicted,
        target_branch: "main".to_string(),
        temporary_branch: "xnote/local-sync/pending".to_string(),
        remote_url: "https://example.invalid/repo.git".to_string(),
        pre_transaction_head: None,
        conflicts: vec![],
    };
    crate::sync::state::write_pending_state(local.path(), &pending).unwrap();

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(std::path::Path::new("/tmp/remote.git"), "main"),
    );
    let result = manager.perform_sync().unwrap();

    assert_eq!(result.outcome, crate::sync::types::GitSyncOutcome::Blocked);
}

#[test]
fn manual_sync_fails_when_remote_cannot_be_fetched() {
    let local = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    git2::Repository::init_bare(remote.path()).unwrap();
    write_file(local.path(), "local.md", "initial\n");

    let manager = crate::sync::GitSyncManager::new(
        local.path().to_path_buf(),
        basic_config(remote.path(), "main"),
    );
    assert_eq!(
        manager.setup_git_sync().unwrap().outcome,
        crate::sync::types::GitSyncOutcome::Success
    );

    drop(remote);
    let result = manager.perform_sync();

    assert!(result.is_err());
}
