
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
        quantity: 12i32,
    };

    let cart = Cart {
        items: vec![item1],
    };

    let snapshot = EventSourcedSnapshot {
        snapshot_sequence: 42i64,
        snapshot: Some(create_any("type.googleapis.com/com.example.shoppingcart.persistence.Cart".to_string(), cart)),
    };

    use event_sourced_stream_in::Message;

    let init_msg = Message::Init(EventSourcedInit{
        service_name: "com.example.shoppingcart.ShoppingCart".to_string(),
        entity_id: "shopcart_entity_id".to_string(),
        snapshot: Some(snapshot),
    });

    let add_line_item = AddLineItem {
        user_id: "user_id".to_owned(),
        product_id: "product_id".to_owned(),
        name: "Product Name".to_owned(),
        quantity: 1,
    };

    let cmd_msg = Message::Command(Command {
        entity_id: "shopcart_entity_id".to_string(),
        id: 56,
        name: "command_name".to_string(),
        payload: Some(create_any("type.googleapis.com/com.example.shoppingcart.AddLineItem".to_owned(), add_line_item.clone())),
        streamed: false,
    });

    let stream_in = msgs_to_stream_in(vec![init_msg, cmd_msg]);
    let response = client.handle(stream_in).await?;

    let mut inbound = response.into_inner();

    {
        let reply1 = expect_reply(&mut inbound).await.expect("Expected Reply");
        assert_eq!(reply1.command_id, 56);

        let reply_body = extract_action_reply_payload(&reply1).expect("Expected Action Reply");

        let reply_msg: () = decode_any(reply_body).expect("Expected empty reply");
        assert_eq!(reply_msg, ());

        assert_eq!(reply1.events.len(), 1);
        let item = decode_any::<ItemAdded>(reply1.events[0].clone())
            .expect("Expect ItemAdded event")
            .item.expect("Expect LineItem");
        assert_eq!(item.product_id, add_line_item.product_id);
        assert_eq!(item.name, add_line_item.name);
        assert_eq!(item.quantity, add_line_item.quantity);
    }

    assert_eq!(inbound.message().await?, None);


    Ok(())
}

fn create_any(type_url: String, msg: impl ::prost::Message) -> ::prost_types::Any {
    let mut buf = vec![];
    msg.encode(&mut buf); //TODO returns Result
    ::prost_types::Any {
        type_url,
        value: buf,
    }
}

fn decode_any<T>(any: prost_types::Any) -> Option<T>
    where T: prost::Message + Default
{
    let bytes = Bytes::from(any.value);
    <T as prost::Message>::decode(bytes).ok()
}

fn extract_action_reply_payload(reply: &EventSourcedReply) -> Option<Any> {
    reply.client_action.iter()
        .flat_map(|v| &v.action)
        .flat_map(|v|
            match v {
                Action::Reply(r) => r.payload.clone(),
                _ => None,
            })
        .last()
}

async fn expect_reply(inbound: &mut Streaming<EventSourcedStreamOut>) -> Option<EventSourcedReply> {
    match inbound.message().await {
        Ok(Some(EventSourcedStreamOut {
                    message: Some(event_sourced_stream_out::Message::Reply(reply))
                })) => Some(reply),
        _ => None,
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