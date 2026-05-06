use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncTransactionType {
    Setup,
    Sync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhase {
    TestingConnection,
    PreparingLocalSnapshot,
    FetchingRemote,
    UpdatingTrackingBranch,
    ApplyingLocalSnapshot,
    Conflicted,
    Pushing,
    Completed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitConflictStatus {
    BothModified,
    LocalDeleted,
    RemoteDeleted,
    BothAdded,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitConflictFile {
    pub file_path: String,
    pub status: GitConflictStatus,
    pub local_content: Option<String>,
    pub remote_content: Option<String>,
    pub is_binary: bool,
    pub diff_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingSyncState {
    pub transaction_id: String,
    pub transaction_type: SyncTransactionType,
    pub phase: SyncPhase,
    pub target_branch: String,
    pub temporary_branch: String,
    pub remote_url: String,
    pub pre_transaction_head: Option<String>,
    pub conflicts: Vec<GitConflictFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitSetupAction {
    UseExistingRepository,
    InitializeRepository,
    UpdateRemote,
    CreateRemoteBranch,
    ContinuePendingTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitConnectionTestResult {
    pub success: bool,
    pub repository_reachable: bool,
    pub auth_success: bool,
    pub default_branch: Option<String>,
    pub target_branch: String,
    pub target_branch_exists: bool,
    pub will_create_branch: bool,
    pub data_dir_has_git: bool,
    pub data_dir_has_uncommitted_content: bool,
    pub remote_mismatch: bool,
    pub pending_transaction: Option<PendingSyncState>,
    pub actions: Vec<GitSetupAction>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitSyncOutcome {
    Success,
    Conflicted,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitSyncTransactionResult {
    pub outcome: GitSyncOutcome,
    pub message: String,
    pub target_branch: Option<String>,
    pub temporary_branch: Option<String>,
    pub pushed: bool,
    pub conflicts: Vec<GitConflictFile>,
    pub pending: Option<PendingSyncState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedConflictFile {
    pub file_path: String,
    pub final_content: Option<String>,
}
