
use bytes::Bytes;
use std::marker::PhantomData;
use std::sync::Arc;
use protocols::protocol::cloudstate::{
    entity_discovery_server::EntityDiscoveryServer,
    eventsourced::event_sourced_server::EventSourcedServer,
};
use tonic::transport::Server;
use protocols::prost_example::{
    shoppingcart::{self, AddLineItem, RemoveLineItem, GetShoppingCart,
    persistence::{Cart, ItemAdded, ItemRemoved, LineItem},},
};
use prost::Message;
use cloudstate_prost_derive::CommandDecoder;
use cloudstate_core::CommandDecoder;
use cloudstate_core::eventsourced::{EntityRegistry, EventSourcedEntity, HandleCommandContext};
use server_spike::{EventSourcedServerImpl, EntityDiscoveryServerImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:8088".parse().unwrap();
    let addr = "0.0.0.0:8088".parse().unwrap();

    let mut registry = EntityRegistry(vec![]);
    registry.add_entity("com.example.shoppingcart.ShoppingCart", ShoppingCartEntity::default);
    // registry.add_entity("shopcart2", ShoppingCartEntity::default);
    // registry.add_entity_type("shopcart3", PhantomData::<ShoppingCartEntity>);

    let server = EventSourcedServerImpl(Arc::new(registry));

    let discovery_server = EntityDiscoveryServerImpl {
        descriptor_set: protocols::example::shopping_cart_descriptor_set().to_vec(),
    };
    let discovery = EntityDiscoveryServer::new(discovery_server);
    let eventsourced = EventSourcedServer::new(server);

    Server::builder()
        .add_service(discovery)
        .add_service(eventsourced)
        .serve(addr).await?;

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

//TODO generate encoding trait
pub enum ShoppingCartReply {
    Cart(shoppingcart::Cart),
}

// Events
// #[package="com.example.shoppingcart.persistence"]
pub enum ShoppingCartEvent {
    ItemAdded(ItemAdded),
    ItemRemoved(ItemRemoved),
}

// Snapshot
#[derive(CommandDecoder)]
#[package="com.example.shoppingcart.persistence"]
pub enum ShoppingCartSnapshot {
    Snapshot(Cart),
}

#[derive(Default)]
//TODO use more convenient type for internal state, e.g. HashMap
pub struct ShoppingCartEntity(Cart);

impl EventSourcedEntity for ShoppingCartEntity {

    type Command = ShoppingCartCommand;
    type Response = ShoppingCartReply;

    type Snapshot = ShoppingCartSnapshot;
    type Event = ShoppingCartEvent;

    fn restore(&mut self, snapshot: Self::Snapshot) {
        let ShoppingCartSnapshot::Snapshot(cart) = snapshot;
        self.0 = cart;
        println!("Snapshot Loaded: {:?}", self.0);
    }

    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) -> Option<Self::Response> {
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
                None
            }
            ShoppingCartCommand::RemoveLine(item) => {
                println!("Handle command: {:?}", item);
                None
            }
            ShoppingCartCommand::GetCart(cart) => {
                println!("Handle command: {:?}", cart);

                Some(
                    ShoppingCartReply::Cart(
                        // convert from domain::cart to shoppingcart::cart
                        shoppingcart::Cart {
                            items: self.0.items.iter()
                                .map(|li| shoppingcart::LineItem {
                                    product_id: li.product_id.clone(),
                                    name: li.name.clone(),
                                    quantity: li.quantity,
                                }).collect()
                        }
                    )
                )

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

