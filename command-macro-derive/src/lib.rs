
use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::{Fields, FieldsUnnamed, Field, Type};

#[proc_macro_derive(CommandDecoder)]
pub fn command_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_command_macro(&ast)
}

fn impl_command_macro(ast: &syn::DeriveInput) -> TokenStream {

    let type_name = &ast.ident;

    let variants: Vec<(_, _)> = match &ast.data {
        syn::Data::Enum(data_enum) => {
            //TODO check that each variant contains only one unnamed field with a type that implements ::prost::Message trait
            data_enum.variants.iter().map(|v| {

                let field_ident = match v.fields {
                    Fields::Unnamed(FieldsUnnamed{ ref unnamed, .. }) => {
                        let fs: Vec<&Field> = unnamed.iter().collect();
                        if fs.len() == 1 {
                            match &fs[0].ty {
                                Type::Path(type_path) => {
                                    if let Some(ident) = type_path.path.get_ident() {
                                        ident
                                    }
                                    else {
                                        panic!("Boom!") //TODO properly handle it
                                    }
                                },
                                _ => {
                                    panic!("Boom!") //TODO properly handle it
                                },
                            }
                            //==> type = Path(TypePath { qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "GetShoppingCart", span: #0 bytes(1423..1438) }, arguments: None }] } })
                        }
                        else {
                            panic!("Exactly one unnamed paramater supported only!") //TODO properly handle it
                        }
                    },
                    _ => {
                        panic!("Only unnamed fields are supported!") //TODO properly handle it
                    }
                };


                (&v.ident, field_ident)
            }).collect()
        },
        _ => vec![], //TODO return an error that only enums are supported
    };

    let unknown_command = quote! {
        unknown_command_type => {
            eprintln!("Unknown command type: {}", unknown_command_type);
            None
        },
    };

    let items: Vec<_> = variants.iter().map(|(enum_id, field_id)| {
        //TODO Add type package. Need to pass the package name somehow, maybe as an enum attribute?
        //TODO Use internal struct name as a type name for matching instead of the enum variant name!
        let variant_name = enum_id.to_string();
        let field_type_name = field_id.to_string();
        quote!(
            #field_type_name => {
                //TODO use variant param type instead of v here
                match <#field_id as Message>::decode(bytes) {
                    Ok(cmd) => {
                        println!("Received {:?}", cmd);
                        Some(#type_name::#enum_id(cmd))
                    },
                    Err(err) => {
                        eprintln!("Error decoding {} command: {}", #variant_name, err);
                        None
                    },
                }
            },
        )
    }).collect();

    let gen = quote! {
        impl CommandDecoder for #type_name {
            fn decode(type_url: String, bytes: Bytes) -> Option<Self> {
                match type_url.as_ref() {
                    #(#items)*
                    #unknown_command
                }
            }
        }
    };

    gen.into()
}
