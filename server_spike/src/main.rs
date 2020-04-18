
use protocols::cloudstate::eventsourced::event_sourced_server::{EventSourced, EventSourcedServer};
use protocols::cloudstate::eventsourced::{EventSourcedStreamIn, EventSourcedStreamOut};
use protocols::cloudstate::eventsourced::{event_sourced_stream_in, event_sourced_stream_out};
use tonic::{Status, Streaming, Response, Request};
use tonic::transport::Server;
use std::pin::Pin;
// use futures_core::Stream; // TODO: it caused compile issues
use futures::Stream;

#[derive(Default)]
struct EventSourcedServerImpl;

#[tonic::async_trait]
impl EventSourced for EventSourcedServerImpl {
    type handleStream = Pin<Box<dyn Stream<Item = Result<EventSourcedStreamOut, Status>> + Send + Sync + 'static>>;

    //TODO https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#bidirectional-streaming-rpc

    async fn handle(&self, request: Request<tonic::Streaming<EventSourcedStreamIn>>) -> Result<Response<Self::handleStream>, Status> {
        let mut stream = request.into_inner();

        //         // match message {
        //         //     event_sourced_stream_in::Message::Init(init) => (),
        //         //     event_sourced_stream_in::Message::Event(init) => (),
        //         //     event_sourced_stream_in::Message::Command(init) => (),
        //         // }
        // // //         //TODO
        //         let msg = EventSourcedStreamOut {
        //             message: None,
        //         }
        //         yield Response::new(msg);

        //message: event_sourced_stream_in::Message

        let output = async_stream::try_stream! {
            while let Some(message) = stream.message().await? {
            // while let Some(message) = stream.next().await {
                let msg = EventSourcedStreamOut {
                    message: None,
                };
                yield msg;
            }
        };

        Ok(Response::new(Box::pin(output) as Self::handleStream))
        // Err(Status::unimplemented("not implemented"))
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

