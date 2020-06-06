
use protocols::protocol::cloudstate::{Command, eventsourced::{
    EventSourcedInit, EventSourcedStreamIn, EventSourcedSnapshot,
    event_sourced_stream_in,
    event_sourced_client::{EventSourcedClient},
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
use protocols::protocol::cloudstate::eventsourced::EventSourcedStreamOut;

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
    };

    let cmd_msg = Message::Command(Command {
        entity_id: "shopcart_entity_id".to_string(),
        id: 56i64,
        name: "command_name".to_string(),
        payload: Some(create_any("type.googleapis.com/com.example.shoppingcart.AddLineItem".to_owned(), add_line_item.clone())),
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

    let reply1 = expect_reply(&mut inbound).await.expect("Expected Reply");
    assert_eq!(reply1.command_id, 56);

    let expected =
        EventSourcedReply {
            command_id: 56i64,
            client_action: Some(
                ClientAction {
                    action: Some(
                        Action::Reply(
                            Reply {
                                payload: Some(create_any("type.googleapis.com/google.protobuf.Empty".to_owned(), ()))
                            }
                        )
                    )
                }
            ),
            side_effects: vec![],
            events: vec![ //TODO would be more informative if events are deserialized for the assertion
                create_any("type.googleapis.com/com.example.shoppingcart.persistence.ItemAdded".to_owned(),
                    ItemAdded {
                        item: Some(
                            LineItem {
                                product_id: add_line_item.product_id.clone(),
                                name: add_line_item.name.clone(),
                                quantity: add_line_item.quantity,
                            }
                        )
                    }
                )
            ],
            snapshot: None,
        };

    assert_eq!(reply1, expected);

    while let Some(note) = inbound.message().await? {
        println!("Response = {:?}", note);
    }

    Ok(())
}

async fn expect_reply(inbound: &mut Streaming<EventSourcedStreamOut>) -> Option<EventSourcedReply> {
    match inbound.message().await {
        Ok(
            Some(
                EventSourcedStreamOut {
                    message: Some(event_sourced_stream_out::Message::Reply(reply))
                }
            )
        ) => Some(reply),
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

