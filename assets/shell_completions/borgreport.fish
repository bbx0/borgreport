complete -c borgreport -l env-dir -d 'Directory to look for *.env files containing BorgBackup repo settings.' -r -f -a "(__fish_complete_directories)"
complete -c borgreport -l file-to -d 'Write the report to <FILE> instead of stdout.' -r -F
complete -c borgreport -l mail-to -d 'Send the report to <ADDR> via `sendmail`' -r -f
complete -c borgreport -l mail-from -d 'Send the report from <ADDR> instead of a default' -r -f
complete -c borgreport -l glob-archives -d 'Enforce a glob archives filter for all repositories.' -r -f
complete -c borgreport -l check -d 'Enforce to run (or not run) `borg check`' -r -f -a "{true\t'',false\t''}"
complete -c borgreport -l borg-binary -d 'Local path to a specific \'borg\' binary' -r -F
complete -c borgreport -l max-age-hours -d 'Threshold to warn when the last archive is older than <HOURS>' -r -f
complete -c borgreport -l no-progress -d 'Suppress all status updates during processing.'
complete -c borgreport -l help-man -d 'Print an extended help message as input for `help2man`'
complete -c borgreport -s h -l help -d 'Print help (see more with \'--help\')'
complete -c borgreport -s V -l version -d 'Print version'
