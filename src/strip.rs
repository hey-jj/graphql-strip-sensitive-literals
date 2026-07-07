//! The redaction walk over a GraphQL document AST.

use apollo_compiler::ast::{
    Argument, Definition, Directive, DirectiveDefinition, Document, EnumTypeDefinition,
    EnumTypeExtension, EnumValueDefinition, Field, FieldDefinition, FragmentDefinition,
    InlineFragment, InputObjectTypeDefinition, InputObjectTypeExtension, InputValueDefinition,
    IntValue, InterfaceTypeDefinition, InterfaceTypeExtension, ObjectTypeDefinition,
    ObjectTypeExtension, OperationDefinition, Selection, Value, VariableDefinition,
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
        Definition::DirectiveDefinition(node) => {
            strip_directive_definition(node.make_mut(), options)
        }
        Definition::SchemaDefinition(node) => {
            let def = node.make_mut();
            strip_description(&mut def.description);
            strip_directives(&mut def.directives.0, options);
        }
        Definition::SchemaExtension(node) => {
            strip_directives(&mut node.make_mut().directives.0, options)
        }
        Definition::ScalarTypeDefinition(node) => {
            let def = node.make_mut();
            strip_description(&mut def.description);
            strip_directives(&mut def.directives.0, options);
        }
        Definition::ScalarTypeExtension(node) => {
            strip_directives(&mut node.make_mut().directives.0, options)
        }
        Definition::ObjectTypeDefinition(node) => strip_object_type(node.make_mut(), options),
        Definition::ObjectTypeExtension(node) => {
            strip_object_type_extension(node.make_mut(), options)
        }
        Definition::InterfaceTypeDefinition(node) => strip_interface_type(node.make_mut(), options),
        Definition::InterfaceTypeExtension(node) => {
            strip_interface_type_extension(node.make_mut(), options)
        }
        Definition::UnionTypeDefinition(node) => {
            let def = node.make_mut();
            strip_description(&mut def.description);
            strip_directives(&mut def.directives.0, options);
        }
        Definition::UnionTypeExtension(node) => {
            strip_directives(&mut node.make_mut().directives.0, options)
        }
        Definition::EnumTypeDefinition(node) => strip_enum_type(node.make_mut(), options),
        Definition::EnumTypeExtension(node) => strip_enum_type_extension(node.make_mut(), options),
        Definition::InputObjectTypeDefinition(node) => {
            strip_input_object_type(node.make_mut(), options)
        }
        Definition::InputObjectTypeExtension(node) => {
            strip_input_object_type_extension(node.make_mut(), options)
        }
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

fn strip_directive_definition(def: &mut DirectiveDefinition, options: StripOptions) {
    strip_description(&mut def.description);
    strip_input_value_definitions(&mut def.arguments, options);
}

fn strip_object_type(def: &mut ObjectTypeDefinition, options: StripOptions) {
    strip_description(&mut def.description);
    strip_directives(&mut def.directives.0, options);
    strip_field_definitions(&mut def.fields, options);
}

fn strip_object_type_extension(def: &mut ObjectTypeExtension, options: StripOptions) {
    strip_directives(&mut def.directives.0, options);
    strip_field_definitions(&mut def.fields, options);
}

fn strip_interface_type(def: &mut InterfaceTypeDefinition, options: StripOptions) {
    strip_description(&mut def.description);
    strip_directives(&mut def.directives.0, options);
    strip_field_definitions(&mut def.fields, options);
}

fn strip_interface_type_extension(def: &mut InterfaceTypeExtension, options: StripOptions) {
    strip_directives(&mut def.directives.0, options);
    strip_field_definitions(&mut def.fields, options);
}

fn strip_enum_type(def: &mut EnumTypeDefinition, options: StripOptions) {
    strip_description(&mut def.description);
    strip_directives(&mut def.directives.0, options);
    strip_enum_values(&mut def.values, options);
}

fn strip_enum_type_extension(def: &mut EnumTypeExtension, options: StripOptions) {
    strip_directives(&mut def.directives.0, options);
    strip_enum_values(&mut def.values, options);
}

fn strip_input_object_type(def: &mut InputObjectTypeDefinition, options: StripOptions) {
    strip_description(&mut def.description);
    strip_directives(&mut def.directives.0, options);
    strip_input_value_definitions(&mut def.fields, options);
}

fn strip_input_object_type_extension(def: &mut InputObjectTypeExtension, options: StripOptions) {
    strip_directives(&mut def.directives.0, options);
    strip_input_value_definitions(&mut def.fields, options);
}

fn strip_field_definitions(fields: &mut [Node<FieldDefinition>], options: StripOptions) {
    for field in fields {
        let field = field.make_mut();
        strip_description(&mut field.description);
        strip_input_value_definitions(&mut field.arguments, options);
        strip_directives(&mut field.directives.0, options);
    }
}

fn strip_input_value_definitions(values: &mut [Node<InputValueDefinition>], options: StripOptions) {
    for value in values {
        strip_input_value_definition(value.make_mut(), options);
    }
}

fn strip_input_value_definition(value: &mut InputValueDefinition, options: StripOptions) {
    strip_description(&mut value.description);
    if let Some(default) = &mut value.default_value {
        strip_value(default.make_mut(), options);
    }
    strip_directives(&mut value.directives.0, options);
}

fn strip_enum_values(values: &mut [Node<EnumValueDefinition>], options: StripOptions) {
    for value in values {
        let value = value.make_mut();
        strip_description(&mut value.description);
        strip_directives(&mut value.directives.0, options);
    }
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

fn strip_description(description: &mut Option<Node<str>>) {
    if description.is_some() {
        *description = Some(Node::new_str(""));
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
                *items = Vec::new();
            } else {
                for item in items {
                    strip_value(item.make_mut(), options);
                }
            }
        }
        Value::Object(fields) => {
            if options.hide_list_and_object_literals {
                *fields = Vec::new();
            } else {
                for (_name, field_value) in fields {
                    strip_value(field_value.make_mut(), options);
                }
            }
        }
        Value::Boolean(_) | Value::Null | Value::Enum(_) | Value::Variable(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_inline(src: &str, options: StripOptions) -> Document {
        let doc = Document::parse(src, "in.graphql").unwrap();
        strip(&doc, options)
    }

    fn description_text(description: &Option<Node<str>>) -> Option<&str> {
        description.as_ref().map(|node| node.as_str())
    }

    #[test]
    fn description_strings_are_redacted() {
        let out = strip_inline(
            r#"
            "secret schema" schema { query: Query }
            "secret directive" directive @tag("secret directive arg" note: String = "Ada") on FIELD_DEFINITION
            "secret scalar" scalar S
            "secret type" type Query { "secret field" user("secret arg" name: String = "Ada"): String }
            "secret interface" interface Node { "secret interface field" id("secret interface arg" x: String = "Ada"): ID }
            "secret union" union Search = Query
            "secret enum" enum E { "secret enum value" A }
            "secret input" input I { "secret input field" x: String = "Ada" }
            "#,
            StripOptions::default(),
        );

        let printed = out.to_string();
        assert!(!printed.contains("secret"), "got `{printed}`");

        for def in &out.definitions {
            match def {
                Definition::DirectiveDefinition(node) => {
                    let def = node.as_ref();
                    assert_eq!(description_text(&def.description), Some(""));
                    assert_eq!(
                        description_text(&def.arguments[0].as_ref().description),
                        Some("")
                    );
                }
                Definition::SchemaDefinition(node) => {
                    assert_eq!(description_text(&node.as_ref().description), Some(""));
                }
                Definition::ScalarTypeDefinition(node) => {
                    assert_eq!(description_text(&node.as_ref().description), Some(""));
                }
                Definition::ObjectTypeDefinition(node) => {
                    let def = node.as_ref();
                    let field = def.fields[0].as_ref();
                    assert_eq!(description_text(&def.description), Some(""));
                    assert_eq!(description_text(&field.description), Some(""));
                    assert_eq!(
                        description_text(&field.arguments[0].as_ref().description),
                        Some("")
                    );
                }
                Definition::InterfaceTypeDefinition(node) => {
                    let def = node.as_ref();
                    let field = def.fields[0].as_ref();
                    assert_eq!(description_text(&def.description), Some(""));
                    assert_eq!(description_text(&field.description), Some(""));
                    assert_eq!(
                        description_text(&field.arguments[0].as_ref().description),
                        Some("")
                    );
                }
                Definition::UnionTypeDefinition(node) => {
                    assert_eq!(description_text(&node.as_ref().description), Some(""));
                }
                Definition::EnumTypeDefinition(node) => {
                    let def = node.as_ref();
                    assert_eq!(description_text(&def.description), Some(""));
                    assert_eq!(
                        description_text(&def.values[0].as_ref().description),
                        Some("")
                    );
                }
                Definition::InputObjectTypeDefinition(node) => {
                    let def = node.as_ref();
                    assert_eq!(description_text(&def.description), Some(""));
                    assert_eq!(
                        description_text(&def.fields[0].as_ref().description),
                        Some("")
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn hidden_list_and_object_values_drop_capacity() {
        let options = StripOptions::builder()
            .hide_list_and_object_literals(true)
            .build();
        let out = strip_inline(
            "{ f(l: [1, 2, 3, 4, 5], o: {a: 1, b: 2, c: 3, d: 4, e: 5}) }",
            options,
        );

        let Definition::OperationDefinition(operation) = &out.definitions[0] else {
            panic!("expected operation");
        };
        let Selection::Field(field) = &operation.selection_set[0] else {
            panic!("expected field");
        };
        let arguments = &field.arguments;

        let Value::List(items) = arguments[0].value.as_ref() else {
            panic!("expected list");
        };
        assert_eq!(items.len(), 0);
        assert_eq!(items.capacity(), 0);

        let Value::Object(fields) = arguments[1].value.as_ref() else {
            panic!("expected object");
        };
        assert_eq!(fields.len(), 0);
        assert_eq!(fields.capacity(), 0);
    }
}
