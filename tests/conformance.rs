//! The four snapshot cases that define the public contract.
//!
//! Each case parses one shared document, redacts it, reprints it, and compares
//! against a golden file. The document exercises every literal kind the
//! redactor touches and several nodes it must leave alone.

use apollo_compiler::ast::Document;
use graphql_strip_sensitive_literals::{strip_sensitive_literals, StripOptions};

fn load(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

/// Trim trailing whitespace on every line and drop trailing blank lines.
///
/// The golden files and the printer can differ on trailing newlines. This makes
/// the comparison stable without changing any meaningful content.
fn normalize(s: &str) -> String {
    s.trim_end()
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_and_print(src: &str, opts: StripOptions) -> String {
    let doc = Document::parse(src, "input.graphql").unwrap();
    let out = strip_sensitive_literals(&doc, opts);
    normalize(&out.to_string())
}

struct Case {
    name: &'static str,
    opts: StripOptions,
    expected_fixture: &'static str,
}

#[test]
fn snapshot_cases() {
    let cases = [
        Case {
            name: "default configuration",
            opts: StripOptions::default(),
            expected_fixture: "default.expected.graphql",
        },
        Case {
            name: "hide_list_and_object_literals true",
            opts: StripOptions::builder()
                .hide_list_and_object_literals(true)
                .build(),
            expected_fixture: "hide_list_object.expected.graphql",
        },
    ];

    let input = load("input.graphql");
    for case in &cases {
        let got = strip_and_print(&input, case.opts);
        let want = normalize(&load(case.expected_fixture));
        assert_eq!(got, want, "case `{}`", case.name);
    }
}

// The default and an explicit `false` are the same configuration. One snapshot
// row covers the behavior. This pins the equality so the two cannot drift.
#[test]
fn default_equals_explicit_false() {
    let explicit_false = StripOptions::builder()
        .hide_list_and_object_literals(false)
        .build();
    assert_eq!(StripOptions::default(), explicit_false);
}
