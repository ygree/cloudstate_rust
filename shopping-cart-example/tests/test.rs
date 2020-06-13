
use protocols::protocol::cloudstate::{Command, eventsourced::{
    EventSourcedInit, EventSourcedStreamIn, EventSourcedStreamOut, EventSourcedSnapshot,
    event_sourced_client::EventSourcedClient,
    event_sourced_stream_in,
    event_sourced_stream_out,
    EventSourcedReply
}, ClientAction, Reply};
use protocols::prost_example::shoppingcart::{
    AddLineItem,
    persistence::*,
};
use futures_util::stream;
use protocols::protocol::cloudstate::client_action::Action;
use tonic::Streaming;
use prost_types::Any;
use bytes::Bytes;
use tokio::runtime::Runtime;
use tonic::transport::Channel;
use shopcart_example::run; // Add methods on commands

#[test]
fn test() {

    let mut rt = Runtime::new().unwrap();

    // Running the server for tests within the same process to make sure it's stopped
    // when a test assertion fails
    rt.spawn(run("0.0.0.0:8088"));

    let mut client = rt.block_on(EventSourcedClient::connect("http://127.0.0.1:8088"))
        .expect("Cannot start client");

    //TODO implement multiple scenarios
    rt.block_on(simple_test(&mut client)).expect("test failed");
}

async fn simple_test(client: &mut EventSourcedClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {

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

    let stream_in = msgs_to_stream_in(vec![
        Message::Init(init_msg),
        Message::Command(cmd_msg.clone())]
    );
    let response = client.handle(stream_in).await?;

    let mut inbound = response.into_inner();

    {
        let reply1 = inbound.expect_reply().await.expect("Expected Reply");
        assert_eq!(reply1.command_id, cmd_msg.id);

        let reply_body = reply1.reply_payload().expect("Expected Action Reply");
        reply_body.decode::<Any>().expect("Expected empty reply");

        assert_eq!(reply1.events.len(), 1);
        let item = reply1.events[0].clone().decode::<ItemAdded>()
            .expect("Expect ItemAdded event")
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