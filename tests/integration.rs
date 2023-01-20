use std::process::Command;

#[test]
fn check_fzf_command_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_jq-repl"))
        .arg("--history-file")
        .arg("/tmp/jq_repl_history")
        .arg("--show-fzf-command")
        .arg("/tmp/foo.json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    let expected = r#"fzf \
--disabled \
--print-query \
$'--preview-window=up,99%,border-bottom' \
$'--info=hidden' \
$'--history=/tmp/jq_repl_history' \
$'--preview=gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json' \
$'--bind=ctrl-k:kill-line,pgup:preview-page-up,pgdn:preview-page-down,alt-w:toggle-preview-wrap,home:preview-top,end:preview-bottom' \
$'--bind=alt-s:change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C --slurp {q} /tmp/foo.json' \
$'--bind=alt-S:change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json' \
$'--bind=alt-c:change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -c {q} /tmp/foo.json' \
$'--bind=alt-C:change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json' \
$'--bind=ctrl-space:change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C -M {q} /tmp/foo.json | gron --colorize' \
$'--bind=alt-space:change-preview:gojq -L ~/.jq -L ~/.jq/.jq --raw-output -C {q} /tmp/foo.json'
"#;

    assert_eq!(expected, stdout);
}
