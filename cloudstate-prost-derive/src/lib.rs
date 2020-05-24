
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, Fields, FieldsUnnamed, Field, Type, Result, Token, LitStr};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

#[proc_macro_derive(CommandDecoder, attributes(package))]
pub fn cloudstate_prost_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_command_macro(&ast)
}

struct ProtobufPacket(String);

impl Parse for ProtobufPacket {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Token![=] = input.parse()?;
        let package_name: LitStr = input.parse()?;
        Ok(ProtobufPacket(package_name.value()))
    }
}

fn impl_command_macro(ast: &syn::DeriveInput) -> TokenStream {
    let type_name = &ast.ident;

    let attrs = &ast.attrs;

    let package_attr_opt = attrs.iter().find(|a| {
        a.path.segments.iter().find(|p| {
            p.ident.to_string() == "package"
        }).is_some()
    });

    let protobuf_packet =
        if let Some(package_attr) = package_attr_opt {
            let tks = proc_macro::TokenStream::from(package_attr.tokens.clone());
            // TODO pass package_attr.span() to point to in the error message
            //  instead of parse_macro_input!(tks as ProtobufPacket) that will point to the derive macro
            match parse_macro_input::parse::<ProtobufPacket>(tks)
                .map_err(|e| syn::Error::new(package_attr.span(), e.to_string())) {
                Ok(data) => data,
                Err(err) => {
                    return TokenStream::from(err.to_compile_error());
                }
            }
        } else {
            panic!("Not found package attribute!")
        };

    let variants: Vec<(_, _)> = match &ast.data {
        syn::Data::Enum(data_enum) => {
            //TODO check that each variant contains only one unnamed field with a type that implements ::prost::Message trait
            data_enum.variants.iter().map(|v| {
                let field_path = match v.fields {
                    Fields::Unnamed(FieldsUnnamed{ ref unnamed, .. }) => {
                        let fs: Vec<&Field> = unnamed.iter().collect();
                        if fs.len() == 1 {
                            match &fs[0].ty {
                                Type::Path(type_path) => {
                                    let result = &type_path.path.segments;
                                    if result.is_empty() {
                                        panic!("An empty path type provided: {}!", v.ident)
                                    }
                                    result
                                },
                                _ => {
                                    panic!("2 Only single non-generic struct parameter is allowed for enum variant {}!", v.ident) //TODO properly handle it
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
                (&v.ident, field_path)
            }).collect()
        },
        _ => vec![], //TODO return an error that only enums are supported
    };

    //TODO split parsing from code-generation

    let unknown_command = quote! {
        unknown_command_type => {
            eprintln!("Unknown command type: {}", unknown_command_type);
            None
        },
    };

    let items: Vec<_> = variants.into_iter().map(|(enum_id, field_path)| {
        let variant_name = enum_id.to_string();
        let field_id = &field_path.last().unwrap().ident;
        let full_type = format!("type.googleapis.com/{}.{}", protobuf_packet.0, &field_id.to_string());
        quote!(
            #full_type => {
                match <#field_path as Message>::decode(bytes) {
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

#[test]
fn foo() {
    let type_name = "type.googleapis.com/com.example.shoppingcart.AddLineItem".to_owned();

    let match_with = ".".to_string() + "AddLineItem";

    let result = match type_name {
        s if s.ends_with(&match_with) => true,
        _ => false,
    };

    assert!(result);
}

#[test]
fn bar() {
    let type_name = "yes".to_string();

    let result = match type_name.as_ref() {
        "yes" => true,
        _ => false,
    };

    assert!(result);
}

