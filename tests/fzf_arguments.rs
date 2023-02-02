use std::process::{Command, Stdio};

#[test]
fn check_fzf_command_output() {
    let output = Command::new(env!(concat!("CARGO_BIN_EXE_", clap::crate_name!())))
        .env_remove("EDITOR")
        .env_remove("PAGER")
        .arg("--history-file")
        .arg("/tmp/jq_repl_history")
        .arg("--show-fzf-command")
        .arg("-n")
        .stdin(Stdio::inherit())
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
$'--prompt=-n> ' \
$'--header=M-e: editor \xE2\x81\x84 M-v: vd \xE2\x81\x84 M-l: pager \xE2\x81\x84 ^<space>: gron' \
$'--history=/tmp/jq_repl_history' \
$'--preview=gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n {q} ' \
$'--bind=ctrl-k:kill-line,pgup:preview-page-up,pgdn:preview-page-down,alt-w:toggle-preview-wrap,home:preview-top,end:preview-bottom' \
$'--bind=alt-s:change-prompt(-ns> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n --slurp {q} ' \
$'--bind=alt-S:change-prompt(-n> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n {q} ' \
$'--bind=alt-c:change-prompt(-nc> )+preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n -c {q} ' \
$'--bind=alt-C:change-prompt(-n> )+preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n {q} ' \
$'--bind=ctrl-space:change-prompt(-n gron> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n -M {q}  | gron --colorize' \
$'--bind=alt-space:change-prompt(-n> )+change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n {q} ' \
$'--bind=alt-e:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n -M {q}  | nvim -c \'set ft=json\' -' \
$'--bind=alt-v:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n -M {q}  | vd --filetype json' \
$'--bind=alt-V:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n -M {q}  | vd --filetype csv' \
$'--bind=alt-l:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n {q}  | less -R' \
$'--bind=alt-L:execute:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -n -M {q}  | bat --language json' < /dev/null
"##;

    assert_eq!(
        expected.lines().count(),
        stdout.lines().count(),
        "expected != actual line count.\nstdout:\n{stdout}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    if expected != stdout {
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
}
