//! Terminal output and presentation formatting helpers.

use colored::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Display;

/// Model for rendering active worktree table rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeRow {
    pub slot: u32,
    pub branch: String,
    pub path: String,
    pub ports: String,
    pub commit: String,
}

/// Structured worktree item emitted by `git-claw list --json`.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::WorktreeListItem;
/// use std::collections::BTreeMap;
///
/// let item = WorktreeListItem {
///     slot_id: 1,
///     name: "auth".to_string(),
///     branch: "feature/auth".to_string(),
///     path: "/tmp/wt/auth".to_string(),
///     ports: BTreeMap::from([("PORT_WEB".to_string(), 8001)]),
///     status: "active".to_string(),
/// };
/// assert_eq!(item.slot_id, 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorktreeListItem {
    pub slot_id: u32,
    pub name: String,
    pub branch: String,
    pub path: String,
    pub ports: BTreeMap<String, u16>,
    pub status: String,
}

/// Structured output emitted by `git-claw list --json`.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::ListSummary;
///
/// let summary = ListSummary { worktrees: vec![] };
/// let json = serde_json::to_string(&summary).unwrap();
/// assert_eq!(json, "{\"worktrees\":[]}");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListSummary {
    pub worktrees: Vec<WorktreeListItem>,
}

/// Structured output emitted by `git-claw <type> start --json`.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::StartSummary;
/// use std::collections::BTreeMap;
///
/// let summary = StartSummary {
///     slot_id: 1,
///     name: "auth".to_string(),
///     branch: "feature/auth".to_string(),
///     path: "/tmp/wt/auth".to_string(),
///     ports: BTreeMap::new(),
///     test_database: Some("test_repo_slot_1".to_string()),
/// };
/// assert_eq!(summary.slot_id, 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StartSummary {
    pub slot_id: u32,
    pub name: String,
    pub branch: String,
    pub path: String,
    pub ports: BTreeMap<String, u16>,
    pub test_database: Option<String>,
}

/// Structured output emitted by `git-claw finish --json`.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::FinishSummary;
///
/// let summary = FinishSummary {
///     name: "auth".to_string(),
///     branch: "feature/auth".to_string(),
///     target_branch: "main".to_string(),
///     slot_id: Some(1),
///     primary_root: "/tmp/repo".to_string(),
///     remote_deleted: false,
/// };
/// assert_eq!(summary.target_branch, "main");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FinishSummary {
    pub name: String,
    pub branch: String,
    pub target_branch: String,
    pub slot_id: Option<u32>,
    pub primary_root: String,
    pub remote_deleted: bool,
}

/// Structured output emitted by `git-claw publish --json` and `git-claw <type> publish --json`.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::PublishSummary;
///
/// let summary = PublishSummary {
///     name: "auth".to_string(),
///     branch: "feature/auth".to_string(),
///     remote: "origin".to_string(),
/// };
/// assert_eq!(summary.remote, "origin");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishSummary {
    pub name: String,
    pub branch: String,
    pub remote: String,
}

/// Structured output emitted by `git-claw spike drop --json`.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::DropSpikeSummary;
///
/// let summary = DropSpikeSummary {
///     name: "poc".to_string(),
///     branch: "spike/poc".to_string(),
///     slot_id: Some(1),
///     primary_root: "/tmp/repo".to_string(),
/// };
/// assert_eq!(summary.branch, "spike/poc");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DropSpikeSummary {
    pub name: String,
    pub branch: String,
    pub slot_id: Option<u32>,
    pub primary_root: String,
}

/// Structured output emitted by `git-claw init --yes --json`.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::InitSummary;
/// use std::collections::BTreeMap;
///
/// let summary = InitSummary {
///     config_path: "/tmp/repo/.git-claw.toml".to_string(),
///     main_branch: "main".to_string(),
///     ports: BTreeMap::new(),
///     copy_files: vec![".env".to_string()],
///     environment_type: Some("pyenv".to_string()),
///     compose_file: Some("docker-compose.yml".to_string()),
/// };
/// assert_eq!(summary.main_branch, "main");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InitSummary {
    pub config_path: String,
    pub main_branch: String,
    pub ports: BTreeMap<String, u16>,
    pub copy_files: Vec<String>,
    pub environment_type: Option<String>,
    pub compose_file: Option<String>,
}

/// Serializes `value` as pretty-printed JSON and writes a single JSON document to stdout.
///
/// # Examples
///
/// ```
/// use git_claw::cli::output::{print_json, ListSummary};
///
/// let summary = ListSummary { worktrees: vec![] };
/// print_json(&summary).unwrap();
/// ```
pub fn print_json<T: Serialize>(value: &T) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{}", json);
    Ok(())
}

/// Print an error message to stderr with red prefix.
pub fn print_error<T: Display>(msg: T) {
    eprintln!("{} {}", "error:".red().bold(), msg);
}

/// Print a warning message to stderr with yellow prefix.
#[allow(dead_code)]
pub fn print_warning<T: Display>(msg: T) {
    eprintln!("{} {}", "warning:".yellow().bold(), msg);
}

/// Print a success message to stdout with green prefix.
#[allow(dead_code)]
pub fn print_success<T: Display>(msg: T) {
    println!("{} {}", "success:".green().bold(), msg);
}

/// Prints the active worktree slots in an aligned ASCII table.
pub fn print_worktree_table(rows: &[WorktreeRow]) {
    if rows.is_empty() {
        println!("No active worktrees found.");
        return;
    }

    let slot_header = "SLOT";
    let branch_header = "BRANCH";
    let path_header = "DIRECTORY";
    let ports_header = "PORTS";
    let commit_header = "COMMIT";

    let slot_w = rows
        .iter()
        .map(|r| r.slot.to_string().len())
        .max()
        .unwrap_or(0)
        .max(slot_header.len());

    let branch_w = rows
        .iter()
        .map(|r| r.branch.len())
        .max()
        .unwrap_or(0)
        .max(branch_header.len());

    let path_w = rows
        .iter()
        .map(|r| r.path.len())
        .max()
        .unwrap_or(0)
        .max(path_header.len());

    let ports_w = rows
        .iter()
        .map(|r| r.ports.len())
        .max()
        .unwrap_or(0)
        .max(ports_header.len());

    let commit_w = rows
        .iter()
        .map(|r| r.commit.len())
        .max()
        .unwrap_or(0)
        .max(commit_header.len());

    println!(
        "{:<slot_w$}  {:<branch_w$}  {:<path_w$}  {:<ports_w$}  {:<commit_w$}",
        slot_header.bold(),
        branch_header.bold(),
        path_header.bold(),
        ports_header.bold(),
        commit_header.bold(),
    );

    for r in rows {
        println!(
            "{:<slot_w$}  {:<branch_w$}  {:<path_w$}  {:<ports_w$}  {:<commit_w$}",
            r.slot.to_string().cyan(),
            r.branch.green(),
            r.path,
            r.ports.yellow(),
            r.commit.bright_black(),
        );
    }
}
