# Git Sync Design

## Context

XNote already has an initial Git sync implementation. The backend uses `git2` in `src-tauri/src/sync/mod.rs`, and the frontend exposes Git settings in `SettingsPane.vue` plus a sync dialog in `SyncDialog.vue`.

The current implementation is incomplete. Settings can be saved without initializing a usable repository, connection testing is a placeholder, the UI composes low-level Git steps such as stash, pull, commit, rebase, and push, and conflict detection/resolution is not implemented end to end. This makes first-time setup with existing notes, normal bidirectional sync, and interrupted conflict handling fragile.

## Goals

- Keep XNote self-contained by using the existing Rust/libgit2 approach instead of depending on a system `git` CLI.
- Move Git sync orchestration into the backend so the frontend calls high-level operations rather than composing Git commands.
- Support first-time Git setup for empty and non-empty note workspaces.
- Preserve both local and remote note content during first-time setup, using conflict resolution instead of silent overwrite.
- Support manual sync only. XNote should not auto-sync on startup or on a timer in this version.
- Support branch auto-detection when the branch field is empty.
- Create a requested remote branch when the user explicitly configures a branch that does not exist.
- Add real settings-page connection testing and setup feedback.
- Add an internal two-column conflict resolver that lets the user edit local content while viewing remote content.
- Make interrupted sync states resumable or abortable after app restart.
- Avoid logging passwords or tokens.

## Non-Goals

- No automatic background sync.
- No system keyring or credential helper integration in this version.
- No dependency on a user-installed Git executable.
- No full three-way merge editor.
- No line-level conflict editing UI in the first version.
- No automatic selection of local or remote content during conflicts.
- No deletion of `.git`, remotes, or history when sync is disabled.

## User Decisions

- Conflict UI is built into XNote.
- Conflict UI has two columns: local editable content on the left and remote read-only content on the right.
- Conflict markers are shown around conflicted file/content areas, not as a complex inline merge editor.
- Clicking continue uses the left local content as the final file content, even if the user did not fully resolve every semantic conflict.
- First-time setup with local and remote content should merge both sides and surface same-file conflicts.
- Branch may be blank. Blank means use the remote default branch.
- If the user enters a branch that does not exist remotely, XNote creates and pushes that branch.
- Credentials remain in XNote config for simplicity and independence.
- Sync is triggered manually by the user.

## Architecture

### Backend Sync Transaction

`GitSyncManager` should become the owner of the Git state machine. It should expose high-level Tauri commands such as:

- `test_git_connection`
- `setup_git_sync`
- `perform_sync`
- `get_pending_git_sync`
- `continue_git_sync`
- `abort_git_sync`

The frontend should not sequence low-level operations like stash, pull, commit, rebase, and push. Existing low-level commands may remain temporarily for debugging or compatibility, but the primary UI flow should use the high-level commands.

### Frontend Responsibilities

`SettingsPane.vue` should:

- Edit Git configuration.
- Run connection testing.
- Show the expected setup action before saving.
- Call setup when enabling Git sync.
- Enter conflict resolution if setup returns a conflicted state.

`SyncDialog.vue` should:

- Show local/remote/history status.
- Run one manual sync action.
- Show progress and result states returned by the backend.
- Enter conflict resolution if sync returns a conflicted state.
- Offer continue, retry, and abort actions for a paused transaction.

### Sync State File

When setup or sync pauses, XNote should write a lightweight state file under the repository metadata, for example:

```text
.git/xnote-sync-state.json
```

The state file should record:

- Transaction type: setup or sync.
- Target branch.
- Temporary branch.
- Current phase.
- Conflicted files.
- Remote URL involved in the transaction.
- Any commit ids needed to continue or abort.

This allows the app to recover after restart without relying on frontend memory.

## Configuration And Credentials

The existing config shape can remain close to the current `GitSyncConfig`:

- `enabled`
- `repository_url`
- `branch`
- `auth_type`
- `username`
- `password`
- `ssh_key_path`

The `password` field may hold a password or token. This is intentionally simple for this version. The backend and frontend must avoid logging password/token values. Connection errors should report authentication failure without echoing secrets.

Disabling sync only disables XNote's sync UI and behavior. It should not delete `.git`, remove remotes, remove branches, or rewrite history.

## Connection Testing

The settings page "test connection" action should call a real backend command. It should validate:

- Repository URL is present and parseable.
- Authentication data required by the selected auth type is present.
- SSH key path exists when explicitly configured.
- Remote repository can be contacted.
- Authentication succeeds.
- Remote default branch can be detected when the branch field is empty.
- The final branch XNote will follow.
- Whether the final branch already exists remotely.
- Whether saving will create a remote branch.
- Current data directory Git state: no repository, existing repository, existing remote, mismatched remote, local uncommitted content, or paused sync transaction.

The result should be structured so the UI can present clear messages, not just a success/failure string.

## First-Time Setup Flow

When enabling Git sync, the frontend should call `setup_git_sync`. The backend should run one transaction:

1. Validate the config and credentials.
2. Ensure the data directory exists.
3. If no `.git` exists, initialize a repository in the data directory.
4. Create a temporary local branch such as `xnote/local-bootstrap/<timestamp>`.
5. Commit the current local workspace content on the temporary branch. If the repository has no commits, this is the initial commit.
6. Add or update `origin` to the configured repository URL.
7. Determine the final branch:
   - If config branch is empty, use the remote default branch.
   - If config branch is set and exists remotely, follow it.
   - If config branch is set and does not exist remotely, create it from the remote default branch when one exists; otherwise create it from the temporary local branch. Then merge the temporary local branch and push the new branch.
8. Check out the final branch and configure upstream tracking to `origin/<branch>`.
9. Merge the temporary branch content into the final branch.
10. If there are no conflicts, commit the merge result and push.
11. If there are conflicts, write sync state and return a conflicted result to the UI.
12. After success, delete the temporary branch if it is no longer needed.

The setup flow must not silently overwrite local or remote note files. Same-file conflicts should be surfaced to the conflict resolver.

## Manual Sync Flow

When the user manually starts sync, the frontend calls `perform_sync`. The backend should run one transaction:

1. Refuse to start a new sync if a previous sync transaction is paused.
2. Create a temporary branch such as `xnote/local-sync/<timestamp>`.
3. Commit current workspace content to the temporary branch if there are changes.
4. Check out the configured tracking branch.
5. Fetch from `origin`.
6. Apply remote updates to the tracking branch with `pull --rebase` semantics:
   - Fast-forward when possible.
   - Rebase local tracking-branch-only commits onto the fetched remote when needed.
7. Bring the temporary branch content back into the tracking branch.
8. Prefer merge for this local-content step because note sync values content preservation and explainable conflicts more than a strictly linear local history.
9. If there are no conflicts, commit the result and push.
10. If conflicts occur, write sync state and return a conflicted result.
11. After successful push, delete the temporary branch.

If there are no local changes, the sync should still fetch and update the tracking branch, then push only if needed.

## Conflict Resolution

The backend should return conflicted files with enough data for a two-column UI:

- File path.
- Local content.
- Remote content.
- Conflict status: both modified, local deleted, remote deleted, both added, or unsupported/binary.
- Optional short diff summary.

The UI should show:

- A conflict file list.
- Left column: local content, editable.
- Right column: remote content, read-only.
- A visible conflict marker around conflicted content areas.
- Continue and abort actions.

When the user clicks continue:

1. The frontend sends the final left-column content for each conflicted file.
2. The backend writes that content to the working tree.
3. The backend stages those files.
4. The backend creates the merge/sync commit.
5. The backend pushes the target branch.
6. The backend clears the sync state file and removes the temporary branch.

The continue action intentionally treats the left local content as final, even if the user did not fully modify it.

Abort should restore the target branch to the last safe pre-transaction state and clear the paused sync state. The temporary branch should not be deleted during abort; it should keep its `xnote/local-*` name so advanced users can recover content if needed.

## Error Handling

- Missing or invalid config: block setup/sync and show a configuration error.
- Authentication failure: show a clear error and do not modify repository history.
- SSH key missing: block setup/sync when the user explicitly configured that key.
- Remote unreachable: block setup/sync and keep local content unchanged.
- Existing repository with mismatched remote: ask the user before replacing remote configuration.
- Existing paused transaction: require continue or abort before changing sync config or starting another sync.
- Push failure: keep local commits and temporary branch, report retryable state.
- Conflict: pause and enter conflict resolution.
- Unsupported binary conflict: show the file as unsupported and require the user to choose whether local content should win or abort.

No error path should delete user notes or silently discard committed local content.

## Data Flow

### Settings

1. User edits Git settings.
2. User clicks test connection.
3. `SettingsPane.vue` calls `test_git_connection`.
4. Backend returns structured diagnostics.
5. User saves settings.
6. If sync is enabled, `SettingsPane.vue` calls `setup_git_sync`.
7. Backend completes setup or returns a conflict.
8. If conflict, the settings page opens the conflict resolver.
9. On success, config is saved and sync becomes enabled.

### Sync

1. User opens the sync dialog.
2. UI loads current config, local changes, remote changes, and any pending transaction.
3. User clicks sync.
4. `SyncDialog.vue` calls `perform_sync`.
5. Backend completes sync or returns a conflict.
6. If conflict, UI opens the conflict resolver.
7. User edits left-column content and clicks continue.
8. UI calls `continue_git_sync`.
9. Backend writes final content, commits, pushes, and clears sync state.
10. UI refreshes status.

## Testing And Verification

Backend tests should cover Git state transitions using temporary repositories:

- Empty data directory first-time setup.
- Existing local notes with an empty remote.
- Local and remote both have different files.
- Local and remote have same-file conflicts.
- Empty branch field follows remote default branch.
- Explicit branch that does not exist is created and pushed.
- Manual sync with only local changes.
- Manual sync with only remote changes.
- Manual sync with local and remote changes that do not conflict.
- Manual sync with same-file conflict.
- Continue conflict resolution pushes left-column local content.
- Push failure preserves local commits and temporary branch.
- Authentication or remote failure does not lose local content.
- Restart recovery can find a paused sync transaction.
- Abort clears paused state and keeps recoverable local content.

Frontend verification should cover:

- Settings page connection test success and failure states.
- Setup result summary before enabling sync.
- Setup conflict opening the two-column resolver.
- Manual sync success.
- Manual sync conflict and continue.
- Pending transaction shown after reopening the app.
- Disable sync hides sync actions without deleting repository data.

Verification commands:

```bash
npm run build
cd src-tauri && cargo test
```

## Implementation Notes

- Keep sync transaction code in small backend units: auth callbacks, remote discovery, branch management, workspace commits, merge/rebase operations, conflict extraction, and transaction state persistence.
- Prefer structured result types over stringly typed status messages.
- Avoid printing repository credentials or token-containing URLs in logs.
- Keep the first conflict UI intentionally simple; correctness of state recovery and content preservation is more important than editor sophistication.
- Do not commit local note data, generated bundles, or test repositories.
