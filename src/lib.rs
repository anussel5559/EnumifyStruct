//! This crate provides the user with a macro, `enumify_struct`, which allows
//! the user to generate structures with enum-ed fields, as well as functions
//! for "fusing" such structures to form a concrete instance of the original,
//! user-defined structure. This was developed with the goal of simplifying
//! aggregating configurations coming from different sources, such as e.g. file,
//! env, CLI, etc.

/// The core of this crate. Call this proc macro on your structures to
/// generate another structure containing provided Enum-wrapped fields, as
/// well as helpers functions to convert those enumified_struct to
/// their base. This makes aggregating structures from different sources
/// (e.g. configuration from file/env/CLI) simple. The generated struct by
/// default will wrap all fields in the targeted enum, unless the field
/// already is the target enum. There are however other attributes that one
/// can use to enforce a different behaviour:
/// enumify_rename => rename the type in the generated structure. Useful
/// when the nested structure itself has an enumified_struct. This enables
/// arbitrary nesting of enumified_struct
/// enumify_skip_wrap => this forces the macro to *not* wrap a value,
/// e.g. `T` stays `T`. This is enabled by default if `T` is already
/// `TargetEnum<U>`.
/// enumify_wrap => this forces wrapping a value, e.g. `U`
/// becomes `TargetEnum<U>`. Enabling this allows nested `TargetEnum`, e.g.
/// `TargetEnum<V>` can become `TargetEnum<TargetEnum<V>>`
pub use enumify_macro::enumify_struct;

/// In order for the generated structure to be resolved to the base structure
/// the enum we wrap our struct in must implement this ResolveToBase trait.
/// This is because the generated structure will contain fields of the enum
/// type, and this trait tells us how to map the enum variants to their
/// underlying base type (in the `build` function of the Applicable trait)
pub trait ResolveToBase<T> {
    fn resolve_to_base(&self) -> T;
}

/// The trait is implemented for every generated structure. Thanks to this, you
/// can use enumify_struct in generic contexts.
pub trait Applicable: Sized {
    /// This is the type the enumify_struct macro was used on. We need the type
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
