use cloudstate_message_derive::CommandDecoder;

mod shopping_cart;
use shopping_cart::*;

#[package("com.example.shoppingcart")]
#[derive(CommandDecoder)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

fn main() {}