//! Redaction reaches literals in type-system definitions.
//!
//! A parsed document can hold schema definitions next to operations. Literals
//! live in field-argument defaults, input-field defaults, and directive
//! arguments. The redactor walks the whole document, so those literals collapse
//! the same as literals in a query.

use apollo_compiler::ast::Document;
use graphql_strip_sensitive_literals::{strip_sensitive_literals, StripOptions};

fn strip_inline(src: &str, opts: StripOptions) -> String {
    let doc = Document::parse(src, "in.graphql").unwrap();
    let out = strip_sensitive_literals(&doc, opts);
    out.to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

struct Row {
    input: &'static str,
    expected: &'static str,
}

const ROWS: &[Row] = &[
    Row {
        input: "input I { x: Int = 5 }",
        expected: "input I { x: Int = 0 }",
    },
    Row {
        input: "type T @foo(n: 5) { id: ID }",
        expected: "type T @foo(n: 0) { id: ID }",
    },
    Row {
        input: "type Q { f(x: Int = 5): ID }",
        expected: "type Q { f(x: Int = 0): ID }",
    },
    Row {
        input: r#"enum E { A @deprecated(reason: "old") }"#,
        expected: r#"enum E { A @deprecated(reason: "") }"#,
    },
    Row {
        input: "directive @foo(n: Int = 3) on FIELD",
        expected: "directive @foo(n: Int = 0) on FIELD",
    },
    Row {
        input: "interface If { g(a: Int = 4): ID }",
        expected: "interface If { g(a: Int = 0): ID }",
    },
    Row {
        input: r#"extend input I @foo(s: "x") { y: Int = 9 }"#,
        expected: r#"extend input I @foo(s: "") { y: Int = 0 }"#,
    },
];

#[test]
fn type_system_literals_redacted() {
    for row in ROWS {
        let got = strip_inline(row.input, StripOptions::default());
        assert_eq!(got, row.expected, "input `{}`", row.input);
    }
}

#[test]
fn mixed_document_redacts_both_halves() {
    let src = "query Q { f(n: 5) }\ninput I { x: Int = 7 }";
    let got = strip_inline(src, StripOptions::default());
    assert_eq!(got, "query Q { f(n: 0) } input I { x: Int = 0 }");
}

#[test]
fn schema_directive_argument_redacted() {
    let got = strip_inline(
        r#"schema @meta(tag: "secret") { query: Q }"#,
        StripOptions::default(),
    );
    assert_eq!(got, r#"schema @meta(tag: "") { query: Q }"#);
}

#[test]
fn input_default_list_emptied_when_hidden() {
    let opts = StripOptions::builder()
        .hide_list_and_object_literals(true)
        .build();
    let got = strip_inline("input I { x: [Int] = [1, 2, 3] }", opts);
    assert_eq!(got, "input I { x: [Int] = [] }");
}
