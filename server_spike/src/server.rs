
use protocols::cloudstate::eventsourced::event_sourced_server::{EventSourced, EventSourcedServer};
use protocols::cloudstate::eventsourced::{EventSourcedStreamIn, EventSourcedStreamOut, EventSourcedReply};
use protocols::cloudstate::eventsourced::{event_sourced_stream_in, event_sourced_stream_out};
use tonic::{Status, Streaming, Response, Request};
use tonic::transport::Server;
use std::pin::Pin;
// use futures_core::Stream; // TODO: it caused compile issues
use futures::Stream;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:9000".parse().unwrap();
    let server = EventSourcedServerImpl::default();

    let svc = EventSourcedServer::new(server);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

#[derive(Default)]
struct EventSourcedServerImpl;

#[tonic::async_trait]
impl EventSourced for EventSourcedServerImpl {
    // it has generated a type with the first letter in lower case
    // TODO: consider fixing it
    type handleStream = Pin<Box<dyn Stream<Item = Result<EventSourcedStreamOut, Status>> + Send + Sync + 'static>>;

    //TODO https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#bidirectional-streaming-rpc

    async fn handle(&self, request: Request<Streaming<EventSourcedStreamIn>>) -> Result<Response<Self::handleStream>, Status> {
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            // while let Some(message) = stream.next().await {
            // got from the examples but it doesn't work, perhaps in previous version of tonic before 0.2.0
            // TODO: maybe submit a PR?

            let mut session = EventSourcedSession();
            session.session_started();

            while let Some(in_msg) = stream.message().await? { // msg: EventSourcedStreamIn

                if let Some(known_msg) = in_msg.message {
                    // none if protobuf version has unknown enum

                    if let Some(out_msg) = session.handle_known_msg(known_msg) {
                        yield out_msg;
                    }
                } else {
                    println!("unknown message")
                }
            }
            session.session_finished();
        };

        Ok(Response::new(Box::pin(output) as Self::handleStream))
        // Err(Status::unimplemented("not implemented"))
    }
}

trait EventSourcedHandler {
    fn session_started(&mut self);
    fn session_finished(&mut self);
    fn handle_known_msg(&mut self, known_msg: event_sourced_stream_in::Message) -> Option<EventSourcedStreamOut>;
}

struct EventSourcedSession();

impl EventSourcedHandler for EventSourcedSession {
    fn session_started(&mut self) {
        println!("starting session");
    }

    fn session_finished(&mut self) {
        //TODO it's not called if the session is not closed properly on the client.
        // It will lead to the resource leak. How to prevent it?
        println!("session finished");
    }

    fn handle_known_msg(&mut self, known_msg: event_sourced_stream_in::Message) -> Option<EventSourcedStreamOut> {
        use event_sourced_stream_in::Message;
        use protocols::shoppingcart::persistence::*;

        match known_msg {
            Message::Init(init) => {
                println!("init service: {} entity_id: {}", init.service_name, init.entity_id);
                if let Some(snapshot) = init.snapshot {
                    println!("snapshot: seq_id = {}", snapshot.snapshot_sequence);
                    if let Some(snapshot_any) = snapshot.snapshot {
                        let bytes = bytes::Bytes::from(snapshot_any.value);
                        use ::prost::Message; // import Message trait to call decode on Cart
                        let result = Cart::decode(bytes);
                        match result {
                            Ok(cart) => {
                                println!("Decoded: {:?}", cart);
                            },
                            Err(err) => {
                                eprintln!("Couldn't decode: {}", snapshot_any.type_url);
                            },
                        }
                    }
                }
            },
            Message::Event(evt) => {
                println!("evt")
            },
            Message::Command(cmd) => {
                println!("cmd")
            },
        }

        use event_sourced_stream_out::Message::*;
        let reply = EventSourcedReply {
            command_id: 1i64, // Only for input input Command
            client_action: None, //TODO action
            side_effects: vec![], //TODO side effects
            events: vec![], //TODO events
            snapshot: None, //TODO snapshot
        };
        let out_msg = EventSourcedStreamOut {
            message: Some(Reply(reply)),
        };
        Some(out_msg)
    }
}
