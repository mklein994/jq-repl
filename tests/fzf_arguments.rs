use std::process::{Command, Stdio};

#[test]
fn check_fzf_command_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_jq-repl"))
        .arg("--history-file")
        .arg("/tmp/jq_repl_history")
        .arg("--show-fzf-command")
        .arg("/tmp/foo.json")
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    let line = line!();
    let expected = r##"#!/bin/bash

fzf \
--disabled \
$'--preview-window=up,99%,border-bottom' \
$'--info=hidden' \
--header-first \
$'--header=M-e: editor \xE2\x81\x84 M-v: vd \xE2\x81\x84 M-l: pager \xE2\x81\x84 ^<space>: gron' \
$'--history=/tmp/jq_repl_history' \
$'--preview=gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json' \
$'--bind=ctrl-k:kill-line,pgup:preview-page-up,pgdn:preview-page-down,alt-w:toggle-preview-wrap,home:preview-top,end:preview-bottom' \
$'--bind=alt-s:change-prompt(-s> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C --slurp {q} /tmp/foo.json' \
$'--bind=alt-S:change-prompt(> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json' \
$'--bind=alt-c:change-prompt(-c> )+preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -c {q} /tmp/foo.json' \
$'--bind=alt-C:change-prompt(> )+preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json' \
$'--bind=ctrl-space:change-prompt(gron> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -M {q} /tmp/foo.json | gron --colorize' \
$'--bind=alt-space:change-prompt(> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json' \
$'--bind=alt-e:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -M {q} /tmp/foo.json | nvim -c \'set ft=json\' -' \
$'--bind=alt-v:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -M {q} /tmp/foo.json | vd --filetype json' \
$'--bind=alt-V:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -M {q} /tmp/foo.json | vd --filetype csv' \
$'--bind=alt-l:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json | less -R' \
$'--bind=alt-L:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -M {q} /tmp/foo.json | bat --language json' < /dev/null
"##;

    assert_eq!(
        expected.lines().count(),
        stdout.lines().count(),
        "expected != actual line count"
    );

    for (line_number, (expected_line, actual_line)) in
        expected.lines().zip(stdout.lines()).enumerate()
    {
        assert_eq!(
            expected_line,
            actual_line,
            "failed on line {}",
            line as usize + line_number + 1
        );
    }
}
