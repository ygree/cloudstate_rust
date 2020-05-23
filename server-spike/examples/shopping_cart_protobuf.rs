
use bytes::Bytes;
use std::marker::PhantomData;
use std::sync::Arc;
use protocols::protocol::cloudstate::{
    entity_discovery_server::EntityDiscoveryServer,
    eventsourced::event_sourced_server::EventSourcedServer,
};
use tonic::transport::Server;
use protocols::example::{
    shoppingcart::{AddLineItem, RemoveLineItem, GetShoppingCart},
    domain::{Cart, ItemAdded, ItemRemoved, LineItem},
};
use server_spike::{EventSourcedEntity, CommandDecoder, HandleCommandContext, EntityRegistry, EntityDiscoveryServerImpl, EventSourcedServerImpl};
use cloudstate_protobuf_derive::CommandDecoder;
use protobuf::SingularPtrField;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:8088".parse().unwrap();
    let addr = "0.0.0.0:8088".parse().unwrap();

    let mut registry = EntityRegistry(vec![]);
    registry.add_entity("shopcart", ShoppingCartEntity::default);
    registry.add_entity("shopcart2", ShoppingCartEntity::default);
    registry.add_entity_type("shopcart3", PhantomData::<ShoppingCartEntity>);

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

    type Snapshot = ShoppingCartSnapshot;
    type Command = ShoppingCartCommand;
    type Event = ShoppingCartEvent;

    fn restore(&mut self, snapshot: Self::Snapshot) {
        let ShoppingCartSnapshot::Snapshot(cart) = snapshot;
        self.0 = cart;
        println!("Snapshot Loaded: {:?}", self.0);
    }

    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) {
        match command {
            ShoppingCartCommand::AddLine(item) => {
                println!("Handle command: {:?}", item);
                context.emit_event(
                    //TODO looks like too much boilerplate
                    ShoppingCartEvent::ItemAdded(
                        ItemAdded {
                            item: SingularPtrField::some(
                                LineItem {
                                    productId: item.product_id,
                                    name: item.name,
                                    quantity: item.quantity,
                                    ..Default::default()
                                }
                            ),
                            ..Default::default()
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
                if let Some(item) = item_added.item.into_option() {
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

