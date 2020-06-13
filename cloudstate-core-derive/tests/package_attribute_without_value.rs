use cloudstate_core_derive::AnyMessage;

mod shopping_cart;
use shopping_cart::*;

#[package]
#[derive(AnyMessage)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

fn main() {}