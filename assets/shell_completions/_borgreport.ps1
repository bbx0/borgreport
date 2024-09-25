
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'borgreport' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'borgreport'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'borgreport' {
            [CompletionResult]::new('--env-dir', '--env-dir', [CompletionResultType]::ParameterName, 'Directory to look for *.env files containing BorgBackup repo settings.')
            [CompletionResult]::new('--file', '--file', [CompletionResultType]::ParameterName, 'Write the report to <FILE> instead of stdout')
            [CompletionResult]::new('--mail-to', '--mail-to', [CompletionResultType]::ParameterName, 'Send the report to <ADDR> via `sendmail`')
            [CompletionResult]::new('--mail-from', '--mail-from', [CompletionResultType]::ParameterName, 'Send the report from <ADDR> instead of a default')
            [CompletionResult]::new('--glob-archives', '--glob-archives', [CompletionResultType]::ParameterName, 'Enforce a glob archives filter for all repositories.')
            [CompletionResult]::new('--no-progress', '--no-progress', [CompletionResultType]::ParameterName, 'Suppress all status updates during processing.')
            [CompletionResult]::new('--no-check', '--no-check', [CompletionResultType]::ParameterName, 'Force disable all `borg check` invocations.')
            [CompletionResult]::new('--help-man', '--help-man', [CompletionResultType]::ParameterName, 'Print an extended help message as input for `help2man`')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
