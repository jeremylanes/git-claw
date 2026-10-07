//! Shell integration workflow (`git claw shell-hook`).
//!
//! Emits shell wrapper functions for bash and zsh enabling automatic directory
//! navigation (`cd`) upon worktree creation and switching.

use crate::workflow::error::WorkflowError;

/// Options for shell hook generation.
pub struct ShellHookOptions<'a> {
    pub shell: &'a str,
}

/// Generates the shell hook script for the specified shell.
///
/// # Example
///
/// ```
/// use git_claw::workflow::shell::generate_shell_hook;
///
/// let script = generate_shell_hook("bash");
/// assert!(script.contains("claw()"));
/// assert!(script.contains("command git-claw"));
/// ```
pub fn generate_shell_hook(_shell: &str) -> String {
    r#"# git-claw shell integration for bash and zsh
# Usage: eval "$(git claw shell-hook)" or eval "$(git-claw shell-hook)"

claw() {
    if [ "$1" = "cd" ]; then
        local dest
        dest="$(command git-claw cd "$2")" || return $?
        if [ -n "$dest" ]; then
            cd "$dest" || return $?
        fi
    elif [ "$2" = "start" ] && { [ "$1" = "feature" ] || [ "$1" = "bugfix" ] || [ "$1" = "hotfix" ] || [ "$1" = "spike" ]; }; then
        local target_name="$3"
        for arg in "$@"; do
            case "$arg" in
                feature|bugfix|hotfix|spike|start|--*) ;;
                *) target_name="$arg"; break ;;
            esac
        done
        command git-claw "$@" || return $?
        if [ -n "$target_name" ]; then
            local dest
            dest="$(command git-claw cd "$target_name")" || return $?
            if [ -n "$dest" ]; then
                cd "$dest" || return $?
            fi
        fi
    else
        command git-claw "$@"
    fi
}

git() {
    if [ "$1" = "claw" ]; then
        shift
        claw "$@"
    else
        command git "$@"
    fi
}
"#
    .to_string()
}

/// Runs the shell hook workflow, outputting the wrapper script to stdout.
///
/// # Example
///
/// ```
/// use git_claw::workflow::shell::{run_shell_hook, ShellHookOptions};
///
/// let res = run_shell_hook(ShellHookOptions { shell: "bash" });
/// assert!(res.is_ok());
/// ```
pub fn run_shell_hook(options: ShellHookOptions) -> Result<(), WorkflowError> {
    let script = generate_shell_hook(options.shell);
    print!("{}", script);
    Ok(())
}
