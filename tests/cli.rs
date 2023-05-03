use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{path::Path, process::Command};

fn cmp_or_override(name: &str) -> impl '_ + Predicate<[u8]> {
    predicate::function(move |content| {
        let path = format!("tests/cli/{name}.stdout");
        let path = Path::new(&path);
        let content = strip_ansi_escapes::strip(content).unwrap();
        if path.exists() {
            let expected = std::fs::read(path).unwrap();
            content == expected
        } else {
            std::fs::write(path, content).unwrap();
            true
        }
    })
}

#[test]
fn check() {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .arg("check")
        .arg("tests/test-data.txt")
        .assert()
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
        .assert()
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
        .assert()
        .success()
        .stdout(cmp_or_override("check_json"));
}
