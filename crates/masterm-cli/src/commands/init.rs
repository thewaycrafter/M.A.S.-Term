//! Init command - generates shell initialization script

use anyhow::Result;
use clap::Args;
use masterm_core::config::ShellType;

/// Init command arguments
#[derive(Args)]
pub struct InitArgs {
    /// Shell to initialize
    shell: String,
}

/// Run the init command
pub async fn run(args: InitArgs) -> Result<()> {
    // Initialize cache
    if let Err(e) = masterm_core::cache::CacheManager::init() {
        // Log error but continue - don't crash shell init
        eprintln!("Failed to initialize cache: {}", e);
    }

    let shell = ShellType::from_path(&args.shell);

    let script = match shell {
        ShellType::Zsh => generate_zsh_init(),
        ShellType::Bash => generate_bash_init(),
        ShellType::Fish => generate_fish_init(),
        _ => {
            eprintln!("Unsupported shell: {}", args.shell);
            std::process::exit(1);
        }
    };

    print!("{}", script);
    Ok(())
}

/// Generate Zsh initialization script
fn generate_zsh_init() -> String {
    r#"
# MASTerm - Master your Terminal
# https://masterm.dev

# Store command start time
__masterm_preexec() {
    __masterm_cmd_start=$EPOCHREALTIME
}

# Generate prompt after command ensures prompt is fresh
__masterm_precmd() {
    local exit_code=$?
    local duration=0

    if [[ -n "$__masterm_cmd_start" ]]; then
        local end=$EPOCHREALTIME
        duration=$(printf "%.0f" $(( (end - __masterm_cmd_start) * 1000 )))
        unset __masterm_cmd_start
    fi

    # Generate full prompt
    PROMPT="$(masterm prompt --shell zsh --exit-code $exit_code --duration $duration)"
    
    # Right prompt logic (placeholder for future expansion)
    RPROMPT=""
}

# Transient prompt: Redraw prompt as a compact version before executing next command
__masterm_zle-line-init() {
    zle reset-prompt
}

# Hook into zsh
autoload -Uz add-zsh-hook
add-zsh-hook preexec __masterm_preexec
add-zsh-hook precmd __masterm_precmd
zle -N zle-line-init __masterm_zle-line-init

# Initial prompt
PROMPT="$(masterm prompt --shell zsh --exit-code 0 --duration 0)"

# Show welcome screen
masterm welcome
"#
    .to_string()
}

/// Generate Bash initialization script
fn generate_bash_init() -> String {
    r#"
# MASTerm - Master your Terminal
# https://masterm.dev

# Store command start time
__masterm_timer_start() {
    __masterm_cmd_start=${__masterm_cmd_start:-$EPOCHREALTIME}
}

# Generate prompt
__masterm_prompt() {
    local exit_code=$?
    local duration=0

    if [[ -n "$__masterm_cmd_start" ]]; then
        local end=$EPOCHREALTIME
        duration=$(printf "%.0f" $(echo "($end - $__masterm_cmd_start) * 1000" | bc))
        unset __masterm_cmd_start
    fi

    PS1="$(masterm prompt --shell bash --exit-code $exit_code --duration $duration)"
}

trap '__masterm_timer_start' DEBUG
PROMPT_COMMAND="__masterm_prompt"

# Initial prompt
PS1="$(masterm prompt --shell bash --exit-code 0 --duration 0)"

# Show welcome screen
masterm welcome
"#
    .to_string()
}

/// Generate Fish initialization script
fn generate_fish_init() -> String {
    r#"
# MASTerm - Master your Terminal
# https://masterm.dev

function fish_prompt
    set -l exit_code $status
    set -l duration $CMD_DURATION

    masterm prompt --shell fish --exit-code $exit_code --duration $duration
end

function fish_right_prompt
    # Empty right prompt - MASTerm handles this via masterm prompt if needed
end

# Transient prompt support (requires 'transient' plugin or similar, setting up foundation)
function fish_mode_prompt; end

# Show welcome screen
masterm welcome
"#
    .to_string()
}
