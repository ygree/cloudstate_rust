
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
use cloudstate_core::eventsourced::{EntityRegistry, EventSourcedEntity, HandleCommandContext, Response};
use server_spike::{EventSourcedServerImpl, EntityDiscoveryServerImpl};
use std::collections::BTreeMap;

pub async fn run(host_port: &str) -> Result<(), tonic::transport::Error> {
    // let addr = "[::1]:8088".parse().unwrap();
    let addr = host_port.parse().unwrap();

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
    // GetCart2(GetShoppingCart, impl Ctx<shoppingcart::Cart>), //TODO what if we encode response type in the command?
    // GetCart2(GetShoppingCart, &mut shoppingcart::Cart), //TODO or this way
}

#[derive(CommandDecoder)]
#[package="com.example.shoppingcart"]
pub enum ShoppingCartReply {
    Cart(shoppingcart::Cart),
}

// Events
#[derive(CommandDecoder)]
#[package="com.example.shoppingcart.persistence"]
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
pub struct ShoppingCartEntity(BTreeMap<ItemId, ItemValue>);

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

    // fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) -> Result<Option<Self::Response>, String> {
    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) -> Result<Response<Self::Response>, String> {
        match command {
            ShoppingCartCommand::AddLine(item) => {
                println!("Handle command: {:?}", item);
                if item.quantity <= 0 {
                    return Err(format!("Cannot add negative quantity of to item {}", item.product_id))
                }
                context.emit_event(
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
                Ok(Response::EmptyReply)
            }
            ShoppingCartCommand::RemoveLine(item) => {
                println!("Handle command: {:?}", item);
                if !self.0.contains_key(&item.product_id) {
                    return Err(format!("Cannot remove item {} because it is not in the cart.", item.product_id))
                }
                context.emit_event(
                    ShoppingCartEvent::ItemRemoved(
                        ItemRemoved { //TODO maybe implement auto-conversion for: ItemAdded -> ShoppingCartEvent::ItemAdded
                            product_id: item.product_id,
                        }
                    )
                );
                Ok(Response::EmptyReply)
            }
            ShoppingCartCommand::GetCart(cart) => {
                println!("Handle command: {:?}", cart);
                Ok(Response::Reply(
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
                ))
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
                self.0.remove(&item_removed.product_id);
            },
        }
    }

}

