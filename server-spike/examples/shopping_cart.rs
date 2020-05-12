
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

// Commands
#[package="com.example.shoppingcart"]
#[derive(CommandDecoder)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
}

// Events
// #[package="com.example.shoppingcart.persistence"]
pub enum ShoppingCartEvent {
    ItemAdded(ItemAdded),
    ItemRemoved(ItemRemoved),
}

#[derive(Default)]
//TODO use more convenient type for internal state, e.g. HashMap
pub struct ShoppingCartEntity(Cart);

impl EventSourcedEntity for ShoppingCartEntity {

    type Snapshot = Cart;
    type Command = ShoppingCartCommand;
    type Event = ShoppingCartEvent;

    fn restore(&mut self, snapshot: Self::Snapshot) {
        self.0 = snapshot;
        println!("Snapshot Loaded: {:?}", self.0);
    }

    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) {
        match command {
            ShoppingCartCommand::AddLine(item) => {
                println!("Handle command: {:?}", item);
                context.emit_event(
                    //TODO looks like too much boilerplate
                    ShoppingCartEvent::ItemAdded(
                        ItemAdded { //TODO maybe implement auto-conversion for: ItemAdded -> ShoppingCartEvent::ItemAdded
                            item: Some(
                                LineItem {
                                    product_id: item.product_id,
                                    name: item.name,
                                    quantity: item.quantity,
                                }
                            )
                        }
                    )
                );
            }
            ShoppingCartCommand::RemoveLine(item) => {
                println!("Handle command: {:?}", item);
            }
            ShoppingCartCommand::GetCart(cart) => {
                println!("Handle command: {:?}", cart);
            }
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

