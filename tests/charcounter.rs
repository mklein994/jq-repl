use std::process::{Command, Stdio};

const INPUT: &str = "IpsumvoluptatesasperioresquisquamaliasullamxtemporibusNisicommodiquaeratasperioresnemosuntoImpeditconsecteturassumendaofficiaitaquererumporroDolorestemporaadipisciplaceatcommodiquasexcepturifugitEnimnihilmolestiaslkjhaaaalkjhasfdlkjhasfdaoiuqewrqljaoiuvzvczvcfdsaopqljqjafdsjhasfljlasdlkjsafdoiuqwera";

fn build_charcounter(length: usize) -> String {
    assert!(length <= INPUT.len());
    let input: String = INPUT.chars().take(length).collect();
    let echo = Command::new("echo")
        .args(["-n", &input])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    String::from_utf8(
        Command::new(env!("CARGO_BIN_EXE_charcounter"))
            .stdin(echo.stdout.unwrap())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
}

#[test]
fn blank_when_given_few_characters() {
    let output = build_charcounter(2);
    assert!(output.is_empty());
}

#[test]
fn blank_when_given_almost_max_characters() {
    let output = build_charcounter(299);
    assert_eq!(output, "\x1b[0;31m299/300\x1b[0m");
}

#[test]
fn blank_when_given_max_characters() {
    let output = build_charcounter(300);
    assert_eq!(output, "\x1b[1;31m300/300\x1b[0m");
}
