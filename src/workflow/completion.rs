//! Shell auto-completion generator workflow (`git claw completion <shell>`).
//!
//! Generates auto-completion scripts for various shells (Bash, Fish, Zsh, PowerShell, Elvish).

use crate::cli::Cli;
use crate::workflow::error::WorkflowError;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io::Write;

/// Generates shell completion script and writes it to the provided writer.
///
/// :param shell: Target shell (e.g., Fish, Bash, Zsh).
/// :param writer: Destination stream implementing [`Write`].
///
/// # Example
///
/// ```
/// use clap_complete::Shell;
/// use git_claw::workflow::completion::generate_completion;
///
/// let mut buffer = Vec::new();
/// generate_completion(Shell::Fish, &mut buffer);
/// let output = String::from_utf8(buffer).expect("valid utf-8");
/// assert!(output.contains("complete -c git-claw"));
/// ```
pub fn generate_completion<W: Write>(shell: Shell, writer: &mut W) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "git-claw", writer);
}

/// Runs the shell completion workflow, writing completion script to stdout.
///
/// :param shell: Target shell for which to generate completions.
/// :return: Result indicating success or error.
///
/// # Example
///
/// ```
/// use clap_complete::Shell;
/// use git_claw::workflow::completion::run_completion_workflow;
///
/// let result = run_completion_workflow(Shell::Bash);
/// assert!(result.is_ok());
/// ```
pub fn run_completion_workflow(shell: Shell) -> Result<(), WorkflowError> {
    let mut stdout = std::io::stdout();
    generate_completion(shell, &mut stdout);
    Ok(())
}
