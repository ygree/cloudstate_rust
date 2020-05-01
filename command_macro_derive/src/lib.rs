extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
// use std::string::String;

#[proc_macro_derive(CommandDecoder)]
pub fn command_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_command_macro(&ast)
}

fn impl_command_macro(ast: &syn::DeriveInput) -> TokenStream {

    let name = &ast.ident;

    if let syn::Data::Enum(data_enum) = &ast.data {
        for v in &data_enum.variants {
            println!("!--> {}", &v.ident);
            //TODO how to build a match out of the list of variants?
        }
    }

    let unknown_command = quote! {
        unknown_command_type => {
            eprintln!("Unknown command type: {}", unknown_command_type);
            None
        },
    };

    let item = quote! {
        "AddLineItem" => {
            match <AddLineItem as Message>::decode(bytes) {
                Ok(command) => {
                    println!("Received {:?}", command);
                    Some(ShoppingCartCommand::AddLineItem(command))
                },
                Err(err) => {
                    eprintln!("Error decoding AddLineItem command: {}", err);
                    None
                },
            }
        },
    };

    let gen = quote! {
        impl CommandDecoder for #name {
            fn decode(type_url: String, bytes: Bytes) -> Option<Self> {
                match type_url.as_ref() {
                    //TODO how to generate it out of enum variants?
                    #item
                    #unknown_command
                }
            }
        }
    };
    gen.into()
}
