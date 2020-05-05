
use ::prost::Message;
use bytes::Bytes;
use std::marker::PhantomData;
use std::sync::Arc;
use protocols::protocol::cloudstate::eventsourced::event_sourced_server::EventSourcedServer;
use tonic::transport::Server;
use protocols::example::shoppingcart::{
    AddLineItem, RemoveLineItem, GetShoppingCart,
    persistence::{Cart, ItemAdded, ItemRemoved, LineItem}
};
use server_spike::{EventSourcedEntity, CommandDecoder, HandleCommandContext, EntityRegistry, EventSourcedServerImpl};
use command_macro_derive::CommandDecoder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:9000".parse().unwrap();

    let mut registry = EntityRegistry(vec![]);
    registry.add_entity("shopcart", ShoppingCartEntity::default);
    registry.add_entity("shopcart2", ShoppingCartEntity::default);
    registry.add_entity_type("shopcart3", PhantomData::<ShoppingCartEntity>);

    let server = EventSourcedServerImpl(Arc::new(registry));

    let svc = EventSourcedServer::new(server);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

//TODO find out how type is encoded to make sure that it's derivable from command types
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

#[derive(Default)]
pub struct ShoppingCartEntity(Cart);
//TODO use more convenient type for internal state, e.g. HashMap

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
                        ItemAdded { //TODO maybe implement auto-conversion for: ItemAdded -> ShoppingCartEvent::ItemAdded
                            item: Some(
                                LineItem {
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

