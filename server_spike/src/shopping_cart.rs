
use ::command_macro_derive::CommandDecoder;
use protocols::example::shoppingcart::{ AddLineItem, RemoveLineItem, GetShoppingCart };
use protocols::example::shoppingcart::persistence::*;
use super::{EventSourcedEntity, CommandDecoder};
use ::prost::Message;
use bytes::Bytes;

pub struct ShoppingCartEntity(Cart);

impl Default for ShoppingCartEntity {
    fn default() -> Self {
        Self(
            Cart {
                items: vec![],
            }
        )
    }
}

//TODO consider using Attribute-like macros, e.g. #[commands(AddLineItem, RemoveLineItem, GetShoppingCart)]
// Combine command into one type.
#[derive(CommandDecoder)]
pub enum ShoppingCartCommand {
    AddLineItem(AddLineItem),
    RemoveLineItem(RemoveLineItem),
    GetShoppingCart(GetShoppingCart),
}

impl EventSourcedEntity for ShoppingCartEntity {

    type Snapshot = Cart;
    type Command = ShoppingCartCommand;

    fn snapshot_loaded(&mut self, snapshot: Self::Snapshot) {
        self.0 = snapshot;
        println!("Snapshot Loaded: {:?}", self.0);
    }

    fn handle_command(&self, command: Self::Command) {
        match command {
            ShoppingCartCommand::AddLineItem(add_line_item) => {
                println!("Handle command: {:?}", add_line_item);
            },
            ShoppingCartCommand::RemoveLineItem(remove_line_item) => {
                println!("Handle command: {:?}", remove_line_item);
            },
            ShoppingCartCommand::GetShoppingCart(get_shopping_cart) => {
                println!("Handle command: {:?}", get_shopping_cart);
            },
        }

    }
}
