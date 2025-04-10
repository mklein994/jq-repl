use std::io::Write;
use std::process::{Command, Stdio};

const INPUT_PARTIAL: &str = "IpsumvoluptatesasperioresquisquamaliasullamxtemporibusNisicommodiquaeratasperioresnemosuntoImpeditco";

fn build_charcounter(length: usize) -> Result<String, Box<dyn std::error::Error>> {
    let test_input = INPUT_PARTIAL.repeat(10);
    assert!(length <= test_input.len());
    let input: String = test_input.chars().take(length).collect();

    let mut charcounter = Command::new(env!("CARGO_BIN_EXE_charcounter"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let mut stdin = charcounter.stdin.take().unwrap();
        stdin.write_all(input.as_bytes())?;
        stdin.flush()?;
    }

    let output = charcounter.wait_with_output().unwrap();
    let count = String::from_utf8(output.stdout)?;

    Ok(count)
}

#[test]
fn blank_when_given_few_characters() {
    let output = build_charcounter(2).unwrap();
    assert!(output.is_empty());
}

#[test]
fn blank_when_given_almost_max_characters() {
    let output = build_charcounter(999).unwrap();
    assert_eq!(output, "\x1b[0;31m999/1000\x1b[0m");
}

#[test]
fn blank_when_given_max_characters() {
    let output = build_charcounter(1_000).unwrap();
    assert_eq!(output, "\x1b[1;31m1000/1000\x1b[0m");
}
