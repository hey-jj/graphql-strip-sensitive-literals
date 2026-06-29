//! Edge cases the shared snapshot document does not isolate.
//!
//! These pin the "do not strip" contract for booleans, nulls, and enums, the
//! numeric collapse to `0`, block strings, nesting, idempotence, and that the
//! input is never mutated.

use apollo_compiler::ast::Document;
use graphql_strip_sensitive_literals::{strip_sensitive_literals, StripOptions};

fn default() -> StripOptions {
    StripOptions::default()
}

fn hide() -> StripOptions {
    StripOptions::builder()
        .hide_list_and_object_literals(true)
        .build()
}

/// Redact `src` and return the reprinted document, collapsed to one line so a
/// single short query is easy to assert against.
fn strip_inline(src: &str, opts: StripOptions) -> String {
    let doc = Document::parse(src, "in.graphql").unwrap();
    let out = strip_sensitive_literals(&doc, opts);
    out.to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn assert_contains(src: &str, opts: StripOptions, needle: &str) {
    let got = strip_inline(src, opts);
    assert!(
        got.contains(needle),
        "got `{got}`, expected to contain `{needle}`"
    );
}

#[test]
fn bool_literal_kept() {
    assert_contains(
        "{ f(flag: true, off: false) }",
        default(),
        "f(flag: true, off: false)",
    );
}

#[test]
fn null_literal_kept() {
    assert_contains("{ f(x: null) }", default(), "f(x: null)");
}

#[test]
fn enum_literal_kept() {
    assert_contains("{ f(dir: NORTH) }", default(), "f(dir: NORTH)");
}

#[test]
fn negative_int_to_zero() {
    assert_contains("{ f(n: -3) }", default(), "f(n: 0)");
}

#[test]
fn big_int_to_zero() {
    assert_contains("{ f(n: 9999999999) }", default(), "f(n: 0)");
}

#[test]
fn exponent_float_to_zero() {
    // Must print as a bare `0`, never `0.0`.
    let got = strip_inline("{ f(n: 1.5e10) }", default());
    assert!(got.contains("f(n: 0)"), "got `{got}`");
    assert!(!got.contains("0.0"), "got `{got}`");
}

#[test]
fn plain_float_to_zero() {
    let got = strip_inline("{ f(n: 0.4) }", default());
    assert!(got.contains("f(n: 0)"), "got `{got}`");
    assert!(!got.contains("0.0"), "got `{got}`");
}

#[test]
fn block_string_to_empty() {
    // A block string becomes a normal empty string, never a triple-quoted block.
    let got = strip_inline(r#"{ f(s: """hi""") }"#, default());
    assert!(got.contains(r#"f(s: "")"#), "got `{got}`");
    assert!(!got.contains(r#"""""#), "got `{got}`");
}

#[test]
fn unicode_and_escaped_string_to_empty() {
    // Redaction replaces the whole value, so multibyte text and escape
    // sequences must collapse to the empty string with nothing left behind.
    let got = strip_inline(r#"{ f(s: "héllo \n 🎉") }"#, default());
    assert!(got.contains(r#"f(s: "")"#), "got `{got}`");
    assert!(!got.contains("héllo"), "got `{got}`");
    assert!(!got.contains('🎉'), "got `{got}`");
}

#[test]
fn nested_list_of_objects_hide_true() {
    assert_contains("{ f(l: [{a: 1}, {b: 2}]) }", hide(), "f(l: [])");
}

#[test]
fn nested_object_default_keeps_structure() {
    assert_contains("{ f(o: {a: {b: 1}}) }", default(), r#"f(o: {a: {b: 0}})"#);
}

#[test]
fn nested_list_default_redacts_inner_scalars() {
    assert_contains(
        r#"{ f(l: ["x", 2, 3.5]) }"#,
        default(),
        r#"f(l: ["", 0, 0])"#,
    );
}

#[test]
fn mutation_op_covered() {
    assert_contains(
        r#"mutation M { set(v: "x", n: 1) }"#,
        default(),
        r#"set(v: "", n: 0)"#,
    );
}

#[test]
fn subscription_op_covered() {
    assert_contains(
        r#"subscription S { watch(id: "abc") }"#,
        default(),
        r#"watch(id: "")"#,
    );
}

#[test]
fn empty_object_list_idempotent_default() {
    assert_contains("{ f(l: [], o: {}) }", default(), "f(l: [], o: {})");
}

#[test]
fn no_literals_unchanged() {
    assert_contains("{ a b c }", default(), "{ a b c }");
}

#[test]
fn mixed_with_variables() {
    // A variable reference is kept, the int next to it is stripped.
    assert_contains(
        "query ($v: Int) { f(a: $v, b: 1) }",
        default(),
        "f(a: $v, b: 0)",
    );
}

#[test]
fn variable_default_value_is_stripped() {
    // graphql-js visit reaches default values too. A default int collapses to 0.
    assert_contains("query ($v: Int = 5) { f(a: $v) }", default(), "$v: Int = 0");
}

#[test]
fn directive_argument_stripped() {
    assert_contains(
        r#"{ f @tag(note: "secret") }"#,
        default(),
        r#"@tag(note: "")"#,
    );
}

#[test]
fn input_not_mutated() {
    let src = r#"{ user(name: "Ada", age: 36) { id } }"#;
    let doc = Document::parse(src, "in.graphql").unwrap();

    let _ = strip_sensitive_literals(&doc, default());

    // The source document still prints its original literals.
    let reprinted = doc
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(reprinted.contains(r#"name: "Ada""#), "got `{reprinted}`");
    assert!(reprinted.contains("age: 36"), "got `{reprinted}`");
}

#[test]
fn idempotent() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/input.graphql"),
    )
    .unwrap();
    let doc = Document::parse(&src, "input.graphql").unwrap();

    for opts in [default(), hide()] {
        let once = strip_sensitive_literals(&doc, opts);
        let twice = strip_sensitive_literals(&once, opts);
        assert_eq!(
            once.to_string(),
            twice.to_string(),
            "strip is not idempotent for {opts:?}",
        );
    }
}
