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
            stdout: strip_ansi_escapes::strip(output.stdout),
            stderr: strip_ansi_escapes::strip(output.stderr),
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
fn check_with_indices() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("check")
        .arg("--input")
        .arg("tests/test-index.json")
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

#[cfg(unix)]
#[test]
fn fix() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("fix")
        .arg("--select-first")
        .arg("--write")
        .arg("tests/test-data.txt:/dev/null")
        .assert_stripped()
        .success()
        .stdout(cmp_or_override("fix"));
}

#[cfg(unix)]
#[test]
fn fix_with_replacements() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("fix")
        .arg("--select-first")
        .arg("--write")
        .arg("--replace")
        .arg("mdi-,md-")
        .arg("tests/test-data.txt:/dev/null")
        .assert_stripped()
        .success()
        .stdout(cmp_or_override("fix_with_replacements"));
}

#[cfg(unix)]
#[test]
fn fix_with_substitutions() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("fix")
        .arg("--select-first")
        .arg("--write")
        .arg("--input")
        .arg("INDICES")
        .arg("--input")
        .arg("tests/test-substitution.json")
        .arg("tests/test-data.txt:/dev/null")
        .assert_stripped()
        .success()
        .stdout(cmp_or_override("fix_with_substitutions"));
}
