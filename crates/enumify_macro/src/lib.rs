use quote::quote;

mod enum_struct;
#[cfg(test)]
mod test;

#[proc_macro_attribute]
pub fn enumify_struct(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let out = enum_struct::enumify_struct(attr.into(), input.into());
    let original = out.original;
    let generated = out.generated;
    proc_macro::TokenStream::from(quote! {
        #original

        #generated
    })
}
