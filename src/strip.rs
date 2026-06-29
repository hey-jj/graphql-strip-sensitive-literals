//! The redaction walk over a GraphQL document AST.

use apollo_compiler::ast::{
    Argument, Definition, Directive, Document, Field, FragmentDefinition, InlineFragment, IntValue,
    OperationDefinition, Selection, Value, VariableDefinition,
};
use apollo_compiler::Node;

use crate::StripOptions;

/// Return a redacted copy of `ast`.
///
/// See [`crate::strip_sensitive_literals`] for the contract. This clones the
/// input and mutates the clone in place, so the caller's document is untouched.
pub(crate) fn strip(ast: &Document, options: StripOptions) -> Document {
    let mut out = ast.clone();
    for def in &mut out.definitions {
        strip_definition(def, options);
    }
    out
}

fn strip_definition(def: &mut Definition, options: StripOptions) {
    match def {
        Definition::OperationDefinition(node) => strip_operation(node.make_mut(), options),
        Definition::FragmentDefinition(node) => strip_fragment(node.make_mut(), options),
        // Type system definitions carry no executable literals this function
        // scrubs. Leave them as they are.
        _ => {}
    }
}

fn strip_operation(op: &mut OperationDefinition, options: StripOptions) {
    for var in &mut op.variables {
        strip_variable_definition(var.make_mut(), options);
    }
    strip_directives(&mut op.directives.0, options);
    strip_selection_set(&mut op.selection_set, options);
}

fn strip_fragment(frag: &mut FragmentDefinition, options: StripOptions) {
    strip_directives(&mut frag.directives.0, options);
    strip_selection_set(&mut frag.selection_set, options);
}

fn strip_variable_definition(var: &mut VariableDefinition, options: StripOptions) {
    if let Some(default) = &mut var.default_value {
        strip_value(default.make_mut(), options);
    }
    strip_directives(&mut var.directives.0, options);
}

fn strip_selection_set(set: &mut [Selection], options: StripOptions) {
    for selection in set {
        match selection {
            Selection::Field(node) => strip_field(node.make_mut(), options),
            Selection::InlineFragment(node) => strip_inline_fragment(node.make_mut(), options),
            Selection::FragmentSpread(node) => {
                strip_directives(&mut node.make_mut().directives.0, options);
            }
        }
    }
}

fn strip_field(field: &mut Field, options: StripOptions) {
    strip_arguments(&mut field.arguments, options);
    strip_directives(&mut field.directives.0, options);
    strip_selection_set(&mut field.selection_set, options);
}

fn strip_inline_fragment(frag: &mut InlineFragment, options: StripOptions) {
    strip_directives(&mut frag.directives.0, options);
    strip_selection_set(&mut frag.selection_set, options);
}

fn strip_directives(directives: &mut [Node<Directive>], options: StripOptions) {
    for directive in directives {
        strip_arguments(&mut directive.make_mut().arguments, options);
    }
}

fn strip_arguments(arguments: &mut [Node<Argument>], options: StripOptions) {
    for argument in arguments {
        strip_value(argument.make_mut().value.make_mut(), options);
    }
}

/// Redact a single value in place.
///
/// Integers and floats become the literal `0`. Strings become the empty string.
/// With [`StripOptions::hide_list_and_object_literals`] set, lists and objects
/// are emptied and their old contents are dropped without recursion. Otherwise
/// the walk descends into list elements and object field values.
///
/// Booleans, nulls, enums, and variables are left as they are.
fn strip_value(value: &mut Value, options: StripOptions) {
    match value {
        // Both Int and Float collapse to the literal 0. graphql-js stores the
        // placeholder as the string "0" inside the matching value node, which
        // prints as a bare `0`. Modeling both as an Int holding "0" reproduces
        // that printed form. A float placeholder must print as `0`, not `0.0`,
        // so it cannot stay a float node here.
        Value::Int(_) | Value::Float(_) => {
            *value = Value::Int(IntValue::new_parsed("0"));
        }
        Value::String(s) => s.clear(),
        Value::List(items) => {
            if options.hide_list_and_object_literals {
                items.clear();
            } else {
                for item in items {
                    strip_value(item.make_mut(), options);
                }
            }
        }
        Value::Object(fields) => {
            if options.hide_list_and_object_literals {
                fields.clear();
            } else {
                for (_name, field_value) in fields {
                    strip_value(field_value.make_mut(), options);
                }
            }
        }
        Value::Boolean(_) | Value::Null | Value::Enum(_) | Value::Variable(_) => {}
    }
}
