//! Fish shell adapter

/// Fish initialization script
pub const INIT_SCRIPT: &str = r#"
# MASTerm - Master your Terminal
# This script is sourced by fish to integrate MASTerm

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
"#;
