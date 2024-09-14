use std::collections::HashSet;

use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Data, DeriveInput, Field, Fields, Ident, Path, Token, Type,
    Visibility,
};

const RENAME_ATTRIBUTE: &str = "enumify_rename";
const SKIP_WRAP_ATTRIBUTE: &str = "enumify_skip_wrap";
const WRAP_ATTRIBUTE: &str = "enumify_wrap";
const CFG_ATTRIBUTE: &str = "cfg";

struct FieldOptions {
    wrapping_behavior: bool,
    cfg_attribute: Option<Attribute>,
    new_type: Option<TokenTree>,
    field_ident: TokenStream,
}

trait EnumFieldVisitor {
    fn visit(
        &mut self,
        global_options: &GlobalOptions,
        old_field: &mut Field,
        new_field: &mut Field,
        field_options: &FieldOptions,
    );
}

struct GenerateApplicableImplVisitor {
    acc_concrete: TokenStream,
}

impl GenerateApplicableImplVisitor {
    fn new() -> Self {
        GenerateApplicableImplVisitor {
            acc_concrete: quote! {},
        }
    }

    fn get_implementation(
        self,
        orig: &DeriveInput,
        new: &DeriveInput,
    ) -> TokenStream {
        let (impl_generics, ty_generics, _) = orig.generics.split_for_impl();
        let orig_name = &orig.ident;
        let new_name = &new.ident;
        let acc_concrete = self.acc_concrete;

        quote! {
            impl #impl_generics enumify_struct::Applicable for #new_name #ty_generics {
                type Base = #orig_name #ty_generics;

                fn apply_to(self, t: &mut Self::Base) {
                    #acc_concrete
                }
            }
        }
    }

    fn get_incremental_setter_concrete(
        ident: &TokenStream,
        is_wrapped: bool,
        is_nested: bool,
        is_base_enum: bool,
    ) -> TokenStream {
        match (is_base_enum, is_wrapped, is_nested) {
            (true, false, true) => quote! {
                if let Some(existing) = &mut t.#ident {
                    self.#ident.apply_to(existing);
                } else {
                    t.#ident = self.#ident.try_into().ok();
                }
            },
            (true, false, false) => quote! {
                t.#ident = self.#ident;
            },
            (false, false, true) => {
                quote! { self.#ident.apply_to(&mut t.#ident); }
            }
            (false, false, false) => quote! { t.#ident = self.#ident; },
            (true, true, true) => {
                quote! { if let (Some(inner), Some(target)) = (self.#ident, &mut t.#ident) { inner.apply_to(target); } }
            }
            (false, true, true) => {
                quote! {
                    let inner = self.#ident.resolve_to_base();
                    inner.apply_to(&mut t.#ident);
                }
            }
            (_, true, false) => {
                quote! { t.#ident = self.#ident.resolve_to_base(); }
            }
        }
    }
}

impl EnumFieldVisitor for GenerateApplicableImplVisitor {
    fn visit(
        &mut self,
        global_options: &GlobalOptions,
        old_field: &mut Field,
        _new_field: &mut Field,
        field_options: &FieldOptions,
    ) {
        let ident = &field_options.field_ident;
        let cfg_attr = &field_options.cfg_attribute;

        let is_wrapped = field_options.wrapping_behavior;
        let is_nested = field_options.new_type.is_some();
        let is_base_enum =
            is_type_target_enum(&old_field.ty, &global_options.target_enum);

        let inc_concrete = Self::get_incremental_setter_concrete(
            ident,
            is_wrapped,
            is_nested,
            is_base_enum,
        );

        let acc_concrete = &self.acc_concrete;
        self.acc_concrete = quote! {
            #acc_concrete

            #cfg_attr
            #inc_concrete
        };
    }
}

struct SetNewFieldVisibilityVisitor;

impl EnumFieldVisitor for SetNewFieldVisibilityVisitor {
    fn visit(
        &mut self,
        global_options: &GlobalOptions,
        _old_field: &mut Field,
        new_field: &mut Field,
        _field_options: &FieldOptions,
    ) {
        if global_options.make_fields_public {
            new_field.vis =
                Visibility::Public(syn::token::Pub(new_field.vis.span()))
        }
    }
}

struct SetNewFieldTypeVisitor;

impl EnumFieldVisitor for SetNewFieldTypeVisitor {
    fn visit(
        &mut self,
        global_options: &GlobalOptions,
        old_field: &mut Field,
        new_field: &mut Field,
        field_options: &FieldOptions,
    ) {
        let mut new_type = if let Some(t) = &field_options.new_type {
            quote! {#t}
        } else {
            let t = &old_field.ty;
            quote! {#t}
        };
        let target_enum = &global_options.target_enum;
        if field_options.wrapping_behavior {
            new_type = quote! {#target_enum<#new_type>};
        };
        new_field.ty = Type::Verbatim(new_type);
    }
}

// https://github.com/rust-lang/rust/issues/65823 :(
struct RemoveHelperAttributesVisitor;

impl EnumFieldVisitor for RemoveHelperAttributesVisitor {
    fn visit(
        &mut self,
        _global_options: &GlobalOptions,
        old_field: &mut Field,
        new_field: &mut Field,
        _field_options: &FieldOptions,
    ) {
        let indexes_to_remove = old_field
            .attrs
            .iter()
            .enumerate()
            .filter_map(|(i, a)| {
                if a.path().is_ident(RENAME_ATTRIBUTE)
                    || a.path().is_ident(SKIP_WRAP_ATTRIBUTE)
                    || a.path().is_ident(WRAP_ATTRIBUTE)
                {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Don't forget to reverse so the indices are removed without being
        // shifted!
        for i in indexes_to_remove.into_iter().rev() {
            old_field.attrs.swap_remove(i);
            new_field.attrs.swap_remove(i);
        }
    }
}

fn borrow_fields(
    derive_input: &mut DeriveInput,
) -> &mut Punctuated<Field, Comma> {
    let data_struct = match &mut derive_input.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("enumify_struct only works for structs"),
    };

    match &mut data_struct.fields {
        Fields::Unnamed(f) => &mut f.unnamed,
        Fields::Named(f) => &mut f.named,
        Fields::Unit => {
            unreachable!("A struct cannot have simply a unit field?")
        }
    }
}

fn visit_fields(
    visitors: &mut [&mut dyn EnumFieldVisitor],
    global_options: &GlobalOptions,
    derive_input: &DeriveInput,
) -> (DeriveInput, DeriveInput) {
    let mut new = derive_input.clone();
    let mut orig = derive_input.clone();
    let old_fields = borrow_fields(&mut orig);
    let new_fields = borrow_fields(&mut new);

    for (struct_index, (old_field, new_field)) in
        old_fields.iter_mut().zip(new_fields.iter_mut()).enumerate()
    {
        let mut overriden_wrapping = false;
        let mut wrapping_behavior =
            !is_type_target_enum(&old_field.ty, &global_options.target_enum)
                && global_options.default_wrapping_behavior;
        let mut cfg_attribute = None;
        let mut new_type = None;
        old_field.attrs
            .iter()
            .for_each(|a| {
                if a.path().is_ident(RENAME_ATTRIBUTE) {
                    let args = a
                        .parse_args()
                        .unwrap_or_else(|_| panic!("'{RENAME_ATTRIBUTE}' attribute expects one and only one argument (the new type to use)"));
                    new_type = Some(args);
                    if !overriden_wrapping {
                        wrapping_behavior = false;
                    }
                } else if a.path().is_ident(SKIP_WRAP_ATTRIBUTE) {
                    wrapping_behavior = false;
                    overriden_wrapping = true;
                } else if a.path().is_ident(WRAP_ATTRIBUTE) {
                    wrapping_behavior = true;
                    overriden_wrapping = true;
                } else if a.path().is_ident(CFG_ATTRIBUTE) {
                    cfg_attribute = Some(a.clone());
                }
            });
        let field_ident = if let Some(ident) = &old_field.ident {
            quote! {#ident}
        } else {
            let i = syn::Index::from(struct_index);
            quote! {#i}
        };
        let field_options = FieldOptions {
            wrapping_behavior,
            cfg_attribute,
            new_type,
            field_ident,
        };
        for v in &mut *visitors {
            v.visit(global_options, old_field, new_field, &field_options);
        }
    }
    (orig, new)
}

fn get_derive_macros(
    new: &DeriveInput,
    extra_derive: &[String],
) -> TokenStream {
    let mut extra_derive = extra_derive.iter().collect::<HashSet<_>>();
    for attributes in &new.attrs {
        let _ = attributes.parse_nested_meta(|derived_trait| {
            let derived_trait = derived_trait.path;
            let full_path = quote! { #derived_trait };
            extra_derive.remove(&full_path.to_string());
            Ok(())
        });
    }

    let mut acc = quote! {};
    for left_trait_to_derive in extra_derive {
        let left_trait_to_derive = format_ident!("{left_trait_to_derive}");
        acc = quote! { # left_trait_to_derive, # acc};
    }

    quote! { #[derive(#acc)] }
}

struct ParsedMacroParameters {
    target_enum: Option<Ident>,
    new_struct_name: Option<String>,
    default_wrapping: bool,
}

impl Parse for ParsedMacroParameters {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = ParsedMacroParameters {
            target_enum: None,
            new_struct_name: None,
            default_wrapping: true,
        };

        if let Ok(target_enum) = Ident::parse(input) {
            out.target_enum = Some(target_enum);
        } else {
            panic!("{}", input.error("Expected target_enum").to_string());
        }

        if input.parse::<Token![,]>().is_err() {
            return Ok(out);
        };

        if let Ok(struct_name) = Ident::parse(input) {
            out.new_struct_name = Some(struct_name.to_string());
        } else {
            return Ok(out);
        };

        if input.parse::<Token![,]>().is_err() {
            return Ok(out);
        };

        if let Ok(wrapping) = syn::LitBool::parse(input) {
            out.default_wrapping = wrapping.value;
        } else {
            return Ok(out);
        };

        Ok(out)
    }
}

fn is_path_enum(p: &Path, target_enum: &Ident) -> bool {
    p.segments
        .last()
        .map(|ps| ps.ident == *target_enum)
        .unwrap_or(false)
}

fn is_type_target_enum(t: &Type, target_enum: &Ident) -> bool {
    macro_rules! wtf {
        ($reason : tt) => {
            panic!(
                "Using enumify_struct for a struct containing a {} is not valid.",
                $reason
            )
        };
    }

    match &t {
        // real work
        Type::Path(type_path) => is_path_enum(&type_path.path, target_enum),
        Type::Array(_) | Type::Tuple(_) => false,
        Type::Paren(type_paren) => {
            is_type_target_enum(&type_paren.elem, target_enum)
        }

        // No clue what to do with those
        Type::ImplTrait(_) | Type::TraitObject(_) => {
            panic!(
                "Might already be the target_enum, but there is no way to tell"
            )
        }
        Type::Infer(_) => panic!("If you cannot tell, neither can I"),
        Type::Macro(_) => panic!("Don't think I can handle this easily..."),

        // Makes no sense to use those in an EnumifyStruct
        Type::Reference(_) => wtf!("reference"),
        Type::Never(_) => wtf!("never-type"),
        Type::Slice(_) => wtf!("slice"),
        Type::Ptr(_) => wtf!("pointer"),
        Type::BareFn(_) => wtf!("function pointer"),

        _ => panic!("Open an issue please"),
    }
}

struct GlobalOptions {
    new_struct_name: String,
    target_enum: Ident,
    extra_derive: Vec<String>,
    default_wrapping_behavior: bool,
    make_fields_public: bool,
}

impl GlobalOptions {
    fn new(
        attr: ParsedMacroParameters,
        struct_definition: &DeriveInput,
    ) -> Self {
        let new_struct_name = attr.new_struct_name.unwrap_or_else(|| {
            "Enumified".to_owned() + &struct_definition.ident.to_string()
        });
        let default_wrapping_behavior = attr.default_wrapping;
        let target_enum = attr.target_enum.unwrap();
        GlobalOptions {
            new_struct_name,
            target_enum,
            extra_derive: vec!["Clone", "PartialEq", "Debug"]
                .into_iter()
                .map(|s| s.to_owned())
                .collect(),
            default_wrapping_behavior,
            make_fields_public: true,
        }
    }
}

pub struct EnumifyStructOutput {
    pub original: TokenStream,
    pub generated: TokenStream,
}

pub fn enumify_struct(
    attr: TokenStream,
    input: TokenStream,
) -> EnumifyStructOutput {
    let derive_input = syn::parse2::<DeriveInput>(input).unwrap();
    let macro_params =
        GlobalOptions::new(syn::parse2::<_>(attr).unwrap(), &derive_input);

    let mut applicable_impl_generator = GenerateApplicableImplVisitor::new();

    let mut visitors = [
        &mut RemoveHelperAttributesVisitor as &mut dyn EnumFieldVisitor,
        &mut SetNewFieldVisibilityVisitor,
        &mut SetNewFieldTypeVisitor,
        &mut applicable_impl_generator,
    ];

    let (orig, mut new) =
        visit_fields(&mut visitors, &macro_params, &derive_input);

    new.ident = Ident::new(&macro_params.new_struct_name, new.ident.span());

    let applicable_impl =
        applicable_impl_generator.get_implementation(&derive_input, &new);

    let derives = get_derive_macros(&new, &macro_params.extra_derive);

    let generated = quote! {
        #derives
        #new

        #applicable_impl
    };

    EnumifyStructOutput {
        original: quote! { #orig },
        generated,
    }
}
