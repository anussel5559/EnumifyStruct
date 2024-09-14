use quote::quote;

use crate::enum_struct::enumify_struct;

#[test]
fn basic_gen() {
    enumify_struct(
        quote!(BasicEnum),
        quote!(
            struct Foo {
                bar: u8,
                baz: String,
            }
        ),
    );
}

#[test]
fn with_nested() {
    enumify_struct(
        quote!(BasicEnum),
        quote!(
            struct Foo {
                #[enumify_rename(OptionalBar)]
                bar: Bar,
            }
        ),
    );
}

#[test]
fn with_redundant_derive() {
    enumify_struct(
        quote!(BasicEnum),
        quote!(
            #[derive(Debug, Clone)]
            struct Foo {
                bar: u8,
                baz: String,
            }
        ),
    );
}

#[test]
fn with_unrelated_derive() {
    enumify_struct(
        quote!(BasicEnum),
        quote!(
            #[derive(Display)]
            struct Foo {
                bar: u8,
                baz: String,
            }
        ),
    );
}

#[test]
fn with_enum() {
    enumify_struct(
        quote!(BasicEnum),
        quote!(
            struct Foo {
                bar: Option<u8>,
            }
        ),
    );
}

#[test]
fn with_cfg_attributes() {
    enumify_struct(
        quote!(BasicEnum),
        quote!(
            struct Foo {
                #[cfg(all())]
                bar: u8,
                #[cfg(any())]
                baz: u8,
            }
        ),
    );
}

#[test]
fn with_nested_option_skip_wrap() {
    enumify_struct(
        quote!(BasicEnum),
        quote!(
            struct Foo {
                #[enumify_skip_wrap]
                #[enumify_rename(EnumInner)]
                inner1: Option<Inner>,

                #[enumify_rename(EnumInner)]
                #[enumify_wrap]
                inner2: Option<Inner>,

                #[enumify_wrap]
                inner3: Option<Inner>,

                #[enumify_skip_wrap]
                inner4: Option<Inner>,

                #[enumify_wrap]
                #[enumify_rename(EnumInner)]
                inner5: Option<Inner>,
            }
        ),
    );
}
