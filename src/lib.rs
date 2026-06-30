//! Redact sensitive literals from a GraphQL document before logging or usage
//! reporting.
//!
//! GraphQL operations often inline user data as literal values. A query like
//! `user(email: "a@b.com")` carries the email in the operation text. If that
//! text is logged or sent to a usage-reporting service, the data leaks. This
//! crate rewrites those literals to fixed placeholders so the operation shape
//! stays useful while the data is gone.
//!
//! [`strip_sensitive_literals`] returns a redacted copy of a parsed document.
//! Scalar literals are replaced:
//!
//! - integers and floats become `0`
//! - strings become `""`
//!
//! With [`StripOptions::hide_list_and_object_literals`] set, list literals
//! become `[]` and object literals become `{}`.
//!
//! Everything else is left as it is. Field names, aliases, variable names and
//! types, fragment names, directives, and the `Boolean`, `Null`, `Enum`, and
//! variable values stay untouched. Booleans, nulls, and enums are not treated
//! as sensitive.
//!
//! The walk covers the whole document. Operations and fragments are redacted.
//! Type-system definitions are too, so literals in field-argument defaults,
//! input-field defaults, and directive arguments collapse the same way.
//!
//! The best defense is to avoid inline literals. Pass user data through GraphQL
//! variables so it never appears in the operation text. This crate is the
//! backstop for operations that still inline data.
//!
//! # apollo-compiler in the public API
//!
//! The document type is [`apollo_compiler::ast::Document`], re-exported here as
//! [`Document`]. It appears in the signature of [`strip_sensitive_literals`], so
//! it is part of this crate's public API. A caller builds input with
//! `Document::parse` from `apollo-compiler`, which means the caller depends on
//! `apollo-compiler` directly. The re-export names the boundary type so call
//! sites can reach it through one path.
//!
//! This couples the crate to the `apollo-compiler` major version. The dependency
//! is pinned to `1.x`. A consumer that passes a `Document` across this boundary
//! must resolve to the same `apollo-compiler` major. An `apollo-compiler` `2.0`
//! would change this crate's public type and require a matching major bump here.
//!
//! # Example
//!
//! ```
//! use apollo_compiler::ast::Document;
//! use graphql_strip_sensitive_literals::{strip_sensitive_literals, StripOptions};
//!
//! let doc = Document::parse(
//!     r#"{ user(name: "Ada", age: 36) { id } }"#,
//!     "query.graphql",
//! )
//! .unwrap();
//!
//! let redacted = strip_sensitive_literals(&doc, StripOptions::default());
//! assert_eq!(
//!     redacted.to_string().trim(),
//!     "{\n  user(name: \"\", age: 0) {\n    id\n  }\n}",
//! );
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod options;
mod strip;

pub use apollo_compiler::ast::Document;
pub use options::{StripOptions, StripOptionsBuilder};

/// Return a redacted copy of `ast`.
///
/// Scalar literals are replaced with fixed placeholders. Integers and floats
/// become `0`, strings become `""`. When `options.hide_list_and_object_literals`
/// is set, list literals become `[]` and object literals become `{}`. In the
/// default mode lists and objects keep their shape and their nested scalars are
/// redacted instead.
///
/// Non-literal nodes are preserved. So are `Boolean`, `Null`, `Enum`, and
/// variable values, which are not treated as sensitive.
///
/// The input is not modified. The returned [`Document`] is a fresh owned tree.
/// Applying the function twice gives the same result as applying it once.
///
/// # Example
///
/// ```
/// use apollo_compiler::ast::Document;
/// use graphql_strip_sensitive_literals::{strip_sensitive_literals, StripOptions};
///
/// let doc = Document::parse(
///     r#"{ f(list: [1, 2], obj: { k: "v" }) }"#,
///     "query.graphql",
/// )
/// .unwrap();
///
/// // Default: keep list and object shape, redact the scalars inside.
/// let kept = strip_sensitive_literals(&doc, StripOptions::default());
/// assert!(kept.to_string().contains(r#"f(list: [0, 0], obj: {k: ""})"#));
///
/// // Opt in: empty the list and the object.
/// let opts = StripOptions::builder()
///     .hide_list_and_object_literals(true)
///     .build();
/// let hidden = strip_sensitive_literals(&doc, opts);
/// assert!(hidden.to_string().contains("f(list: [], obj: {})"));
/// ```
#[must_use = "redaction returns a new document and does not modify `ast` in place"]
pub fn strip_sensitive_literals(ast: &Document, options: StripOptions) -> Document {
    strip::strip(ast, options)
}
