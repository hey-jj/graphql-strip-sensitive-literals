//! Configuration for [`strip_sensitive_literals`](crate::strip_sensitive_literals).

/// Controls how aggressively literals are redacted.
///
/// The default redacts scalar literals only. Scalars are integers, floats, and
/// strings. List and object literals keep their shape and the redactor recurses
/// into their elements.
///
/// Set [`hide_list_and_object_literals`](Self::hide_list_and_object_literals) to
/// `true` to also empty every list and object literal. An empty list or object
/// reveals nothing about the data it held, including its length and its keys.
///
/// An absent option and an explicit `false` behave the same. Only `true` turns
/// list and object emptying on.
///
/// # Examples
///
/// ```
/// use graphql_strip_sensitive_literals::StripOptions;
///
/// // Default keeps list and object structure.
/// let keep = StripOptions::default();
/// assert!(!keep.hide_list_and_object_literals);
///
/// // Opt in to empty lists and objects.
/// let hide = StripOptions { hide_list_and_object_literals: true };
/// assert!(hide.hide_list_and_object_literals);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct StripOptions {
    /// Empty every list and object literal when `true`.
    ///
    /// A list becomes `[]` and an object becomes `{}`. The redactor drops the
    /// contents and does not recurse into them. Default is `false`, which keeps
    /// the structure and redacts the nested scalars instead.
    pub hide_list_and_object_literals: bool,
}
