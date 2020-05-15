use ::prost::Message;
use bytes::Bytes;
use server_spike::CommandDecoder; //TODO should probably move this to a separate trait out of server_spike
use command_macro_derive::CommandDecoder;

mod shopping_cart;
use shopping_cart::*;

// #[package = "com.example.shoppingcart"]
#[derive(CommandDecoder)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

fn main() {}