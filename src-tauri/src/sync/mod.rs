use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use git2::{Repository, Signature, DiffOptions, RemoteCallbacks, Cred, PushOptions};
use std::path::PathBuf;
use chrono;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_sync: Option<String>,
    pub has_conflicts: bool,
    pub local_changes: usize,
    pub remote_changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDiff {
    pub file_path: String,
    pub status: String, // "added", "modified", "deleted"
    pub local_content: Option<String>,
    pub remote_content: Option<String>,
    pub diff_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub conflicts: Option<Vec<SyncDiff>>,
    pub changes_pushed: usize,
    pub changes_pulled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommit {
    pub id: String,
    pub title: String,
    pub time: String,
    pub author: String,
}

pub struct GitSyncManager {
    repo_path: PathBuf,
    config: crate::config::GitSyncConfig,
}

impl GitSyncManager {
    pub fn new(repo_path: PathBuf, config: crate::config::GitSyncConfig) -> Self {
        Self { repo_path, config }
    }

    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;

        let head = repo.head()?;
        let tree = head.peel_to_tree()?;
        
        // Check for local changes
        let mut diff_opts = DiffOptions::new();
        let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_opts))?;
        let local_changes = diff.deltas().len();

        // Get remote changes (this will fetch automatically)
        let remote_changes = self.count_remote_changes(&repo).unwrap_or(0);

        // Check if there are local commits that haven't been pushed
        if let Ok(unpushed_commits) = self.count_unpushed_commits(&repo) {
            if unpushed_commits > 0 {
                println!("📤 Found {} unpushed commits, attempting to push...", unpushed_commits);
                if let Err(e) = self.push_to_remote() {
                    println!("❌ Auto-push failed: {}", e);
                } else {
                    println!("✅ Auto-push completed successfully");
                }
            }
        }

        Ok(SyncStatus {
            is_syncing: false,
            last_sync: self.get_last_sync_time(),
            has_conflicts: false,
            local_changes,
            remote_changes,
        })
    }

    pub fn get_local_changes(&self) -> Result<Vec<SyncDiff>> {
        println!("🔍 Checking for local changes in: {}", self.repo_path.display());
        
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        // Check for untracked files
        let mut status_opts = git2::StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);
        status_opts.recurse_untracked_dirs(true);
        
        let statuses = repo.statuses(Some(&mut status_opts))?;
        let mut changes = Vec::new();
        
        println!("📁 Found {} status entries", statuses.len());
        
        for entry in statuses.iter() {
            let file_path = entry.path().unwrap_or("unknown").to_string();
            let status_flags = entry.status();
            
            println!("🔍 Processing file: {} with status: {:?}", file_path, status_flags);
            
            let (status, diff_content) = if status_flags.contains(git2::Status::WT_NEW) {
                println!("📝 Untracked file: {}", file_path);
                ("added", self.get_file_diff(&file_path, true)?)
            } else if status_flags.contains(git2::Status::INDEX_NEW) {
                println!("📝 New file in index: {}", file_path);
                ("added", self.get_file_diff(&file_path, true)?)
            } else if status_flags.contains(git2::Status::WT_MODIFIED) {
                println!("📝 Modified file: {}", file_path);
                ("modified", self.get_file_diff(&file_path, false)?)
            } else if status_flags.contains(git2::Status::INDEX_MODIFIED) {
                println!("📝 Modified file in index: {}", file_path);
                ("modified", self.get_file_diff(&file_path, false)?)
            } else if status_flags.contains(git2::Status::WT_DELETED) {
                println!("📝 Deleted file: {}", file_path);
                ("deleted", format!("File deleted: {}", file_path))
            } else {
                println!("⏭️ Skipping file with status: {:?}", status_flags);
                continue; // Skip other status types
            };
            
            println!("✅ Adding {} file to changes: {}", status, file_path);
            
            changes.push(SyncDiff {
                file_path: file_path.clone(),
                status: status.to_string(),
                local_content: self.read_file_content(&file_path).ok(),
                remote_content: None,
                diff_content,
            });
        }
        
        // If no changes found through git status, check if there are untracked files manually
        if changes.is_empty() {
            println!("🔍 No changes found via git status, checking for untracked files manually...");
            self.check_untracked_files_manually(&mut changes)?;
        }
        
        println!("✅ Found {} local changes", changes.len());
        Ok(changes)
    }
    
    fn get_file_diff(&self, file_path: &str, is_new: bool) -> Result<String> {
        let full_path = self.repo_path.join(file_path);
        
        if is_new {
            // For new files, show the entire content as added
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let lines: Vec<&str> = content.lines().collect();
                let mut diff = format!("--- /dev/null\n+++ {}\n@@ -0,0 +1,{} @@\n", file_path, lines.len());
                for line in lines.iter().take(50) { // Limit to first 50 lines
                    diff.push_str(&format!("+{}\n", line));
                }
                if lines.len() > 50 {
                    diff.push_str(&format!("... and {} more lines\n", lines.len() - 50));
                }
                Ok(diff)
            } else {
                Ok(format!("New file: {}", file_path))
            }
        } else {
            // For modified files, try to generate a proper diff
            Ok(format!("Modified file: {} (detailed diff not implemented yet)", file_path))
        }
    }
    
    fn read_file_content(&self, file_path: &str) -> Result<String> {
        let full_path = self.repo_path.join(file_path);
        std::fs::read_to_string(full_path).context("Failed to read file content")
    }

    pub fn commit_changes(&self, message: &str) -> Result<()> {
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;

        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        let signature = Signature::now("XNote User", "user@xnote.local")?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(())
    }


    fn get_last_sync_time(&self) -> Option<String> {
        // In a real implementation, you'd store this in config or git notes
        None
    }
    
    pub fn get_remote_commits(&self) -> Result<Vec<RemoteCommit>> {
        println!("🔍 Getting remote commits...");
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        // Fetch first to get latest remote state
        if let Err(e) = self.fetch_from_remote(&repo) {
            println!("⚠️ Failed to fetch from remote: {}", e);
            return Ok(vec![]);
        }
        
        let local_branch = repo.head()?.peel_to_commit()?;
        let remote_branch_name = format!("origin/{}", self.config.branch);
        
        let mut commits = Vec::new();
        
        match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
            Ok(remote_branch) => {
                let remote_commit = remote_branch.get().peel_to_commit()?;
                let (_ahead, behind) = repo.graph_ahead_behind(local_branch.id(), remote_commit.id())?;
                
                if behind > 0 {
                    // Walk through remote commits that are not in local branch
                    let mut revwalk = repo.revwalk()?;
                    revwalk.push(remote_commit.id())?;
                    revwalk.hide(local_branch.id())?;
                    
                    for commit_id in revwalk.take(10) { // Limit to 10 commits
                        let commit_id = commit_id?;
                        let commit = repo.find_commit(commit_id)?;
                        
                        // Convert git time to ISO string
                        let commit_time = commit.time();
                        let timestamp = commit_time.seconds();
                        let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
                            .unwrap_or_else(|| chrono::Utc::now());
                        
                        commits.push(RemoteCommit {
                            id: commit.id().to_string(),
                            title: commit.summary().unwrap_or("No message").to_string(),
                            time: datetime.to_rfc3339(),
                            author: commit.author().name().unwrap_or("Unknown").to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                println!("❌ Remote branch '{}' not found: {}", remote_branch_name, e);
            }
        }
        
        println!("📥 Found {} remote commits", commits.len());
        Ok(commits)
    }

    pub fn initialize_repository(&self) -> Result<()> {
        log::info!("Initializing repository at: {}", self.repo_path.display());
        println!("🔄 Initializing repository at: {}", self.repo_path.display());
        println!("📁 This should be the data_directory where notes are stored");
        
        if !self.repo_path.exists() {
            log::info!("Creating data directory: {}", self.repo_path.display());
            println!("📁 Creating data directory: {}", self.repo_path.display());
            std::fs::create_dir_all(&self.repo_path)?;
        } else {
            println!("📁 Data directory already exists: {}", self.repo_path.display());
            // List contents of the directory
            if let Ok(entries) = std::fs::read_dir(&self.repo_path) {
                let mut file_count = 0;
                for entry in entries {
                    if let Ok(entry) = entry {
                        file_count += 1;
                        if file_count <= 10 { // Show first 10 files
                            println!("  📄 {}", entry.file_name().to_string_lossy());
                        }
                    }
                }
                if file_count > 10 {
                    println!("  ... and {} more files", file_count - 10);
                }
                println!("📊 Total files in data directory: {}", file_count);
            }
        }

        let git_dir = self.repo_path.join(".git");
        if !git_dir.exists() {
            if !self.config.repository_url.is_empty() {
                log::info!("Cloning repository from: {}", self.config.repository_url);
                self.clone_to_existing_directory()
                    .context("Failed to clone repository to existing directory")?;
            } else {
                log::info!("Initializing new local repository");
                Repository::init(&self.repo_path)
                    .context("Failed to initialize repository")?;
            }
        } else {
            log::info!("Repository already exists at: {}", git_dir.display());
        }

        Ok(())
    }

    fn clone_to_existing_directory(&self) -> Result<()> {
        use std::fs;
        
        log::info!("Starting clone to existing directory process");
        
        // Create a temporary directory for cloning
        let temp_dir = self.repo_path.join(".tmp_clone");
        log::info!("Using temporary directory: {}", temp_dir.display());
        
        if temp_dir.exists() {
            log::info!("Removing existing temporary directory");
            fs::remove_dir_all(&temp_dir)
                .context("Failed to remove existing temp directory")?;
        }
        
        // Clone to temporary directory with authentication
        log::info!("Cloning repository with authentication...");
        log::info!("Repository URL: {}", self.config.repository_url);
        log::info!("Auth type: {}", self.config.auth_type);
        
        println!("🔄 Cloning repository: {}", self.config.repository_url);
        println!("🔐 Auth type: {}", self.config.auth_type);
        
        let _repo = self.clone_with_auth(&self.config.repository_url, &temp_dir)
            .context("Failed to clone repository with authentication")?;
        
        log::info!("Clone completed successfully");
        
        // Move .git directory to data directory
        let git_source = temp_dir.join(".git");
        let git_target = self.repo_path.join(".git");
        
        log::info!("Moving .git directory from {} to {}", git_source.display(), git_target.display());
        
        if git_source.exists() {
            if git_target.exists() {
                log::info!("Removing existing .git directory");
                fs::remove_dir_all(&git_target)
                    .context("Failed to remove existing .git directory")?;
            }
            self.move_directory(&git_source, &git_target)
                .context("Failed to move .git directory")?;
            log::info!(".git directory moved successfully");
        } else {
            let error_msg = "No .git directory found in cloned repository";
            println!("❌ {}", error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }
        
        // Move other files from temp directory, but don't overwrite existing files
        log::info!("Moving other files from temporary directory");
        for entry in fs::read_dir(&temp_dir)
            .context("Failed to read temporary directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let file_name = entry.file_name();
            
            // Skip .git directory as we already moved it
            if file_name == ".git" {
                continue;
            }
            
            let source_path = entry.path();
            let target_path = self.repo_path.join(&file_name);
            
            log::info!("Processing file: {}", file_name.to_string_lossy());
            
            // Only move if target doesn't exist (don't overwrite existing files)
            if !target_path.exists() {
                if source_path.is_dir() {
                    log::info!("Moving directory: {}", file_name.to_string_lossy());
                    self.move_directory(&source_path, &target_path)
                        .with_context(|| format!("Failed to move directory: {}", file_name.to_string_lossy()))?;
                } else {
                    log::info!("Copying file: {}", file_name.to_string_lossy());
                    fs::copy(&source_path, &target_path)
                        .with_context(|| format!("Failed to copy file: {}", file_name.to_string_lossy()))?;
                }
            } else {
                log::info!("Skipping existing file: {}", file_name.to_string_lossy());
            }
        }
        
        // Clean up temporary directory
        log::info!("Cleaning up temporary directory");
        fs::remove_dir_all(&temp_dir)
            .context("Failed to clean up temporary directory")?;
        
        log::info!("Clone to existing directory completed successfully");
        
        Ok(())
    }
    
    fn move_directory(&self, source: &std::path::Path, target: &std::path::Path) -> Result<()> {
        use std::fs;
        
        log::debug!("Moving directory from {} to {}", source.display(), target.display());
        
        if source.is_dir() {
            fs::create_dir_all(target)
                .with_context(|| format!("Failed to create target directory: {}", target.display()))?;
            
            for entry in fs::read_dir(source)
                .with_context(|| format!("Failed to read source directory: {}", source.display()))? {
                let entry = entry.context("Failed to read directory entry")?;
                let source_path = entry.path();
                let target_path = target.join(entry.file_name());
                
                if source_path.is_dir() {
                    self.move_directory(&source_path, &target_path)?;
                } else {
                    fs::copy(&source_path, &target_path)
                        .with_context(|| format!("Failed to copy file from {} to {}", 
                            source_path.display(), target_path.display()))?;
                }
            }
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
            }
            fs::copy(source, target)
                .with_context(|| format!("Failed to copy file from {} to {}", 
                    source.display(), target.display()))?;
        }
        
        Ok(())
    }
    
    fn clone_with_auth(&self, url: &str, path: &std::path::Path) -> Result<Repository> {
        log::info!("Setting up clone with authentication");
        let mut builder = git2::build::RepoBuilder::new();
        
        // Set up authentication callbacks
        let mut callbacks = RemoteCallbacks::new();
        let config = self.config.clone();
        
        callbacks.credentials(move |url, username_from_url, allowed_types| {
            log::info!("Authentication callback triggered");
            log::info!("URL: {}", url);
            log::info!("Username from URL: {:?}", username_from_url);
            log::info!("Allowed types: {:?}", allowed_types);
            log::info!("Config auth type: {}", config.auth_type);
            
            println!("🔐 Authentication callback for: {}", url);
            println!("👤 Username: {:?}", username_from_url);
            println!("🔑 Auth type: {}", config.auth_type);
            
            let result = match config.auth_type.as_str() {
                "basic" => {
                    log::info!("Using basic authentication");
                    if let (Some(username), Some(password)) = (&config.username, &config.password) {
                        log::info!("Using provided username: {}", username);
                        Cred::userpass_plaintext(username, password)
                    } else {
                        log::warn!("Basic auth selected but username/password not provided");
                        Cred::default()
                    }
                }
                "ssh" => {
                    log::info!("Using SSH authentication");
                    let username = username_from_url.unwrap_or("git");
                    log::info!("SSH username: {}", username);
                    
                    if let Some(ssh_key_path) = &config.ssh_key_path {
                        if !ssh_key_path.is_empty() {
                            log::info!("Using specified SSH key: {}", ssh_key_path);
                            println!("🔑 Using SSH key: {}", ssh_key_path);
                            
                            // Expand ~ to home directory
                            let expanded_path = if ssh_key_path.starts_with("~/") {
                                if let Some(home_dir) = dirs::home_dir() {
                                    home_dir.join(&ssh_key_path[2..])
                                } else {
                                    std::path::PathBuf::from(ssh_key_path)
                                }
                            } else {
                                std::path::PathBuf::from(ssh_key_path)
                            };
                            
                            println!("🔍 Expanded SSH key path: {}", expanded_path.display());
                            
                            if expanded_path.exists() {
                                println!("✅ SSH key file exists");
                                Cred::ssh_key(username, None, &expanded_path, None)
                            } else {
                                println!("❌ SSH key file not found: {}", expanded_path.display());
                                log::warn!("SSH key file not found, falling back to SSH agent");
                                Cred::ssh_key_from_agent(username)
                            }
                        } else {
                            log::info!("SSH key path is empty, using SSH agent");
                            println!("🔑 Using SSH agent (empty key path)");
                            Cred::ssh_key_from_agent(username)
                        }
                    } else {
                        log::info!("Using SSH agent or default SSH key");
                        println!("🔑 Using SSH agent (no key specified)");
                        Cred::ssh_key_from_agent(username)
                    }
                }
                _ => {
                    log::info!("Using default authentication");
                    Cred::default()
                }
            };
            
            match &result {
                Ok(_) => {
                    log::info!("Authentication credentials created successfully");
                    println!("✅ Authentication credentials created");
                },
                Err(e) => {
                    log::error!("Failed to create authentication credentials: {}", e);
                    println!("❌ Authentication failed: {}", e);
                },
            }
            
            result
        });
        
        // Add progress callback
        callbacks.update_tips(|refname, a, b| {
            log::info!("Update tips: {} {} -> {}", refname, a, b);
            true
        });
        
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);
        
        log::info!("Starting git clone operation...");
        let result = builder.clone(url, path);
        
        match &result {
            Ok(_) => {
                log::info!("Git clone completed successfully");
                println!("✅ Git clone completed successfully");
            },
            Err(e) => {
                log::error!("Git clone failed: {}", e);
                println!("❌ Git clone failed: {}", e);
            },
        }
        
        result.context("Failed to clone with authentication")
    }
    
    pub fn perform_sync(&self) -> Result<SyncResult> {
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;

        // Fetch from remote
        self.fetch_from_remote(&repo)?;
        
        // Check for conflicts and perform rebase
        match self.perform_rebase(&repo) {
            Ok(_) => {
                // Check if there are local changes to push
                let local_changes = self.count_local_changes(&repo)?;
                if local_changes > 0 {
                    self.push_to_remote()?;
                }
                
                Ok(SyncResult {
                    success: true,
                    message: "同步完成".to_string(),
                    conflicts: None,
                    changes_pushed: local_changes,
                    changes_pulled: 0, // TODO: count pulled changes
                })
            }
            Err(e) => {
                // Check if it's a conflict error
                if e.to_string().contains("conflict") {
                    let conflicts = self.get_conflicts(&repo)?;
                    Ok(SyncResult {
                        success: false,
                        message: "检测到冲突，需要手动解决".to_string(),
                        conflicts: Some(conflicts),
                        changes_pushed: 0,
                        changes_pulled: 0,
                    })
                } else {
                    Err(e)
                }
            }
        }
    }
    
    fn fetch_from_remote(&self, repo: &Repository) -> Result<()> {
        println!("🔄 Starting fetch from remote...");
        
        let mut remote = repo.find_remote("origin")
            .context("Failed to find origin remote")?;
        
        let mut callbacks = RemoteCallbacks::new();
        let config = self.config.clone();
        
        callbacks.credentials(move |url, username_from_url, _allowed_types| {
            println!("🔐 Fetch authentication callback for: {}", url);
            
            match config.auth_type.as_str() {
                "basic" => {
                    if let (Some(username), Some(password)) = (&config.username, &config.password) {
                        println!("🔑 Using basic auth for fetch");
                        Cred::userpass_plaintext(username, password)
                    } else {
                        println!("🔑 Using default auth for fetch");
                        Cred::default()
                    }
                }
                "ssh" => {
                    let username = username_from_url.unwrap_or("git");
                    println!("🔑 Using SSH auth for fetch, username: {}", username);
                    
                    if let Some(ssh_key_path) = &config.ssh_key_path {
                        if !ssh_key_path.is_empty() {
                            // Expand ~ to home directory
                            let expanded_path = if ssh_key_path.starts_with("~/") {
                                if let Some(home_dir) = dirs::home_dir() {
                                    home_dir.join(&ssh_key_path[2..])
                                } else {
                                    std::path::PathBuf::from(ssh_key_path)
                                }
                            } else {
                                std::path::PathBuf::from(ssh_key_path)
                            };
                            
                            println!("🔑 Using SSH key for fetch: {}", expanded_path.display());
                            Cred::ssh_key(username, None, &expanded_path, None)
                        } else {
                            println!("🔑 Using SSH agent for fetch (empty key path)");
                            Cred::ssh_key_from_agent(username)
                        }
                    } else {
                        println!("🔑 Using SSH agent for fetch (no key specified)");
                        Cred::ssh_key_from_agent(username)
                    }
                }
                _ => {
                    println!("🔑 Using default auth for fetch");
                    Cred::default()
                }
            }
        });
        
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        println!("🔄 Executing fetch operation...");
        let result = remote.fetch(&[&self.config.branch], Some(&mut fetch_options), None);
        
        match &result {
            Ok(_) => println!("✅ Fetch completed successfully"),
            Err(e) => println!("❌ Fetch failed: {}", e),
        }
        
        result.context("Failed to fetch from remote")?;
        
        Ok(())
    }
    
    fn perform_rebase(&self, repo: &Repository) -> Result<()> {
        // 这个方法保留用于向后兼容，但现在调用新的实现
        let remote_branch_name = format!("origin/{}", self.config.branch);
        let remote_branch = repo.find_branch(&remote_branch_name, git2::BranchType::Remote)?;
        let remote_commit = remote_branch.get().peel_to_commit()?;
        
        self.perform_rebase_operation(repo, &remote_commit)
    }
    
    // 新的 rebase 实现
    fn perform_rebase_operation(&self, repo: &Repository, onto_commit: &git2::Commit) -> Result<()> {
        println!("🔄 Starting rebase operation...");
        
        // 获取当前分支
        let branch = repo.head()?;
        let branch_name = branch.shorthand().unwrap_or("HEAD");
        
        // 获取当前提交
        let current_commit = repo.head()?.peel_to_commit()?;
        
        // 找到共同祖先
        let merge_base = repo.merge_base(current_commit.id(), onto_commit.id())?;
        let base_commit = repo.find_commit(merge_base)?;
        
        println!("📍 Rebase base: {}", base_commit.id());
        println!("📍 Rebase onto: {}", onto_commit.id());
        
        // 收集需要重放的提交
        let mut commits_to_replay = Vec::new();
        let mut revwalk = repo.revwalk()?;
        revwalk.push(current_commit.id())?;
        revwalk.hide(base_commit.id())?;
        
        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = repo.find_commit(commit_id)?;
            commits_to_replay.push(commit);
        }
        
        // 反转顺序，从最早的提交开始重放
        commits_to_replay.reverse();
        
        println!("🔄 Replaying {} commits...", commits_to_replay.len());
        
        // 将 HEAD 移动到目标提交
        repo.set_head_detached(onto_commit.id())?;
        
        // 更新工作目录
        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();
        repo.checkout_head(Some(&mut checkout_opts))?;
        
        let mut current_head = onto_commit.id();
        
        // 重放每个提交
        for (i, commit) in commits_to_replay.iter().enumerate() {
            println!("🔄 Replaying commit {}/{}: {}", i + 1, commits_to_replay.len(), 
                     commit.summary().unwrap_or("No message"));
            
            // 获取提交的树
            let commit_tree = commit.tree()?;
            
            // 获取父提交的树
            let parent_tree = if commit.parent_count() > 0 {
                commit.parent(0)?.tree()?
            } else {
                // 如果是根提交，创建一个空树
                let empty_tree_id = repo.treebuilder(None)?.write()?;
                repo.find_tree(empty_tree_id)?
            };
            
            // 获取当前 HEAD 的树
            let head_commit = repo.find_commit(current_head)?;
            let head_tree = head_commit.tree()?;
            
            // 执行三方合并
            let mut index = repo.merge_trees(&parent_tree, &head_tree, &commit_tree, None)?;
            
            // 检查冲突
            if index.has_conflicts() {
                println!("⚠️ Rebase conflict detected at commit: {}", commit.id());
                return Err(anyhow::anyhow!("Rebase conflict detected. Please resolve conflicts manually."));
            }
            
            // 创建新的提交
            let tree_id = index.write_tree_to(repo)?;
            let tree = repo.find_tree(tree_id)?;
            
            let signature = commit.author();
            let message = commit.message().unwrap_or("No message");
            
            let new_commit_id = repo.commit(
                None, // 不更新引用，稍后手动更新
                &signature,
                &signature,
                message,
                &tree,
                &[&head_commit],
            )?;
            
            current_head = new_commit_id;
        }
        
        // 更新分支引用
        let reference_name = format!("refs/heads/{}", branch_name);
        repo.reference(&reference_name, current_head, true, "Rebase completed")?;
        
        // 将 HEAD 指向分支
        repo.set_head(&reference_name)?;
        
        // 最终更新工作目录
        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();
        repo.checkout_head(Some(&mut checkout_opts))?;
        
        println!("✅ Rebase operation completed successfully");
        Ok(())
    }
    
    pub fn push_to_remote(&self) -> Result<()> {
        println!("🚀 Starting push to remote repository...");
        
        // 尝试推送，如果失败则自动 rebase 后重试
        for attempt in 1..=3 {
            println!("📤 Push attempt {}/3", attempt);
            
            match self.try_push_to_remote() {
                Ok(_) => {
                    println!("✅ Push to remote completed successfully on attempt {}", attempt);
                    return Ok(());
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    println!("❌ Push attempt {} failed: {}", attempt, error_msg);
                    
                    // 检查是否是非快进错误
                    if error_msg.contains("NotFastForward") || error_msg.contains("non-fastforwardable") {
                        if attempt < 3 {
                            println!("🔄 Non-fast-forward detected, attempting rebase and retry...");
                            
                            // 尝试 rebase 后重试
                            if let Err(rebase_err) = self.pull_rebase() {
                                println!("❌ Rebase failed: {}", rebase_err);
                                return Err(anyhow::anyhow!("Push failed and rebase failed: {}", rebase_err));
                            }
                            
                            println!("✅ Rebase completed, retrying push...");
                            continue;
                        }
                    }
                    
                    // 如果是最后一次尝试或者不是快进错误，直接返回错误
                    return Err(e);
                }
            }
        }
        
        Err(anyhow::anyhow!("Push failed after 3 attempts"))
    }
    
    // 实际的推送实现
    fn try_push_to_remote(&self) -> Result<()> {
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        let mut remote = repo.find_remote("origin")
            .context("Failed to find origin remote")?;
        
        let mut callbacks = RemoteCallbacks::new();
        let config = self.config.clone();
        
        callbacks.credentials(move |url, username_from_url, _allowed_types| {
            println!("🔐 Push authentication callback for: {}", url);
            
            match config.auth_type.as_str() {
                "basic" => {
                    if let (Some(username), Some(password)) = (&config.username, &config.password) {
                        println!("🔑 Using basic auth for push");
                        Cred::userpass_plaintext(username, password)
                    } else {
                        println!("🔑 Using default auth for push");
                        Cred::default()
                    }
                }
                "ssh" => {
                    let username = username_from_url.unwrap_or("git");
                    println!("🔑 Using SSH auth for push, username: {}", username);
                    
                    if let Some(ssh_key_path) = &config.ssh_key_path {
                        if !ssh_key_path.is_empty() {
                            // Expand ~ to home directory
                            let expanded_path = if ssh_key_path.starts_with("~/") {
                                if let Some(home_dir) = dirs::home_dir() {
                                    home_dir.join(&ssh_key_path[2..])
                                } else {
                                    std::path::PathBuf::from(ssh_key_path)
                                }
                            } else {
                                std::path::PathBuf::from(ssh_key_path)
                            };
                            
                            println!("🔑 Using SSH key for push: {}", expanded_path.display());
                            Cred::ssh_key(username, None, &expanded_path, None)
                        } else {
                            println!("🔑 Using SSH agent for push (empty key path)");
                            Cred::ssh_key_from_agent(username)
                        }
                    } else {
                        println!("🔑 Using SSH agent for push (no key specified)");
                        Cred::ssh_key_from_agent(username)
                    }
                }
                _ => {
                    println!("🔑 Using default auth for push");
                    Cred::default()
                }
            }
        });
        
        // Add progress callback
        callbacks.push_update_reference(|refname, status| {
            if let Some(msg) = status {
                println!("❌ Push failed for ref {}: {}", refname, msg);
            } else {
                println!("✅ Push succeeded for ref: {}", refname);
            }
            Ok(())
        });
        
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        let refspec = format!("refs/heads/{}:refs/heads/{}", self.config.branch, self.config.branch);
        println!("📤 Pushing refspec: {}", refspec);
        
        let result = remote.push(&[&refspec], Some(&mut push_options));
        
        result.context("Failed to push to remote")
    }
    
    fn count_local_changes(&self, repo: &Repository) -> Result<usize> {
        let head = repo.head()?;
        let tree = head.peel_to_tree()?;
        
        let mut diff_opts = DiffOptions::new();
        let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_opts))?;
        
        Ok(diff.deltas().len())
    }
    
    fn get_conflicts(&self, _repo: &Repository) -> Result<Vec<SyncDiff>> {
        // TODO: Implement conflict detection
        Ok(vec![])
    }
    
    fn count_remote_changes(&self, repo: &Repository) -> Result<usize> {
        println!("🔍 Checking remote changes...");
        
        // Always fetch first to get latest remote state
        if let Err(e) = self.fetch_from_remote(repo) {
            println!("⚠️ Failed to fetch from remote: {}", e);
            return Ok(0);
        }
        
        let local_branch = repo.head()?.peel_to_commit()?;
        let remote_branch_name = format!("origin/{}", self.config.branch);
        
        match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
            Ok(remote_branch) => {
                let remote_commit = remote_branch.get().peel_to_commit()?;
                let (ahead, behind) = repo.graph_ahead_behind(local_branch.id(), remote_commit.id())?;
                
                println!("📊 Local branch is {} commits ahead, {} commits behind remote", ahead, behind);
                
                if behind > 0 {
                    println!("📥 Found {} remote changes that need to be pulled", behind);
                }
                
                Ok(behind)
            }
            Err(e) => {
                println!("❌ Remote branch '{}' not found: {}", remote_branch_name, e);
                Ok(0)
            }
        }
    }
    
    fn count_unpushed_commits(&self, repo: &Repository) -> Result<usize> {
        let local_branch = repo.head()?.peel_to_commit()?;
        let remote_branch_name = format!("origin/{}", self.config.branch);
        
        match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
            Ok(remote_branch) => {
                let remote_commit = remote_branch.get().peel_to_commit()?;
                let (ahead, _behind) = repo.graph_ahead_behind(local_branch.id(), remote_commit.id())?;
                Ok(ahead)
            }
            Err(_) => {
                // If remote branch doesn't exist, count all local commits
                let mut revwalk = repo.revwalk()?;
                revwalk.push_head()?;
                Ok(revwalk.count())
            }
        }
    }
    
    // 新的Git操作方法
    pub fn stash_changes(&self) -> Result<()> {
        println!("📦 Stashing local changes...");
        let mut repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        let signature = Signature::now("XNote User", "user@xnote.local")?;
        
        repo.stash_save(&signature, "XNote auto-stash before sync", Some(git2::StashFlags::DEFAULT))
            .context("Failed to stash changes")?;
        
        println!("✅ Local changes stashed successfully");
        Ok(())
    }
    
    pub fn stash_pop(&self) -> Result<()> {
        println!("📦 Restoring stashed changes...");
        let mut repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        repo.stash_pop(0, None)
            .context("Failed to pop stashed changes")?;
        
        println!("✅ Stashed changes restored successfully");
        Ok(())
    }
    
    pub fn pull_from_remote(&self) -> Result<()> {
        println!("⬇️ Pulling from remote...");
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        // 1. 首先 fetch 远程变更
        self.fetch_from_remote(&repo)
            .context("Failed to fetch from remote")?;
        
        // 2. 获取当前分支和远程分支
        let local_branch = repo.head()?.peel_to_commit()?;
        let remote_branch_name = format!("origin/{}", self.config.branch);
        
        match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
            Ok(remote_branch) => {
                let remote_commit = remote_branch.get().peel_to_commit()?;
                let (ahead, behind) = repo.graph_ahead_behind(local_branch.id(), remote_commit.id())?;
                
                println!("📊 Local is {} ahead, {} behind remote", ahead, behind);
                
                if behind > 0 {
                    // 3. 执行 fast-forward merge 或 merge
                    println!("🔄 Merging remote changes...");
                    self.merge_remote_changes(&repo, &remote_commit)?;
                    println!("✅ Successfully merged {} remote commits", behind);
                } else {
                    println!("✅ Local branch is already up to date");
                }
            }
            Err(e) => {
                println!("⚠️ Remote branch '{}' not found: {}", remote_branch_name, e);
                return Err(anyhow::anyhow!("Remote branch not found: {}", remote_branch_name));
            }
        }
        
        println!("✅ Pull completed successfully");
        Ok(())
    }
    
    pub fn pull_rebase(&self) -> Result<()> {
        println!("🔄 Pulling with rebase...");
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        // 1. 首先 fetch 远程变更
        self.fetch_from_remote(&repo)?;
        
        // 2. 获取当前分支和远程分支
        let local_branch = repo.head()?.peel_to_commit()?;
        let remote_branch_name = format!("origin/{}", self.config.branch);
        
        match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
            Ok(remote_branch) => {
                let remote_commit = remote_branch.get().peel_to_commit()?;
                let (ahead, behind) = repo.graph_ahead_behind(local_branch.id(), remote_commit.id())?;
                
                println!("📊 Local is {} ahead, {} behind remote", ahead, behind);
                
                if behind > 0 {
                    if ahead > 0 {
                        // 需要 rebase
                        println!("🔄 Rebasing local commits onto remote...");
                        self.perform_rebase_operation(&repo, &remote_commit)?;
                        println!("✅ Successfully rebased {} local commits onto {} remote commits", ahead, behind);
                    } else {
                        // 只需要 fast-forward
                        println!("⚡ Fast-forwarding to remote...");
                        self.fast_forward_merge(&repo, &remote_commit)?;
                    }
                } else {
                    println!("✅ Local branch is already up to date");
                }
            }
            Err(e) => {
                println!("⚠️ Remote branch '{}' not found: {}", remote_branch_name, e);
                return Err(anyhow::anyhow!("Remote branch not found: {}", remote_branch_name));
            }
        }
        
        println!("✅ Pull rebase completed successfully");
        Ok(())
    }
    
    fn check_untracked_files_manually(&self, changes: &mut Vec<SyncDiff>) -> Result<()> {
        println!("🔍 Manually checking for .md files in data directory...");
        
        let entries = std::fs::read_dir(&self.repo_path)?;
        let mut md_files = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "md" {
                        if let Some(file_name) = path.file_name() {
                            md_files.push(file_name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        
        println!("📁 Found {} .md files in directory", md_files.len());
        
        // Check if these files are tracked by git
        let repo = Repository::open(&self.repo_path)?;
        
        for file_name in md_files {
            // Check if file is in git index
            let mut is_tracked = false;
            if let Ok(index) = repo.index() {
                is_tracked = index.get_path(std::path::Path::new(&file_name), 0).is_some();
            }
            
            if !is_tracked {
                println!("📝 Found untracked .md file: {}", file_name);
                let diff_content = self.get_file_diff(&file_name, true)?;
                
                changes.push(SyncDiff {
                    file_path: file_name.clone(),
                    status: "added".to_string(),
                    local_content: self.read_file_content(&file_name).ok(),
                    remote_content: None,
                    diff_content,
                });
            } else {
                println!("✅ File {} is already tracked", file_name);
            }
        }
        
        println!("✅ Manual check completed, found {} untracked files", changes.len());
        Ok(())
    }
    
    pub fn get_commit_history(&self, limit: usize) -> Result<Vec<RemoteCommit>> {
        println!("🔍 Getting commit history...");
        let repo = Repository::open(&self.repo_path)
            .context("Failed to open repository")?;
        
        let mut commits = Vec::new();
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?; // Sort by time
        
        for commit_id in revwalk.take(limit) {
            let commit_id = commit_id?;
            let commit = repo.find_commit(commit_id)?;
            
            // Convert git time to ISO string
            let commit_time = commit.time();
            let timestamp = commit_time.seconds();
            let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
                .unwrap_or_else(|| chrono::Utc::now());
            
            commits.push(RemoteCommit {
                id: commit.id().to_string(),
                title: commit.summary().unwrap_or("No message").to_string(),
                time: datetime.to_rfc3339(),
                author: commit.author().name().unwrap_or("Unknown").to_string(),
            });
        }
        
        println!("📚 Found {} commits in history", commits.len());
        Ok(commits)
    }
    
    // 新增：合并远程变更到本地
    fn merge_remote_changes(&self, repo: &Repository, remote_commit: &git2::Commit) -> Result<()> {
        println!("🔄 Starting merge of remote changes...");
        
        // 获取当前 HEAD
        let local_commit = repo.head()?.peel_to_commit()?;
        
        // 检查是否可以进行 fast-forward
        let (ahead, _behind) = repo.graph_ahead_behind(local_commit.id(), remote_commit.id())?;
        
        if ahead == 0 {
            // 可以进行 fast-forward
            println!("⚡ Performing fast-forward merge...");
            self.fast_forward_merge(repo, remote_commit)?;
        } else {
            // 需要进行三方合并
            println!("🔀 Performing three-way merge...");
            self.three_way_merge(repo, &local_commit, remote_commit)?;
        }
        
        Ok(())
    }
    
    // Fast-forward 合并
    fn fast_forward_merge(&self, repo: &Repository, target_commit: &git2::Commit) -> Result<()> {
        // 更新 HEAD 引用
        let refname = "HEAD";
        repo.reference(refname, target_commit.id(), true, "Fast-forward merge")?;
        
        // 更新工作目录
        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();
        repo.checkout_head(Some(&mut checkout_opts))?;
        
        println!("✅ Fast-forward merge completed");
        Ok(())
    }
    
    // 三方合并
    fn three_way_merge(&self, repo: &Repository, local_commit: &git2::Commit, remote_commit: &git2::Commit) -> Result<()> {
        // 找到共同祖先
        let merge_base = repo.merge_base(local_commit.id(), remote_commit.id())?;
        let base_commit = repo.find_commit(merge_base)?;
        
        // 获取树对象
        let local_tree = local_commit.tree()?;
        let remote_tree = remote_commit.tree()?;
        let base_tree = base_commit.tree()?;
        
        // 执行三方合并
        let mut index = repo.merge_trees(&base_tree, &local_tree, &remote_tree, None)?;
        
        // 检查是否有冲突
        if index.has_conflicts() {
            println!("⚠️ Merge conflicts detected, need manual resolution");
            return Err(anyhow::anyhow!("Merge conflicts detected. Please resolve conflicts manually."));
        }
        
        // 写入合并结果
        let tree_id = index.write_tree_to(repo)?;
        let tree = repo.find_tree(tree_id)?;
        
        // 创建合并提交
        let signature = Signature::now("XNote User", "user@xnote.local")?;
        let merge_message = format!("Merge remote changes from {}", self.config.branch);
        
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &merge_message,
            &tree,
            &[local_commit, remote_commit],
        )?;
        
        // 更新工作目录
        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();
        repo.checkout_head(Some(&mut checkout_opts))?;
        
        println!("✅ Three-way merge completed");
        Ok(())
    }
}