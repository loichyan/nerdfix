// NOTE: ignored on Windows as CRLF causes differences in spans
#![cfg(unix)]

use assert_cmd::{assert::Assert, prelude::*};
use core::fmt;
use predicates::prelude::*;
use std::{
    env,
    path::Path,
    process::{Command, Output},
};

fn normalize_output(bytes: Vec<u8>) -> Vec<u8> {
    strip_ansi_escapes::strip(bytes)
}

#[extend::ext]
impl Command {
    fn assert_stripped(&mut self) -> Assert {
        let output = self.unwrap();
        Assert::new(Output {
            stdout: normalize_output(output.stdout),
            stderr: normalize_output(output.stderr),
            ..output
        })
    }
}

// TODO: respect find_case
struct BoxedPredicate<V: ?Sized>(Box<dyn Predicate<V>>);

impl<V: ?Sized> BoxedPredicate<V> {
    pub fn new<P>(pred: P) -> Self
    where
        P: 'static + Predicate<V>,
    {
        Self(Box::new(pred))
    }
}

impl<V: ?Sized> fmt::Display for BoxedPredicate<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<V: ?Sized> predicates::reflection::PredicateReflection for BoxedPredicate<V> {
    fn parameters<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = predicates::reflection::Parameter<'a>> + 'a> {
        self.0.parameters()
    }

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = predicates::reflection::Child<'a>> + 'a> {
        self.0.children()
    }
}

impl<V: ?Sized> predicates::Predicate<V> for BoxedPredicate<V> {
    fn eval(&self, variable: &V) -> bool {
        self.0.eval(variable)
    }

    fn find_case<'a>(
        &'a self,
        expected: bool,
        variable: &V,
    ) -> Option<predicates::reflection::Case<'a>> {
        self.0.find_case(expected, variable)
    }
}

fn cmp_or_override(file: &str) -> impl '_ + Predicate<[u8]> {
    let path = Path::new("tests").join(file);
    if matches!(env::var("NERDFIX_TEST").as_deref(), Ok("overwrite")) {
        BoxedPredicate::new(predicate::function(move |expected: &[u8]| {
            std::fs::write(&path, expected).unwrap();
            true
        }))
    } else {
        let expected = std::fs::read_to_string(path).unwrap();
        BoxedPredicate::new(predicate::str::diff(expected).from_utf8())
    }
}

fn test_cli(name: &str, args: &[&str]) {
    Command::cargo_bin("nerdfix")
        .unwrap()
        .args(args)
        .assert_stripped()
        .success()
        .stdout(cmp_or_override(&format!("cli/{}.stdout", name)))
        .stderr(cmp_or_override(&format!("cli/{}.stderr", name)));
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
fn check_with_db() {
    test_cli!(
        "check_with_db",
        "check",
        "-i=tests/test-icons.json",
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
        "-i=src/icons.json",
        "-i=tests/test-substitutions.json",
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
        "-i=src/icons.json",
        "--sub=prefix:mdi-/md-",
        "tests/test-data.txt:-"
    );
}
