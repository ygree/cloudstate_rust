
use protocols::cloudstate::eventsourced::event_sourced_server::{EventSourced, EventSourcedServer};
use protocols::cloudstate::eventsourced::{EventSourcedStreamIn, EventSourcedStreamOut};
use protocols::cloudstate::eventsourced::event_sourced_stream_out;
use tonic::{Status, Streaming, Response, Request};
use tonic::transport::Server;
use std::pin::Pin;
use futures_core::Stream;

#[derive(Default)]
struct EventSourcedServerImpl;

#[tonic::async_trait]
impl EventSourced for EventSourcedServerImpl {
    type handleStream = Pin<Box<dyn Stream<Item = Result<EventSourcedStreamOut, Status>> + Send + Sync>>;

    //TODO https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#bidirectional-streaming-rpc

    async fn handle(&self, request: Request<Streaming<EventSourcedStreamIn>>) -> Result<Response<Self::handleStream>, Status> {
        let mut stream = request.into_inner();

        // let output = async_stream::try_stream! {
        //     while let Some(message) = stream.next().await {
        //         //TODO
        //         yield
        //     }
        // };
        //
        // Ok(Response::new(Box::pin(output)
        //     as Pin<Box<dyn Stream<Item = Result<EventSourcedStreamOut, Status>> + Send + Sync + 'static>,
        // >))

        // event_sourced_stream_out::Message::Reply

        //EventSourcedStreamIn.message.
        Err(Status::unimplemented("not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:9000".parse().unwrap();
    let server = EventSourcedServerImpl::default();

    let svc = EventSourcedServer::new(server);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

