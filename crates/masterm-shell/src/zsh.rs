//! Zsh shell adapter

/// Zsh initialization script
pub const INIT_SCRIPT: &str = r#"
# MASTerm - Master your Terminal
# This script is sourced by zsh to integrate MASTerm

setopt PROMPT_SUBST

__masterm_preexec() {
    __masterm_cmd_start=$EPOCHREALTIME
    
    # Run safety check
    # We use the raw command line from $1
    # If the check fails (exit code 1), we can't easily stop execution in vanilla zsh preexec
    # without re-binding enter-key or similar complex hooks.
    # HOWEVER, since 'masterm check' is interactive, if it returns 1 (user said no),
    # we want to stop.
    # The only reliable way in standard Zsh preexec to "stop" is to send a signal or throw an error
    # but that kills the shell or looks ugly.
    # For now, we rely on the fact that 'masterm check' effectively pauses execution
    # and warns the user. If they say 'no' in the rust binary, it exits 1.
    # We can print a big CANCELLED message.
    
    masterm check "$1"
}

__masterm_precmd() {
    local exit_code=$?
    local duration=0

    if [[ -n "$__masterm_cmd_start" ]]; then
        local end=$EPOCHREALTIME
        duration=$(printf "%.0f" $((($end - $__masterm_cmd_start) * 1000)))
        unset __masterm_cmd_start
    fi

    PROMPT="$(masterm prompt --shell zsh --exit-code $exit_code --duration $duration 2>/dev/null)"
    RPROMPT=""
}

if [[ -z "$__masterm_hooked" ]]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook preexec __masterm_preexec
    add-zsh-hook precmd __masterm_precmd
    __masterm_hooked=1
fi

PROMPT="$(masterm prompt --shell zsh --exit-code 0 --duration 0 2>/dev/null)"
"#;
