# graphql-strip-sensitive-literals

Redact sensitive literals from a GraphQL document before logging or usage reporting.

GraphQL operations often inline user data as literal values. A query like
`user(email: "a@b.com")` carries the email in the operation text. Logging or
reporting that text leaks the data. This crate rewrites the literals to fixed
placeholders so the operation shape stays useful while the data is gone.

## What it does

`strip_sensitive_literals` returns a redacted copy of a parsed document:

- integers and floats become `0`
- strings become `""`
- with `hide_list_and_object_literals` set, lists become `[]` and objects become `{}`

Everything else is preserved. Field names, aliases, variable names and types,
fragment names, and directives stay as they are. Booleans, nulls, enums, and
variable references are not treated as sensitive and are left untouched.

The walk covers the whole document. Operations, fragments, and type-system
definitions are all redacted, so literals in field-argument defaults,
input-field defaults, and directive arguments collapse the same way.

The input document is never modified. The result is a fresh owned tree. Running
the function twice gives the same result as running it once.

## Installation

```toml
[dependencies]
graphql-strip-sensitive-literals = "0.1"
```

## Usage

```rust
use apollo_compiler::ast::Document;
use graphql_strip_sensitive_literals::{strip_sensitive_literals, StripOptions};

let doc = Document::parse(
    r#"{ user(name: "Ada", age: 36, tags: ["x", "y"]) { id } }"#,
    "query.graphql",
)
.unwrap();

// Default: redact scalars, keep list and object shape.
let redacted = strip_sensitive_literals(&doc, StripOptions::default());
// user(name: "", age: 0, tags: ["", ""]) { id }

// Opt in to empty lists and objects too.
let opts = StripOptions::builder()
    .hide_list_and_object_literals(true)
    .build();
let hidden = strip_sensitive_literals(&doc, opts);
// user(name: "", age: 0, tags: []) { id }
```

The strongest defense is to avoid inline literals. Pass user data through
GraphQL variables so it never reaches the operation text. This crate is the
backstop for operations that still inline data.

## apollo-compiler version

The `Document` type comes from `apollo-compiler` and appears in the public API.
Build input with `Document::parse`, which means you depend on `apollo-compiler`
directly. That ties this crate to the `apollo-compiler` major version. The
dependency is pinned to `1.x`. Pass a `Document` across the boundary only when
both crates resolve to the same `apollo-compiler` major.

## License

Licensed under the [MIT license](LICENSE).
