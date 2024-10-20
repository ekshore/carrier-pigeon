use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Type, Variant};

#[proc_macro_derive(DisplayEnum)]
pub fn display_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let options: Vec<proc_macro2::TokenStream> = if let syn::Data::Enum(data) = input.data {
        data.variants
            .into_iter()
            .map(|v| v.ident)
            .map(|variant| quote! { #name::#variant => stringify!(#variant), })
            .collect()
    } else {
        panic!("DisplayEnum only supports Enums");
    };

    TokenStream::from(quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        #(#options)*
                    }
                )
            }
        }
    })
}

#[proc_macro_derive(ListEnum)]
pub fn list_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let entries: Vec<proc_macro2::TokenStream> = if let syn::Data::Enum(data) = input.data {
        data.variants
            .into_iter()
            .map(|v| v.ident)
            .map(|val| quote! { #name::#val, })
            .collect()
    } else {
        panic!("ListEnum only works for Enums.");
    };

    TokenStream::from(quote! {
        impl #name {
            pub fn to_vec() -> Vec<Self> {
                vec![ #(#entries)* ]
            }
        }
    })
}

#[proc_macro_derive(OrderedEnum)]
pub fn ordered_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (to_usize, to_varient, last_varient) = if let syn::Data::Enum(data) = input.data {
        let last_varient = if let Some(variant) = &data.variants.last() {
            quote! { _ => #name::#variant }
        } else {
            quote! { _ => panic!("There are no variants!"); }
        };

        let (to_usize, to_varient) = data.variants.clone().into_iter()
            .map(|v| v.ident)
            .enumerate().fold(
            (vec![], vec![]),
            |(mut to_usize, mut to_varient), (idx, varient)| {
                to_usize.push(quote! { #name::#varient => #idx, });
                to_varient.push(quote! { #idx => #name::#varient, });
                (to_usize, to_varient)
            },
        );
        (to_usize, to_varient, last_varient)
    } else {
        panic!("OrderedEnum only supports enums.");
    };

    let expanded = quote! {
       impl From<#name> for usize {
            fn from(val: #name) -> Self {
                match val {
                    #(#to_usize)*
                }
            }
        }

        /// This implementation of From essentially index's into the enum.
        /// Returning the Enum variant corresponding to the index of the usize being converted.
        /// When the index is out of bounds the last variant of the Enum is returned.
        impl From<usize> for #name {
            fn from(val: usize) -> Self {
                match val {
                    #(#to_varient)*
                    #last_varient
                }
            }
        }
    };

    TokenStream::from(expanded)
}

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
                        *ident == *"wrapper"
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

    let expanded = quote! {
        #(#from_implimentations)*
    };
    TokenStream::from(expanded)
}
