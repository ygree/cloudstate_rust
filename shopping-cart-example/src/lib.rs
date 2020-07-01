
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
use cloudstate_core::AnyMessage;
use cloudstate_core_derive::AnyMessage;
use cloudstate_core::eventsourced::{EntityRegistry, EventSourcedEntity, CommandContext, Response};
use cloudstate_server::{EventSourcedServerImpl, EntityDiscoveryServerImpl};
use std::collections::BTreeMap;

pub async fn run_server(host_port: String) -> Result<(), tonic::transport::Error> {
    let addr = host_port.parse().unwrap();

    let mut registry = EntityRegistry::new();
    registry.register_event_sourced_entity("com.example.shoppingcart.ShoppingCart", "shopping-cart", ShoppingCartEntity::default);
    registry.register_event_sourced_entity("test-snapshot-every-time", "shopping-cart", || ShoppingCartEntity::new(1));
    // registry.add_entity("shopcart2", ShoppingCartEntity::default);
    // registry.add_entity_type("shopcart3", PhantomData::<ShoppingCartEntity>);

    let entity_registry = Arc::new(registry);
    let server = EventSourcedServerImpl(entity_registry.clone());

    let discovery_server = EntityDiscoveryServerImpl {
        descriptor_set: protocols::example::shopping_cart_descriptor_set().to_vec(),
        entity_registry,
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
#[derive(AnyMessage)]
pub enum ShoppingCartCommand {
    AddLine(AddLineItem),
    RemoveLine(RemoveLineItem),
    GetCart(GetShoppingCart),
    // GetCart2(GetShoppingCart, impl Ctx<shoppingcart::Cart>), //TODO what if we encode response type in the command?
    // GetCart2(GetShoppingCart, &mut shoppingcart::Cart), //TODO or this way
}

#[derive(AnyMessage)]
#[package="com.example.shoppingcart"]
pub enum ShoppingCartReply {
    Cart(shoppingcart::Cart),
}

// Events
#[derive(AnyMessage)]
#[package="com.example.shoppingcart.persistence"]
pub enum ShoppingCartEvent {
    ItemAdded(ItemAdded),
    ItemRemoved(ItemRemoved),
}

// Snapshot
#[derive(AnyMessage)]
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
pub struct ShoppingCartEntity {
    items: BTreeMap<ItemId, ItemValue>,
    snapshot_every: Option<u32>,
}

impl ShoppingCartEntity {

    fn new(snapshot_every: u32) -> ShoppingCartEntity {
        ShoppingCartEntity {
            items: BTreeMap::new(),
            snapshot_every: Some(snapshot_every),
        }
    }
}

impl EventSourcedEntity for ShoppingCartEntity {

    type Command = ShoppingCartCommand;
    type Response = ShoppingCartReply;

    type Snapshot = ShoppingCartSnapshot;
    type Event = ShoppingCartEvent;

    fn snapshot_every(&self) -> Option<u32> {
        self.snapshot_every
    }

    fn handle_snapshot(&mut self, snapshot: Self::Snapshot) {
        let ShoppingCartSnapshot::Snapshot(cart) = snapshot;

        println!("Loading snapshot: {:?}", &cart);

        self.items.clear();

        for LineItem { product_id, name,  quantity } in cart.items {
            self.items.insert(product_id, ItemValue { name, qty: quantity });
        }
    }

    fn handle_command(&self, command: Self::Command, context: &mut impl CommandContext<Self::Event>) -> Result<Response<Self::Response>, String> {
        match command {
            ShoppingCartCommand::AddLine(item) => self.add_line(context, item).map(|_| Response::EmptyReply),
            ShoppingCartCommand::RemoveLine(item) => self.remove_line(context, item).map(|_| Response::EmptyReply),
            ShoppingCartCommand::GetCart(cart) => Ok(Response::Reply(self.get_cart(cart))),
        }
    }

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            ShoppingCartEvent::ItemAdded(item_added) => {
                println!("Handle event: {:?}", item_added);
                if let Some(LineItem { product_id, name, quantity }) = item_added.item {
                    let mut item_val = self.items.entry(product_id)
                        .or_insert(ItemValue { name, qty: 0 });
                    item_val.qty += quantity;
                }
            },
            ShoppingCartEvent::ItemRemoved(item_removed) => {
                println!("Handle event: {:?}", item_removed);
                self.items.remove(&item_removed.product_id);
            },
        }
    }

}


impl ShoppingCartEntity {

    fn add_line(&self, context: &mut impl CommandContext<ShoppingCartEvent>, item: AddLineItem) -> Result<(), String> {
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
        Ok(())
    }

    fn remove_line(&self, context: &mut impl CommandContext<ShoppingCartEvent>, item: RemoveLineItem) -> Result<(), String> {
        println!("Handle command: {:?}", item);
        if !self.items.contains_key(&item.product_id) {
            return Err(format!("Cannot remove item {} because it is not in the cart.", item.product_id))
        }
        context.emit_event(
            ShoppingCartEvent::ItemRemoved(
                ItemRemoved { //TODO maybe implement auto-conversion for: ItemAdded -> ShoppingCartEvent::ItemAdded
                    product_id: item.product_id,
                }
            )
        );
        Ok(())
    }

    fn get_cart(&self, cart: GetShoppingCart) -> ShoppingCartReply {
        println!("Handle command: {:?}", cart);
        ShoppingCartReply::Cart(
            shoppingcart::Cart {
                items: self.items.iter()
                    .map(|(item_id, item_val)| shoppingcart::LineItem {
                        product_id: item_id.clone(),
                        name: item_val.name.clone(),
                        quantity: item_val.qty,
                    }).collect()
            }
        )
    }
}
