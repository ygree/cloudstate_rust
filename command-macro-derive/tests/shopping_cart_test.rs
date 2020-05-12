use ::prost::Message;
use bytes::Bytes;
use server_spike::CommandDecoder; //TODO should probably move this to a separate trait out of server_spike
use command_macro_derive::CommandDecoder;

mod shopping_cart;
use shopping_cart::*;

#[package = "com.example.shoppingcart"]
#[derive(CommandDecoder, Debug, PartialEq)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

#[test]
fn test_command_decoder() {
    let msg = add_line_item();

    let bytes = encode(&msg);

    let result = <ShoppingCartCommand as CommandDecoder>::decode("type.googleapis.com/com.example.shoppingcart.AddLineItem".to_owned(), bytes);

    assert_eq!(result, Some(ShoppingCartCommand::AddLine(msg)));
}

#[test]
fn test_command_decoder_with_incorrect_type() {
    let msg = add_line_item();

    let bytes = encode(&msg);

    let result = <ShoppingCartCommand as CommandDecoder>::decode("AddLineItem".to_owned(), bytes);

    assert_eq!(result, None);
}
