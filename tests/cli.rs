use assert_cmd::{assert::Assert, prelude::*};
use predicates::prelude::*;
use std::{
    env,
    path::Path,
    process::{Command, Output},
};

#[extend::ext]
impl Command {
    fn assert_stripped(&mut self) -> Assert {
        let output = self.unwrap();
        Assert::new(Output {
            stdout: strip_ansi_escapes::strip(output.stdout).unwrap(),
            stderr: strip_ansi_escapes::strip(output.stderr).unwrap(),
            ..output
        })
    }
}

fn cmp_or_override(name: &str) -> impl '_ + Predicate<[u8]> {
    predicate::function(move |content| {
        let path = format!("tests/cli/{name}.stdout");
        let path = Path::new(&path);
        if matches!(env::var("NERDFIX_TEST").as_deref(), Ok("overwrite")) {
            std::fs::write(path, content).unwrap();
        }
        let expected = std::fs::read(path).unwrap_or_default();
        content == expected
    })
}

#[test]
fn check() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("check")
        .arg("tests/test-data.txt")
        .assert_stripped()
        .success()
        .stdout(cmp_or_override("check"));
}

#[test]
fn check_with_input() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("-i")
        .arg("tests/test-input.txt")
        .arg("check")
        .arg("tests/test-data.txt")
        .assert_stripped()
        .success()
        .stdout(cmp_or_override("check_with_input"));
}

#[test]
fn check_json() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg("tests/test-data.txt")
        .assert_stripped()
        .success()
        .stdout(cmp_or_override("check_json"));
}
