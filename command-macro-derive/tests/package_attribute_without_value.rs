use command_macro_derive::CommandDecoder;

mod shopping_cart;
use shopping_cart::*;

#[package]
#[derive(CommandDecoder)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

fn main() {}