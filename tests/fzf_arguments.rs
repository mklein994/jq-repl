use std::process::{Command, Stdio};

#[test]
fn check_fzf_command_output() {
    let output = Command::new(env!(concat!("CARGO_BIN_EXE_", clap::crate_name!())))
        .env("JQ_REPL_TEST", "true")
        // Internal programs
        .env_remove("JQ_REPL_TRANSFORM_BIN")
        .env_remove("JQ_REPL_CHARCOUNTER_BIN")
        .env_remove("JQ_REPL_COMPLETION_BIN")
        // Settings
        .env_remove("JQ_REPL_HISTORY")
        .env_remove("JQ_REPL_LIB")
        // Default programs
        .env_remove("JQ_BIN")
        .env_remove("JQ_REPL_JQ_BIN")
        .arg("--history-file")
        .arg("/tmp/jq_repl_history")
        .arg("--config")
        .arg("./tests/config.toml")
        .arg("--show-fzf-command")
        .arg("-n")
        .arg("./tests/foo bar.json")
        .stdin(Stdio::inherit())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    insta::assert_snapshot!(stdout);
}
