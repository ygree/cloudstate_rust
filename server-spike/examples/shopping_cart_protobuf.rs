
use bytes::Bytes;
use std::sync::Arc;
use protocols::protocol::cloudstate::{
    entity_discovery_server::EntityDiscoveryServer,
    eventsourced::event_sourced_server::EventSourcedServer,
};
use tonic::transport::Server;
use protocols::example::{
    shoppingcart::{self, AddLineItem, RemoveLineItem, GetShoppingCart},
    domain::{Cart, ItemAdded, ItemRemoved, LineItem},
};
use cloudstate_protobuf_derive::CommandDecoder;
use protobuf::{SingularPtrField, RepeatedField};
use cloudstate_core::CommandDecoder;
use cloudstate_core::eventsourced::{EntityRegistry, EventSourcedEntity, HandleCommandContext, Response};
use server_spike::{EventSourcedServerImpl, EntityDiscoveryServerImpl};
use std::collections::BTreeMap;


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

#[derive(Debug)]
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

        for LineItem { productId, name,  quantity, .. } in cart.items.into_vec() {
            self.0.insert(productId, ItemValue { name, qty: quantity });
        }
    }

    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) -> Result<Response<Self::Response>, String> {
        match command {
            ShoppingCartCommand::AddLine(item) => {
                println!("Handle command: {:?}", item);
                if item.quantity <= 0 {
                    return Err(format!("Cannot add negative quantity of to item {}", item.product_id))
                }
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
                Ok(Response::EmptyReply)
            }
            ShoppingCartCommand::RemoveLine(item) => {
                println!("Handle command: {:?}", item);
                if !self.0.contains_key(&item.product_id) {
                    return Err(format!("Cannot remove item {} because it is not in the cart.", &item.product_id))
                }
                context.emit_event(
                    ShoppingCartEvent::ItemRemoved(
                        ItemRemoved {
                            productId: item.product_id,
                            ..Default::default()
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
                            items: RepeatedField::from_vec(self.0.iter()
                                .map(|(item_id, item_val)| shoppingcart::LineItem {
                                    product_id: item_id.clone(),
                                    name: item_val.name.clone(),
                                    quantity: item_val.qty,
                                    ..Default::default()
                                }).collect()),
                            ..Default::default()
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
                if let Some(item) = item_added.item.into_option() {
                    {
                        let mut item_val = self.0.entry(item.productId.clone())
                            .or_insert(ItemValue { name: item.name.clone(), qty: 0 });
                        item_val.qty += item.quantity;
                    }
                    println!("----> ItemAdded : {:?} : {:?}", &item, self.0.entry(item.productId.clone()));
                }
            },
            ShoppingCartEvent::ItemRemoved(item_removed) => {
                println!("Handle event: {:?}", item_removed);
                self.0.remove(&item_removed.productId);
                println!("----> ItemRemoved : {:?} : {:?}", &item_removed, self.0.entry(item_removed.productId.clone()));
            },
        }
    }

}

