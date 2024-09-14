//! This crate provides the user with a macro, `optional_struct`, which allows
//! the user to generate structures with optional fields, as well as functions
//! for "fusing" such structures to form a concrete instance of the original,
//! user-defined structure. This was developed with the goal of simplifying
//! aggregating configurations coming from different sources, such as e.g. file,
//! env, CLI, etc.

/// The core of this crate. Call this proc macro on your structures to
/// generate another structure containing provided Enum-wrapped fields, as
/// well as helpers functions to convert those enumified_struct to
/// their base, or even update only fields that have been set. This makes
/// aggregating structures from different sources (e.g. configuration from
/// file/env/CLI) simple. The generated struct by default will wrap all
/// fields in an the targeted enum, unless the field already is the target
/// enum. There are however other attributes that one can use to enforce a
/// different behaviour:
/// enumify_rename => rename the type in the generated structure. Useful
/// when the nested structure itself has an enumified_struct. This enables
/// arbitrary nesting of enumified_struct enumify_skip_wrap => this forces
/// the macro to *not* wrapping a value, e.g. `T` stays `T`. This is
/// enabled by default if `T` is already `TargetEnum<U>`.
/// enumify_wrap => this forces wrapping a value, e.g. `U` becomes
/// `TargetEnum<U>`. Enabling this allows nested `TargetEnum`, e.g.
/// `TargetEnum<V>` can become `TargetEnum<TargetEnum<V>>`
pub use enumify_macro::enumify_struct;

pub trait ResolveToBase<T> {
    fn resolve_to_base(&self) -> T;
}

/// The trait is implemented for every generated structure. Thanks to this, you
/// can use enumify_struct in generic contexts.
pub trait Applicable: Sized {
    /// This is the type the optional_struct macro was used on. We need the type
    /// to be able to generate methods generating such structures.
    type Base;

    /// This function applies all the fields set in this structure to an
    /// instance of its Base. Note that this works recursively, enabling the
    /// use of nested optional_struct structures.
    fn build(self, mut base: Self::Base) -> Self::Base {
        self.apply_to(&mut base);
        base
    }

    /// Similar to `Applicable::build`, but takes the Base by reference.
    fn apply_to(self, base: &mut Self::Base);
}
