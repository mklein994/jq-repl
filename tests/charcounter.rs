use std::io::Write;
use std::process::{Command, Stdio};

const INPUT: &str = "IpsumvoluptatesasperioresquisquamaliasullamxtemporibusNisicommodiquaeratasperioresnemosuntoImpeditconsecteturassumendaofficiaitaquererumporroDolorestemporaadipisciplaceatcommodiquasexcepturifugitEnimnihilmolestiaslkjhaaaalkjhasfdlkjhasfdaoiuqewrqljaoiuvzvczvcfdsaopqljqjafdsjhasfljlasdlkjsafdoiuqwera";

fn build_charcounter(length: usize) -> Result<String, Box<dyn std::error::Error>> {
    assert!(length <= INPUT.len());
    let input: String = INPUT.chars().take(length).collect();

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
    let output = build_charcounter(299).unwrap();
    assert_eq!(output, "\x1b[0;31m299/300\x1b[0m");
}

#[test]
fn blank_when_given_max_characters() {
    let output = build_charcounter(300).unwrap();
    assert_eq!(output, "\x1b[1;31m300/300\x1b[0m");
}
