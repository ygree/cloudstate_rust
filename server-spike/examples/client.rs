
use protocols::protocol::cloudstate::{
    Command,
    eventsourced::{
        EventSourcedInit, EventSourcedStreamIn, EventSourcedSnapshot,
        event_sourced_stream_in,
        event_sourced_client::{EventSourcedClient}
    },
};
use protocols::prost_example::shoppingcart::{
    AddLineItem,
    persistence::*,
};
use futures_util::stream;

fn create_any(type_url: String, msg: impl ::prost::Message) -> ::prost_types::Any {
    let mut buf = vec![];
    msg.encode(&mut buf); //TODO returns Result
    ::prost_types::Any {
        type_url,
        value: buf,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut client = EventSourcedClient::connect("http://[::1]:8088").await?;
    let mut client = EventSourcedClient::connect("http://127.0.0.1:8088").await?;

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
        ..Default::default()
    };

    let cmd_msg = Message::Command(Command {
        entity_id: "shopcart_entity_id".to_string(),
        id: 56i64,
        name: "command_name".to_string(),
        payload: Some(create_any("type.googleapis.com/com.example.shoppingcart.AddLineItem".to_owned(), add_line_item)),
        streamed: false,
    });

    // let outbound = async_stream::stream! {
    //     yield EventSourcedStreamIn {
    //         message: Some(init_msg),
    //     };
    //     yield EventSourcedStreamIn {
    //         message: Some(cmd_msg),
    //     };
    // };
    // let response = client.handle(outbound).await?;

    // let messages: Vec<_> = vec![init_msg, cmd_msg].iter()
    //     .map(|msg| EventSourcedStreamIn { message: Some(msg.clone()) })
    //     .collect();
    // let response = client.handle(stream::iter(messages)).await?;

    let stream_in = msgs_to_stream_in(vec![init_msg, cmd_msg]);
    let response = client.handle(stream_in).await?;

    let mut inbound = response.into_inner();

    while let Some(note) = inbound.message().await? {
        println!("Response = {:?}", note);
    }

    Ok(())
}

fn msgs_to_stream_in<T>(msgs: T) -> impl tonic::IntoStreamingRequest<Message = EventSourcedStreamIn>
    where T: IntoIterator<Item = event_sourced_stream_in::Message>
{
    let messages: Vec<_> = msgs.into_iter()
        .map(|msg| EventSourcedStreamIn { message: Some(msg.clone()) })
        .collect();
    stream::iter(messages)
}

