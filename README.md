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
let hidden = strip_sensitive_literals(
    &doc,
    StripOptions { hide_list_and_object_literals: true },
);
// user(name: "", age: 0, tags: []) { id }
```

The strongest defense is to avoid inline literals. Pass user data through
GraphQL variables so it never reaches the operation text. This crate is the
backstop for operations that still inline data.

## License

Licensed under the [MIT license](LICENSE).
