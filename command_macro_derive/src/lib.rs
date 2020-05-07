
use proc_macro::TokenStream;
use quote::quote;
use syn;

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

    let variants = match &ast.data {
        syn::Data::Enum(data_enum) => {
            //TODO check that each variant contains only one unnamed field with a type that implements ::prost::Message trait
            data_enum.variants.iter().map(|v| &v.ident).collect()
        },
        _ => vec![], //TODO return an error that only enums are supported
    };

    let unknown_command = quote! {
        unknown_command_type => {
            eprintln!("Unknown command type: {}", unknown_command_type);
            None
        },
    };

    let items: Vec<_> = variants.iter().map(|v| {
        //TODO Add type package. Need to pass the package name somehow, maybe as an enum attribute?
        //TODO Use internal struct name as a type name for matching instead of the enum variant name!
        let variant_name = v.to_string();
        quote!(
            #variant_name => {
                //TODO use variant param type instead of v here
                match <#v as Message>::decode(bytes) {
                    Ok(cmd) => {
                        println!("Received {:?}", cmd);
                        Some(#type_name::#v(cmd))
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
