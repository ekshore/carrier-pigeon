use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Type, Variant};

#[proc_macro_derive(FromWrappedError, attributes(wrapper))]
pub fn from_wrapped_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let from_implimentations = if let syn::Data::Enum(data) = input.data {
        let wrapper_variants: Vec<&Variant> = data
            .variants
            .iter()
            .filter(|v| {
                let attr = v.attrs.iter().find(|attr| {
                    if let Some(ident) = attr.path().get_ident() {
                        ident.to_string() == String::from("wrapper")
                    } else {
                        false
                    }
                });
                attr.is_some()
            })
            .collect();

        let from_impls: Vec<proc_macro2::TokenStream> = wrapper_variants
            .iter()
            .map(|variant| {
                let error_name = &variant.ident;
                let wrapped_error = match &variant.fields {
                    Fields::Unnamed(fields) => {
                        let inner = fields.unnamed.first().unwrap();
                        &inner.ty
                    }
                    Fields::Named(_fields) => panic!("Named Fields are not supported wrappers"),
                    Fields::Unit => panic!("Unit Error types cannot be wrappers"),
                };
                let wrapped_error = match wrapped_error {
                    Type::Path(error_type) => &error_type.path,
                    _ => panic!("Wrapped error must be of Type::Path"),
                };

                quote! {
                    impl From<#wrapped_error> for #name {
                        fn from(val: #wrapped_error) -> Self {
                            #name::#error_name(val)
                        }
                    }
                }
            })
            .collect();

        from_impls
    } else {
        panic!("Only enums error types are supported at this time");
    };

    let expanded = quote!{
        #(#from_implimentations)*
    };
    TokenStream::from(expanded)
}
