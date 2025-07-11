
use builtin;
use str;

set edit:completion:arg-completer[borgreport] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'borgreport'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'borgreport'= {
            cand --env-dir 'Directory to look for *.env files containing BORG_* variables for a repository.'
            cand --env-inherit 'Inherit BORG_* variables for a single <REPOSITORY> name from the current environment.'
            cand --text-to 'Write the text report to <FILE> instead of stdout.'
            cand --html-to 'Write the HTML report to <FILE>.'
            cand --metrics-to 'Write metrics to <FILE>.'
            cand --mail-to 'Send the report to <ADDR> via `sendmail`'
            cand --mail-from 'Send the report from <ADDR> instead of a default'
            cand --glob-archives 'Enforce a glob archives filter for all repositories.'
            cand --check 'Enforce to run (or not run) `borg check`'
            cand --check-options 'Enforce override of raw `borg check` options for all repositories.'
            cand --compact 'Enforce to run (or not run) `borg compact`'
            cand --compact-options 'Enforce override of raw `borg compact` options for all repositories.'
            cand --borg-binary 'Local path to a specific ''borg'' binary'
            cand --max-age-hours 'Threshold to warn when the last archive is older than <HOURS>'
            cand --no-progress 'Suppress all status updates during processing.'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
