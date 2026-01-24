# MASTerm - Bash Integration
# Source this file from your .bashrc: eval "$(masterm init bash)"

__masterm_timer_start() {
    __masterm_cmd_start=${__masterm_cmd_start:-$(date +%s%3N)}
}

__masterm_prompt() {
    local exit_code=$?
    local duration=0
    if [[ -n "$__masterm_cmd_start" ]]; then
        local end=$(date +%s%3N)
        duration=$((end - __masterm_cmd_start))
        unset __masterm_cmd_start
    fi
    PS1="$(masterm prompt --shell bash --exit-code $exit_code --duration $duration 2>/dev/null)"
}

trap '__masterm_timer_start' DEBUG
PROMPT_COMMAND="__masterm_prompt"
PS1="$(masterm prompt --shell bash --exit-code 0 --duration 0 2>/dev/null)"
