//! Shell integration workflow (`git claw shell-hook`).
//!
//! Emits shell wrapper functions for Bash, Zsh, and Fish enabling automatic directory
//! navigation (`cd`) upon worktree creation, switching, and finishing.

use crate::workflow::error::WorkflowError;

/// Options for shell hook generation.
///
/// :param shell: Target shell name (e.g., "bash", "zsh", "fish").
///
/// # Example
///
/// ```
/// use git_claw::workflow::ShellHookOptions;
///
/// let options = ShellHookOptions { shell: "fish" };
/// assert_eq!(options.shell, "fish");
/// ```
pub struct ShellHookOptions<'a> {
    pub shell: &'a str,
}

/// Generates POSIX-compatible shell integration functions for Bash and Zsh.
///
/// :return: Shell script containing `claw` and `git` wrapper functions.
///
/// # Example
///
/// ```
/// use git_claw::workflow::shell::generate_posix_hook;
///
/// let script = generate_posix_hook();
/// assert!(script.contains("claw()"));
/// assert!(script.contains("main_dest"));
/// ```
pub fn generate_posix_hook() -> String {
    r#"# git-claw shell integration for bash and zsh
# Usage: eval "$(git claw shell-hook)" or eval "$(git-claw shell-hook)"

claw() {
    if [ "$1" = "cd" ]; then
        local dest
        if [ -n "$2" ]; then
            dest="$(command git-claw cd "$2")" || return $?
        else
            dest="$(command git-claw cd)" || return $?
        fi
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
    elif [ "$1" = "finish" ] || [ "$2" = "finish" ] || { [ "$1" = "spike" ] && [ "$2" = "drop" ]; }; then
        local main_dest
        main_dest="$(command git-claw cd main 2>/dev/null || command git-claw cd 2>/dev/null)"
        command git-claw "$@"
        local ret=$?
        if [ $ret -eq 0 ] && [ ! -d "$PWD" ]; then
            if [ -z "$main_dest" ] && [ -f /tmp/.git_claw_target_cd ]; then
                main_dest="$(cat /tmp/.git_claw_target_cd 2>/dev/null)"
            fi
            if [ -n "$main_dest" ]; then
                cd "$main_dest" || return $?
            fi
        fi
        rm -f /tmp/.git_claw_target_cd 2>/dev/null
        return $ret
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

/// Generates native shell integration functions for Fish shell.
///
/// :return: Fish script containing `claw` and `git` function definitions.
///
/// # Example
///
/// ```
/// use git_claw::workflow::shell::generate_fish_hook;
///
/// let script = generate_fish_hook();
/// assert!(script.contains("function claw"));
/// assert!(script.contains("function git"));
/// assert!(script.contains("main_dest"));
/// ```
pub fn generate_fish_hook() -> String {
    r#"# git-claw shell integration for fish
# Usage: git-claw shell-hook fish | source

function claw
    if test (count $argv) -eq 0
        command git-claw
        return $status
    end

    if test "$argv[1]" = "cd"
        set -l dest
        if test (count $argv) -gt 1
            set dest (command git-claw cd $argv[2])
        else
            set dest (command git-claw cd)
        end
        if test $status -eq 0; and test -n "$dest"
            builtin cd "$dest"
        end
    else if test (count $argv) -ge 2; and test "$argv[2]" = "start"; and contains -- "$argv[1]" feature bugfix hotfix spike
        set -l target_name
        for arg in $argv
            switch $arg
                case feature bugfix hotfix spike start "--*"
                case "*"
                    set target_name $arg
                    break
            end
        end
        command git-claw $argv
        set -l ret $status
        if test $ret -eq 0; and test -n "$target_name"
            set -l dest (command git-claw cd $target_name)
            if test $status -eq 0; and test -n "$dest"
                builtin cd "$dest"
            end
        end
        return $ret
    else if test "$argv[1]" = "finish"; or begin test (count $argv) -ge 2; and test "$argv[2]" = "finish"; end; or begin test (count $argv) -ge 2; and test "$argv[1]" = "spike"; and test "$argv[2]" = "drop"; end
        set -l main_dest (command git-claw cd main 2>/dev/null; or command git-claw cd 2>/dev/null)
        command git-claw $argv
        set -l ret $status
        if test $ret -eq 0; and not test -d "$PWD"
            if test -z "$main_dest"; and test -f /tmp/.git_claw_target_cd
                set main_dest (cat /tmp/.git_claw_target_cd 2>/dev/null)
            end
            if test -n "$main_dest"
                builtin cd "$main_dest"
            end
        end
        command rm -f /tmp/.git_claw_target_cd 2>/dev/null
        return $ret
    else
        command git-claw $argv
    end
end

function git
    if test (count $argv) -gt 0; and test "$argv[1]" = "claw"

        claw $argv[2..-1]
    else
        command git $argv
    end
end
"#
    .to_string()
}

/// Generates the shell hook script for the specified shell.
///
/// :param shell: Target shell name ("fish", "bash", "zsh", etc.).
/// :return: Wrapper script as a String.
///
/// # Example
///
/// ```
/// use git_claw::workflow::shell::generate_shell_hook;
///
/// let script = generate_shell_hook("bash");
/// assert!(script.contains("claw()"));
/// assert!(script.contains("command git-claw"));
///
/// let fish_script = generate_shell_hook("fish");
/// assert!(fish_script.contains("function claw"));
/// ```
pub fn generate_shell_hook(shell: &str) -> String {
    match shell.to_lowercase().as_str() {
        "fish" => generate_fish_hook(),
        _ => generate_posix_hook(),
    }
}

/// Runs the shell hook workflow, outputting the wrapper script to stdout.
///
/// :param options: Options containing the target shell.
/// :return: Result indicating success or [`WorkflowError`].
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
