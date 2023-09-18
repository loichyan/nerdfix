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

fn cmp_or_override(file: &str) -> impl '_ + Predicate<[u8]> {
    predicate::function(move |content| {
        let path = Path::new("tests/cli").join(file);
        if matches!(env::var("NERDFIX_TEST").as_deref(), Ok("overwrite")) {
            std::fs::write(path, content).unwrap();
            true
        } else {
            content == std::fs::read(path).unwrap_or_default()
        }
    })
}

fn test_cli(name: &str, args: &[&str]) {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .args(args)
        .assert_stripped()
        .success()
        .stdout(cmp_or_override(&format!("{}.stdout", name)))
        .stderr(cmp_or_override(&format!("{}.stderr", name)));
}

macro_rules! test_cli {
    ($name:expr, $($args:expr),* $(,)?) => {
        test_cli($name, &[$($args,)*]);
    };
}

#[test]
fn check() {
    test_cli!("check", "check", "tests/test-data.txt");
}

#[test]
fn check_with_input() {
    test_cli!(
        "check_with_input",
        "check",
        "--input=tests/test-index.json",
        "tests/test-data.txt"
    );
}

#[test]
fn check_json() {
    test_cli!(
        "check_json",
        "check",
        "--format=json",
        "tests/test-data.txt"
    );
}

#[test]
fn fix() {
    test_cli!(
        "fix",
        "fix",
        "--select-first",
        "--write",
        "tests/test-data.txt:-"
    );
}

#[test]
fn fix_with_exact_subs() {
    test_cli!(
        "fix_with_exact_subs",
        "fix",
        "--select-first",
        "--write",
        "--input=tests/test-substitution.json",
        "--input=src/index.json",
        "tests/test-data.txt:-"
    );
}

#[test]
fn fix_with_prefix_subs() {
    test_cli!(
        "fix_with_prefix_subs",
        "fix",
        "--select-first",
        "--write",
        "--sub=prefix:mdi-/md-",
        "tests/test-data.txt:-"
    );
}
