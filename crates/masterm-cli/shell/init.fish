# MASTerm - Fish Integration
# Source this file: masterm init fish | source

function fish_prompt
    set -l exit_code $status
    set -l duration $CMD_DURATION
    if test -z "$duration"
        set duration 0
    end
    masterm prompt --shell fish --exit-code $exit_code --duration $duration 2>/dev/null
end

function fish_right_prompt
end

set -g fish_greeting ""
