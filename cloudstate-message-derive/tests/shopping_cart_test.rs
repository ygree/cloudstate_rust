use ::prost::Message;
use bytes::Bytes;
use cloudstate_core::CommandDecoder;
use cloudstate_message_derive::CommandDecoder;

mod shopping_cart;
use shopping_cart::*;

//TODO Fix these tests. They've been built for Prost but now the macro generates code for protobuf.

#[package = "com.example.shoppingcart"]
#[derive(CommandDecoder, Debug, PartialEq)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

fn test_command_decoder() {
    let msg = add_line_item();

    let bytes = encode(&msg);

    let result = <ShoppingCartCommand as CommandDecoder>::decode("type.googleapis.com/com.example.shoppingcart.AddLineItem".to_owned(), bytes);

    assert_eq!(result, Some(ShoppingCartCommand::AddLine(msg)));
}

fn test_command_decoder_with_incorrect_type() {
    let msg = add_line_item();

    let bytes = encode(&msg);

    let result = <ShoppingCartCommand as CommandDecoder>::decode("AddLineItem".to_owned(), bytes);

    assert_eq!(result, None);
}

fn main() {
    test_command_decoder();
    test_command_decoder_with_incorrect_type();
}