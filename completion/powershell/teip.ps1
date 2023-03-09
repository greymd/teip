
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'teip' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'teip'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-')) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'teip' {
            [CompletionResult]::new('-g', 'g', [CompletionResultType]::ParameterName, 'Bypassing lines that match the regular expression <pattern>')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Bypassing these white-space separated fields')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Use <delimiter> for field delimiter of -f')
            [CompletionResult]::new('-D', 'D', [CompletionResultType]::ParameterName, 'Use regular expression <pattern> for field delimiter of -f')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Bypassing these characters')
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'Bypassing those lines')
            [CompletionResult]::new('-I', 'I', [CompletionResultType]::ParameterName, 'Replace the <replace-str> with bypassed chunk in the <command> and -s is forcefully enabled.')
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'Execute <string> on another process that will receive identical standard input as the teip, and numbers given by the result are used as line numbers for bypassing')
            [CompletionResult]::new('-A', 'A', [CompletionResultType]::ParameterName, 'Alias of -e ''grep -n -A <number> <pattern>''')
            [CompletionResult]::new('-B', 'B', [CompletionResultType]::ParameterName, 'Alias of -e ''grep -n -B <number> <pattern>''')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'Alias of -e ''grep -n -C <number> <pattern>''')
            [CompletionResult]::new('--sed', 'sed', [CompletionResultType]::ParameterName, 'Alias of -e ''sed -n "<pattern>="''')
            [CompletionResult]::new('--awk', 'awk', [CompletionResultType]::ParameterName, 'Alias of -e ''awk "<pattern>{print NR}"''')
            [CompletionResult]::new('--completion', 'completion', [CompletionResultType]::ParameterName, 'completion')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, '-g bypasses only matched parts')
            [CompletionResult]::new('-G', 'G', [CompletionResultType]::ParameterName, '-g interprets Oniguruma regular expressions.')
            [CompletionResult]::new('--csv', 'csv', [CompletionResultType]::ParameterName, '-f interprets <list> as field number of a CSV according to RFC 4180, instead of white-space separated fields')
            [CompletionResult]::new('--unko', 'unko', [CompletionResultType]::ParameterName, 'unko')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Execute new command for each bypassed chunk')
            [CompletionResult]::new('--chomp', 'chomp', [CompletionResultType]::ParameterName, 'Command spawned by -s receives standard input without trailing newlines')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Invert the range of bypassing')
            [CompletionResult]::new('-z', 'z', [CompletionResultType]::ParameterName, 'Line delimiter is NUL instead of a newline')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
