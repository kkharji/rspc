mod attr;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use attr::*;

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input);

    let DeriveInput {
        ident, data, attrs, ..
    } = &derive_input;

    let container_attrs = ContainerAttr::from_attrs(attrs).unwrap();
    let crate_name = format_ident!("{}", container_attrs.crate_name.unwrap_or("specta".into()));

    let body = match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) => {
                let fields = data.fields.iter().filter_map(|field| {
                    let attrs = FieldAttr::from_attrs(&field.attrs).unwrap();

                    if attrs.skip {
                        return None;
                    }

                    let ident = &field.ident;

                    Some(quote! {
                        #crate_name::ObjectField {
                            name: stringify!(#ident).to_string(),
                            ty: self.#ident.to_data_type(),
                            optional: false,
                            flatten: false
                        }
                    })
                });

                quote! {
                    #crate_name::DataType::Object(#crate_name::ObjectType {
                        name: stringify!(#ident).to_string(),
                        generics: vec![],
                        fields: vec![#(#fields),*],
                        tag: None,
                        type_id: None
                    })
                }
            }
            Fields::Unnamed(_) => {
                let fields = data.fields.iter().enumerate().map(|(i, _)| {
                    let i = proc_macro2::Literal::usize_unsuffixed(i);
                    quote!(self.#i.to_data_type())
                });

                quote! {
                    #crate_name::DataType::Tuple(#crate_name::TupleType {
                        name: stringify!(#ident).to_string(),
                        generics: vec![],
                        fields: vec![#(#fields),*]
                    })
                }
            }
            _ => todo!("ToDataType only supports named structs"),
        },
        _ => todo!("ToDataType only supports named structs"),
    };

    quote! {
        impl #crate_name::ToDataType for #ident {
            fn to_data_type(self) -> #crate_name::DataType {
                #body
            }
        }
    }
    .into()
}
