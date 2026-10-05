//! Terminal output and presentation formatting helpers.

use colored::*;
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
