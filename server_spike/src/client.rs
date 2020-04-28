
use protocols::protocol::cloudstate::eventsourced::{
    EventSourcedInit, EventSourcedStreamIn, EventSourcedSnapshot,
    event_sourced_stream_in,
    event_sourced_client::{EventSourcedClient}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EventSourcedClient::connect("http://[::1]:9000").await?;

    use protocols::example::shoppingcart::persistence::*;

    let item1 = LineItem {
        product_id: "soap33".to_string(),
        name: "soap".to_string(),
        quantity: 12i32,
    };
    let cart = Cart {
        items: vec![item1],
    };
    use ::prost::Message; // import Message trait to call encode on Cart
    use bytes::BufMut;
    let mut buf = vec![];
    cart.encode(&mut buf);

    let snapshot_any = ::prost_types::Any {
        //TODO is it even a correct type?
        type_url: "com.example/shoppingcart.persistence.Cart".to_string(),
        value: buf,
    };

    let snapshot = EventSourcedSnapshot {
        snapshot_sequence: 42i64,
        snapshot: Some(snapshot_any),
    };

    // use event_sourced_stream_in::Message; //TODO doesn't shadow previous import of ::prost::Message
    let msg = event_sourced_stream_in::Message::Init(EventSourcedInit{
        service_name: "shopcart".to_string(),
        entity_id: "shopcart".to_string(),
        snapshot: Some(snapshot),
    });

    let outbound = async_stream::stream! {
        yield EventSourcedStreamIn{
            message: Some(msg),
        };
    };

    let response = client.handle(outbound).await?;

    let mut inbound = response.into_inner();

    while let Some(note) = inbound.message().await? {
        println!("Response = {:?}", note);
    }

    Ok(())
}
