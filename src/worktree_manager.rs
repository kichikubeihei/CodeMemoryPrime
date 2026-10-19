use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub agent_id: String,
    pub branch: String,
    pub path: String,
    pub commit: String,
    pub is_clean: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeMergeResult {
    pub agent_id: String,
    pub tests_passed: bool,
    pub test_output: String,
    pub merged: bool,
    pub merge_commit: Option<String>,
    pub worktree_cleaned: bool,
    pub message: String,
}

fn get_worktree_dir(repo_path: &Path, agent_id: &str) -> PathBuf {
    let repo_name = repo_path.file_name().and_then(|s| s.to_str()).unwrap_or("repo");
    let parent = repo_path.parent().unwrap_or(repo_path);
    parent.join(".cmp_worktrees").join(repo_name).join(agent_id)
}

fn setup_sibling_symlinks(repo_path: &Path) {
    if let Some(parent) = repo_path.parent() {
        let worktree_parent = parent.join(".cmp_worktrees");
        let _ = std::fs::create_dir_all(&worktree_parent);
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let name = entry.file_name();
                        let target = worktree_parent.join(&name);
                        if !target.exists() && name != ".cmp_worktrees" {
                            #[cfg(unix)]
                            let _ = std::os::unix::fs::symlink(entry.path(), target);
                        }
                    }
                }
            }
        }
    }
}

pub fn create_agent_worktree(
    repo_path_str: &str,
    agent_id: &str,
    base_branch: Option<&str>,
) -> Result<WorktreeInfo, String> {
    let repo_path = PathBuf::from(repo_path_str);
    if !repo_path.exists() {
        return Err(format!("Repository path does not exist: {}", repo_path_str));
    }

    setup_sibling_symlinks(&repo_path);

    let worktree_dir = get_worktree_dir(&repo_path, agent_id);
    if let Some(p) = worktree_dir.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let branch_name = format!("agent/{}", agent_id);

    // If worktree already exists, remove it first
    if worktree_dir.exists() {
        let _ = Command::new("git")
            .args(["-C", repo_path_str, "worktree", "remove", "--force", worktree_dir.to_str().unwrap()])
            .output();
        let _ = Command::new("git")
            .args(["-C", repo_path_str, "branch", "-D", &branch_name])
            .output();
    }

    let mut cmd = Command::new("git");
    cmd.args(["-C", repo_path_str, "worktree", "add", "-b", &branch_name, worktree_dir.to_str().unwrap()]);
    if let Some(base) = base_branch {
        cmd.arg(base);
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute git worktree add: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "git worktree add failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Get HEAD commit of new worktree
    let rev_out = Command::new("git")
        .args(["-C", worktree_dir.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;
    let commit = String::from_utf8_lossy(&rev_out.stdout).trim().to_string();

    Ok(WorktreeInfo {
        agent_id: agent_id.to_string(),
        branch: branch_name,
        path: worktree_dir.to_string_lossy().to_string(),
        commit,
        is_clean: true,
    })
}

pub fn run_worktree_gauntlet_and_merge(
    repo_path_str: &str,
    agent_id: &str,
    test_command: Option<&str>,
) -> Result<WorktreeMergeResult, String> {
    let repo_path = PathBuf::from(repo_path_str);
    let worktree_dir = get_worktree_dir(&repo_path, agent_id);
    let branch_name = format!("agent/{}", agent_id);

    if !worktree_dir.exists() {
        return Err(format!("Worktree for agent '{}' does not exist at {:?}", agent_id, worktree_dir));
    }

    // 1. Commit any uncommitted changes inside the worktree
    let status_out = Command::new("git")
        .args(["-C", worktree_dir.to_str().unwrap(), "status", "--porcelain"])
        .output()
        .map_err(|e| e.to_string())?;

    if !status_out.stdout.is_empty() {
        let _ = Command::new("git")
            .args(["-C", worktree_dir.to_str().unwrap(), "add", "-A"])
            .output();
        let _ = Command::new("git")
            .args(["-C", worktree_dir.to_str().unwrap(), "commit", "-m", &format!("work: automated progress from agent {}", agent_id)])
            .output();
    }

    // 2. Run Test Command / Gauntlet inside the isolated worktree
    let mut tests_passed = true;
    let mut test_output = String::from("No test command specified; proceeding with merge.");

    if let Some(cmd_str) = test_command {
        if !cmd_str.trim().is_empty() {
            let out = Command::new("zsh")
                .arg("-c")
                .arg(cmd_str)
                .current_dir(&worktree_dir)
                .output()
                .map_err(|e| format!("Failed to execute test command inside worktree: {}", e))?;

            tests_passed = out.status.success();
            let raw_test = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            test_output = crate::terminal_bounder::bound_terminal_output(&raw_test, 3000);
        }
    }

    // 3. If tests failed, enforce the "No-Mistakes" Gate and ABORT merge
    if !tests_passed {
        return Ok(WorktreeMergeResult {
            agent_id: agent_id.to_string(),
            tests_passed: false,
            test_output,
            merged: false,
            merge_commit: None,
            worktree_cleaned: false,
            message: format!("GAUNTLET FAILED: Agent '{}' changes failed tests. Merge blocked to protect main branch. Worktree preserved for debugging.", agent_id),
        });
    }

    // 4. Merge branch into current main branch
    let merge_msg = format!("feat(agent): merge verified worktree task from agent {} (Gauntlet Passed)", agent_id);
    let merge_out = Command::new("git")
        .args(["-C", repo_path_str, "merge", "--no-ff", &branch_name, "-m", &merge_msg])
        .output()
        .map_err(|e| format!("Git merge failed: {}", e))?;

    let merged = merge_out.status.success();
    if !merged {
        let err_msg = format!(
            "Merge conflict or error:\n{}\n{}",
            String::from_utf8_lossy(&merge_out.stdout),
            String::from_utf8_lossy(&merge_out.stderr)
        );
        return Ok(WorktreeMergeResult {
            agent_id: agent_id.to_string(),
            tests_passed: true,
            test_output,
            merged: false,
            merge_commit: None,
            worktree_cleaned: false,
            message: err_msg,
        });
    }

    // Get merge commit SHA
    let head_out = Command::new("git")
        .args(["-C", repo_path_str, "rev-parse", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;
    let merge_commit = Some(String::from_utf8_lossy(&head_out.stdout).trim().to_string());

    // 5. Cleanup Worktree and Branch
    let _ = Command::new("git")
        .args(["-C", repo_path_str, "worktree", "remove", "--force", worktree_dir.to_str().unwrap()])
        .output();
    let _ = Command::new("git")
        .args(["-C", repo_path_str, "branch", "-d", &branch_name])
        .output();

    Ok(WorktreeMergeResult {
        agent_id: agent_id.to_string(),
        tests_passed: true,
        test_output,
        merged: true,
        merge_commit,
        worktree_cleaned: true,
        message: format!("SUCCESS: Worktree for agent '{}' passed gauntlet and was cleanly merged into main.", agent_id),
    })
}

pub fn cleanup_worktree(repo_path_str: &str, agent_id: &str) -> Result<String, String> {
    let repo_path = PathBuf::from(repo_path_str);
    let worktree_dir = get_worktree_dir(&repo_path, agent_id);
    let branch_name = format!("agent/{}", agent_id);

    let _ = Command::new("git")
        .args(["-C", repo_path_str, "worktree", "remove", "--force", worktree_dir.to_str().unwrap()])
        .output();
    let _ = Command::new("git")
        .args(["-C", repo_path_str, "branch", "-D", &branch_name])
        .output();

    Ok(format!("Worktree and branch for agent '{}' cleaned successfully.", agent_id))
}

pub fn list_active_worktrees(repo_path_str: &str) -> Result<Vec<WorktreeInfo>, String> {
    let output = Command::new("git")
        .args(["-C", repo_path_str, "worktree", "list", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to run git worktree list: {}", e))?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();

    let mut current_path = String::new();
    let mut current_commit = String::new();
    let mut current_branch = String::new();

    for line in text.lines() {
        if line.starts_with("worktree ") {
            current_path = line.trim_start_matches("worktree ").trim().to_string();
        } else if line.starts_with("HEAD ") {
            current_commit = line.trim_start_matches("HEAD ").trim().to_string();
        } else if line.starts_with("branch ") {
            current_branch = line.trim_start_matches("branch ").trim().to_string();
            if current_branch.starts_with("refs/heads/agent/") || current_path.contains(".cmp_worktrees") {
                let agent_id = current_branch.replace("refs/heads/agent/", "");
                worktrees.push(WorktreeInfo {
                    agent_id,
                    branch: current_branch.clone(),
                    path: current_path.clone(),
                    commit: current_commit.clone(),
                    is_clean: true,
                });
            }
        } else if line.is_empty() {
            current_path.clear();
            current_commit.clear();
            current_branch.clear();
        }
    }

    Ok(worktrees)
}
