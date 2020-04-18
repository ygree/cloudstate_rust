
use protocols::cloudstate::eventsourced::event_sourced_client::{EventSourcedClient};
use protocols::cloudstate::eventsourced::{EventSourcedInit, EventSourcedStreamIn};
use protocols::cloudstate::eventsourced::{event_sourced_stream_in};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EventSourcedClient::connect("http://[::1]:9000").await?;

    let outbound = async_stream::stream! {
        use event_sourced_stream_in::Message;
        let msg = Message::Init(EventSourcedInit{
            service_name: "shopcart".to_string(),
            entity_id: "shopcart".to_string(),
            snapshot: None, //TODO pass snapshot serialize to Any
        });
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
