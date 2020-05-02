
use ::command_macro_derive::CommandDecoder;
use protocols::example::shoppingcart::{AddLineItem, RemoveLineItem, GetShoppingCart, persistence};
use protocols::example::shoppingcart::persistence::{Cart, ItemAdded, ItemRemoved};
use super::{EventSourcedEntity, CommandDecoder};
use ::prost::Message;
use bytes::Bytes;
use crate::HandleCommandContext;

#[derive(Default)]
pub struct ShoppingCartEntity(Cart);
//TODO use more convenient type for internal state, e.g. HashMap

//TODO consider using Attribute-like macros, e.g. #[commands(AddLineItem, RemoveLineItem, GetShoppingCart)]
// Combine command into one type.
#[derive(CommandDecoder)]
pub enum ShoppingCartCommand {
    AddLineItem(AddLineItem),
    RemoveLineItem(RemoveLineItem),
    GetShoppingCart(GetShoppingCart),
}

pub enum ShoppingCartEvent {
    ItemAdded(ItemAdded),
    ItemRemoved(ItemRemoved),
}

// impl ShoppingCartEvent {
//     fn from_message(msg: impl ::prost::Message) -> Option<ShoppingCartEvent> {
//         match msg {
//             cmd @ ItemAdded {..} => Some(cmd),
//             cmd @ ItemRemoved {..} => Some(cmd),
//             _ => None,
//         }
//     }
// }

impl EventSourcedEntity for ShoppingCartEntity {

    type Snapshot = Cart;
    type Command = ShoppingCartCommand;
    type Event = ShoppingCartEvent;

    fn snapshot_loaded(&mut self, snapshot: Self::Snapshot) {
        self.0 = snapshot;
        println!("Snapshot Loaded: {:?}", self.0);
    }

    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) {
        match command {
            ShoppingCartCommand::AddLineItem(add_line_item) => {
                println!("Handle command: {:?}", add_line_item);
                context.emit_event(
                    //TODO looks like too much boilerplate
                    ShoppingCartEvent::ItemAdded(
                        ItemAdded {
                            item: Some(
                                persistence::LineItem {
                                    product_id: add_line_item.product_id,
                                    name: add_line_item.name,
                                    quantity: add_line_item.quantity,
                                }
                            )
                        }
                    )
                );
            },
            ShoppingCartCommand::RemoveLineItem(remove_line_item) => {
                println!("Handle command: {:?}", remove_line_item);
            },
            ShoppingCartCommand::GetShoppingCart(get_shopping_cart) => {
                println!("Handle command: {:?}", get_shopping_cart);
            },
        }
    }

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            ShoppingCartEvent::ItemAdded(item_added) => {
                println!("Handle event: {:?}", item_added);
                if let Some(item) = item_added.item {
                    self.0.items.push(item);
                }
            },
            ShoppingCartEvent::ItemRemoved(item_removed) => {
                println!("Handle event: {:?}", item_removed);
                //TODO remove item
            },
        }
    }

}

