//! Zsh shell adapter

/// Zsh initialization script
pub const INIT_SCRIPT: &str = r#"
# MASTerm - Master your Terminal
# This script is sourced by zsh to integrate MASTerm

setopt PROMPT_SUBST

__masterm_preexec() {
    __masterm_cmd_start=$EPOCHREALTIME
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
