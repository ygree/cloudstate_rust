
use bytes::Bytes;
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
use std::collections::HashMap;

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

#[derive(CommandDecoder)]
#[package="com.example.shoppingcart"]
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

struct ItemValue {
    name: String,
    qty: i32,
}

type ItemId = String;

#[derive(Default)]
pub struct ShoppingCartEntity(HashMap<ItemId, ItemValue>);

impl EventSourcedEntity for ShoppingCartEntity {

    type Command = ShoppingCartCommand;
    type Response = ShoppingCartReply;

    type Snapshot = ShoppingCartSnapshot;
    type Event = ShoppingCartEvent;

    fn restore(&mut self, snapshot: Self::Snapshot) {
        let ShoppingCartSnapshot::Snapshot(cart) = snapshot;

        println!("Loading snapshot: {:?}", &cart);

        self.0.clear();

        for LineItem { product_id, name,  quantity } in cart.items {
            self.0.insert(product_id, ItemValue { name, qty: quantity });
        }
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
                        shoppingcart::Cart {
                            items: self.0.iter()
                                .map(|(item_id, item_val)| shoppingcart::LineItem {
                                    product_id: item_id.clone(),
                                    name: item_val.name.clone(),
                                    quantity: item_val.qty,
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
                if let Some(LineItem { product_id, name, quantity }) = item_added.item {
                    let mut item_val = self.0.entry(product_id)
                        .or_insert(ItemValue { name, qty: 0 });
                    item_val.qty += quantity;
                }
            },
            ShoppingCartEvent::ItemRemoved(item_removed) => {
                println!("Handle event: {:?}", item_removed);
                //TODO remove item
            },
        }
    }

}

