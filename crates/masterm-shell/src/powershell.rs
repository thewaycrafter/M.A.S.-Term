//! PowerShell adapter

/// PowerShell initialization script
pub const INIT_SCRIPT: &str = r#"
# MASTerm - Master your Terminal
# This script is sourced by PowerShell to integrate MASTerm

$global:__mastermCmdStart = $null

function global:__MastermPreCommand {
    $global:__mastermCmdStart = Get-Date
}

function global:prompt {
    $exitCode = $LASTEXITCODE
    if ($null -eq $exitCode) { $exitCode = 0 }
    
    $duration = 0
    if ($null -ne $global:__mastermCmdStart) {
        $duration = [int]((Get-Date) - $global:__mastermCmdStart).TotalMilliseconds
        $global:__mastermCmdStart = $null
    }
    
    $prompt = & masterm prompt --shell powershell --exit-code $exitCode --duration $duration 2>$null
    if ($prompt) {
        return $prompt
    } else {
        return "PS> "
    }
}

Set-PSReadLineOption -PromptText ""
Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {
    __MastermPreCommand
    [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
}
"#;
