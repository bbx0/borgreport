complete -c borgreport -l env-dir -d 'Directory to look for *.env files containing BORG_* variables for a repository.' -r -f -a "(__fish_complete_directories)"
complete -c borgreport -l env-inherit -d 'Inherit BORG_* variables for a single <REPOSITORY> name from the current environment.' -r -f
complete -c borgreport -l text-to -d 'Write the text report to <FILE> instead of stdout.' -r -F
complete -c borgreport -l html-to -d 'Write the HTML report to <FILE>.' -r -F
complete -c borgreport -l metrics-to -d 'Write metrics to <FILE>.' -r -F
complete -c borgreport -l mail-to -d 'Send the report to <ADDR> via `sendmail`' -r -f
complete -c borgreport -l mail-from -d 'Send the report from <ADDR> instead of a default' -r -f
complete -c borgreport -l glob-archives -d 'Enforce a glob archives filter for all repositories.' -r -f
complete -c borgreport -l check -d 'Enforce to run (or not run) `borg check`' -r -f -a "true\t''
false\t''"
complete -c borgreport -l check-options -d 'Enforce override of raw `borg check` options for all repositories.' -r -f
complete -c borgreport -l compact -d 'Enforce to run (or not run) `borg compact`' -r -f -a "true\t''
false\t''"
complete -c borgreport -l compact-options -d 'Enforce override of raw `borg compact` options for all repositories.' -r -f
complete -c borgreport -l borg-binary -d 'Local path to a specific \'borg\' binary' -r -F
complete -c borgreport -l max-age-hours -d 'Threshold to warn when the last archive is older than <HOURS>' -r -f
complete -c borgreport -l no-progress -d 'Suppress all status updates during processing.'
complete -c borgreport -s h -l help -d 'Print help (see more with \'--help\')'
complete -c borgreport -s V -l version -d 'Print version'
