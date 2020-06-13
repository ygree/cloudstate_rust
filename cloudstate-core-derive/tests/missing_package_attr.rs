use ::prost::Message;
use bytes::Bytes;
use cloudstate_core::AnyMessage;
use cloudstate_core_derive::AnyMessage;

mod shopping_cart;
use shopping_cart::*;

// #[package = "com.example.shoppingcart"]
#[derive(AnyMessage)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

fn main() {}