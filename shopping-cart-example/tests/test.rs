
use bytes::Bytes;
use futures_util::stream;
use protocols::protocol::cloudstate::{
    Command, client_action::Action,
    eventsourced::{
        EventSourcedInit, EventSourcedStreamIn, EventSourcedStreamOut, EventSourcedSnapshot,
        event_sourced_client::EventSourcedClient,
        event_sourced_stream_in,
        event_sourced_stream_out,
        EventSourcedReply
    },
    entity_discovery_client::EntityDiscoveryClient, ProxyInfo
};
use protocols::prost_example::shoppingcart::{
    AddLineItem,
    persistence::*,
};
use prost_types::Any;
use tonic::{Streaming, IntoStreamingRequest, Request};
use tonic::transport::Channel;
use tokio::runtime::Runtime;
use shopcart_example::run;
use protocols::protocol::cloudstate::entity_discovery_server::EntityDiscovery;

#[test]
fn test() {
    let mut rt = Runtime::new().unwrap();

    // Running the server for tests within the same process to make sure it's stopped
    // when a test assertion fails
    rt.spawn(run("0.0.0.0:8088"));

    let mut entity_discovery_client = rt.block_on(EntityDiscoveryClient::connect("http://127.0.0.1:8088"))
        .expect("Cannot start entity discovery client");

    let mut event_sourced_client = rt.block_on(EventSourcedClient::connect("http://127.0.0.1:8088"))
        .expect("Cannot start event sourced client");

    //TODO implement more scenarios
    rt.block_on(discovery_test(&mut entity_discovery_client)).expect("test failed");
    rt.block_on(event_sourced_test(&mut event_sourced_client)).expect("test failed");
}

async fn discovery_test(client: &mut EntityDiscoveryClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {
    // verify that the user function process responds

    let proxy_info = ProxyInfo {
        protocol_major_version: 0,
        protocol_minor_version: 1,
        proxy_name: "test".to_owned(),
        proxy_version: "0.1".to_owned(),
        supported_entity_types: vec!["cloudstate.eventsourced.EventSourced".to_owned()]
    };

    let entity_spec = client.discover(proxy_info).await.expect("Expected response");
    let message = entity_spec.get_ref();

    assert!(!message.proto.is_empty());
    let entity = message.entities.first().expect("Expected one entity");
    assert_eq!(entity.entity_type, "cloudstate.eventsourced.EventSourced");
    assert_eq!(entity.service_name, "com.example.shoppingcart.ShoppingCart");
    assert_eq!(entity.persistence_id, "shopping_cart");

    Ok(())
}

async fn event_sourced_test(client: &mut EventSourcedClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {

    let item1 = LineItem {
        product_id: "soap33".to_string(),
        name: "soap".to_string(),
        quantity: 12,
    };

    let cart = Cart {
        items: vec![item1],
    };

    let snapshot = EventSourcedSnapshot {
        snapshot_sequence: 42,
        snapshot: Some(cart.to_any("type.googleapis.com/com.example.shoppingcart.persistence.Cart")),
    };

    use event_sourced_stream_in::Message;

    let init_msg = EventSourcedInit{
        service_name: "com.example.shoppingcart.ShoppingCart".to_string(),
        entity_id: "shopcart_entity_id".to_string(),
        snapshot: Some(snapshot),
    };

    let add_line_item = AddLineItem {
        user_id: "user_id".to_owned(),
        product_id: "product_id".to_owned(),
        name: "Product Name".to_owned(),
        quantity: 1,
    };

    let cmd_msg = Command {
        entity_id: "shopcart_entity_id".to_string(),
        id: 56,
        name: "command_name".to_string(),
        payload: Some(add_line_item.clone().to_any("type.googleapis.com/com.example.shoppingcart.AddLineItem")),
        streamed: false,
    };

    let requests = msgs_to_stream_in(vec![
        Message::Init(init_msg),
        Message::Command(cmd_msg.clone())]
    );
    let response = client.handle(requests).await?;

    let mut inbound = response.into_inner();

    {
        let reply1 = inbound.expect_reply().await.expect("Expected Reply");
        assert_eq!(reply1.command_id, cmd_msg.id);

        let reply_body = reply1.reply_payload().expect("Expected Action Reply");
        reply_body.decode::<Any>().expect("Expected empty reply");

        assert_eq!(reply1.events.len(), 1);
        let item = reply1.events[0].clone().decode::<ItemAdded>().expect("Expect ItemAdded event")
            .item.expect("Expect LineItem");
        assert_eq!(item.product_id, add_line_item.product_id);
        assert_eq!(item.name, add_line_item.name);
        assert_eq!(item.quantity, add_line_item.quantity);
    }

    assert_eq!(inbound.message().await?, None);

    Ok(())
}

#[tonic::async_trait]
trait StreamingEventSourcedStreamOutExt {
    async fn expect_reply(&mut self) -> Option<EventSourcedReply>;
}

#[tonic::async_trait]
impl StreamingEventSourcedStreamOutExt for Streaming<EventSourcedStreamOut> {
    async fn expect_reply(&mut self) -> Option<EventSourcedReply> {
        match self.message().await {
            Ok(Some(EventSourcedStreamOut {
                        message: Some(event_sourced_stream_out::Message::Reply(reply))
                    })) => Some(reply),
            _ => None,
        }
    }
}

trait EventSourcedReplyExt {
    fn reply_payload(&self) -> Option<Any>;
}

impl EventSourcedReplyExt for EventSourcedReply {
    fn reply_payload(&self) -> Option<Any> {
        self.client_action.iter()
            .flat_map(|v| &v.action)
            .flat_map(|v|
                match v {
                    Action::Reply(r) => r.payload.clone(),
                    _ => None,
                })
            .last()
    }
}

trait AnyExt {
    fn decode<T>(self) -> Option<T>
        where T: prost::Message + Default;
}

impl AnyExt for Any {
    fn decode<T>(self) -> Option<T> where T: prost::Message + Default {
        let bytes = Bytes::from(self.value);
        <T as prost::Message>::decode(bytes).ok()
    }
}

trait ProstMessageExt {
    fn to_any(&self, type_url: &str) -> Any;
}

impl<T> ProstMessageExt for T
    where T: ::prost::Message {

    fn to_any(&self, type_url: &str) -> Any {
        let mut buf = vec![];
        self.encode(&mut buf); //TODO returns Result
        ::prost_types::Any {
            type_url: type_url.to_owned(),
            value: buf,
        }
    }
}

fn msgs_to_stream_in<T>(msgs: T) -> impl tonic::IntoStreamingRequest<Message = EventSourcedStreamIn>
    where T: IntoIterator<Item = event_sourced_stream_in::Message>
{
    let messages: Vec<_> = msgs.into_iter()
        .map(|msg| EventSourcedStreamIn { message: Some(msg.clone()) })
        .collect();
    stream::iter(messages)
}
