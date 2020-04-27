
use protocols::cloudstate::eventsourced::event_sourced_server::{EventSourced, EventSourcedServer};
use protocols::cloudstate::eventsourced::{EventSourcedStreamIn, EventSourcedStreamOut, EventSourcedReply};
use protocols::cloudstate::eventsourced::{event_sourced_stream_in, event_sourced_stream_out};
use tonic::{Status, Streaming, Response, Request};
use tonic::transport::Server;
use std::pin::Pin;
// use futures_core::Stream; // TODO: it caused compile issues
use futures::Stream;
use bytes::Bytes;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:9000".parse().unwrap();

    //TODO how to construct a server that handles more than one type of entity?
    // probably need some kind combinator type. See Server::builder (below) for an example.
    // let factory = EntityFactory(entity);
    let mut factory = EntityFactory(vec![]);
    factory.add_entity("shopcart", ShoppingCartEntity::default);

    let server = EventSourcedServerImpl(Arc::new(factory));

    let svc = EventSourcedServer::new(server);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

struct EntityFactory(Vec<Box<dyn Fn(&str) -> Option<Box<dyn EventSourcedEntityHandler + Send + Sync>> + Send + Sync>>);

impl EntityFactory {

    fn add_entity<T, F>(&mut self, service_name: &str, creator: F)
        where T: EventSourcedEntityHandler + Send + Sync + 'static,
              F: Fn () -> T + Send + Sync + 'static
    {
        let expected_service_name = service_name.to_owned();
        let create_entity_function: Box<dyn Fn(&str) -> Option<Box<dyn EventSourcedEntityHandler + Send + Sync>> + Send + Sync> =
            Box::new(move |name| {
                if name == expected_service_name {
                    let f = &creator;
                    Some(Box::new(f()))
                } else {
                    None
                }
            });

        self.0.push(Box::new(create_entity_function));
    }

    fn create(&self, service_name: &str) -> Option<Box<dyn EventSourcedEntityHandler + Send + Sync>> {
        for creator in &self.0 {
            if let Some(entity) = creator(service_name) {
                return Some(entity);
            }
        }
        return None;
    }
}

struct EventSourcedServerImpl(Arc<EntityFactory>);

#[tonic::async_trait]
impl EventSourced for EventSourcedServerImpl {
    // These traits are required here because `EventSourced: Send + Sync + 'static`

    // it has generated a type with the first letter in lower case
    // TODO: consider fixing it
    type handleStream = Pin<Box<dyn Stream<Item = Result<EventSourcedStreamOut, Status>> + Send + Sync + 'static>>;

    //TODO https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#bidirectional-streaming-rpc

    async fn handle(&self, request: Request<Streaming<EventSourcedStreamIn>>) -> Result<Response<Self::handleStream>, Status> {
        let mut stream = request.into_inner();

        let factory = self.0.clone();

        let output = async_stream::try_stream! {
            // while let Some(message) = stream.next().await {
            // got from the examples but it doesn't work, perhaps in previous version of tonic before 0.2.0
            // TODO: maybe submit a PR?

            let mut session = EventSourcedSession::new(factory);

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
            session.session_finished(); // might not be called
        };

        Ok(Response::new(Box::pin(output) as Self::handleStream))
        // Err(Status::unimplemented("not implemented"))
    }
}

trait EventSourcedEntityHandler {
    fn snapshot_loaded(&self, bytes: bytes::Bytes);
}

//TODO extract it into a separate module
trait EventSourcedEntity: EventSourcedEntityHandler {

    // Entity can only have one type of snapshot thus it's an associated type instead of a trait's type parameter
    type Snapshot : ::prost::Message + Default;

    fn decode_snapshot(&self, bytes: bytes::Bytes) -> Result<Self::Snapshot, DecodeError> {
        // default implementation that can be overridden if needed
        use ::prost::Message; // import Message trait to call decode on Snapshot
        // Self::Snapshot::decode(bytes)
        <Self::Snapshot as Message>::decode(bytes) // explicitly call a trait's associated method
    }

    fn snapshot_loaded(&mut self, snapshot: Self::Snapshot);
}

#[derive(Clone)] // clone is needed to move the copy into the async stream
struct ShoppingCartEntity(Cart);

impl Default for ShoppingCartEntity {
    fn default() -> Self {
        Self(
            Cart {
                items: vec![],
            }
        )
    }
}

use protocols::shoppingcart::persistence::*;
use prost::DecodeError;
use std::sync::Arc;

impl EventSourcedEntityHandler for ShoppingCartEntity {

    fn snapshot_loaded(&self, bytes: Bytes) {
        use ::prost::Message; // import Message trait to call decode on Snapshot
        match <Cart as Message>::decode(bytes) {
            Ok(snapshot) => {
                println!("Decoded: {:?}", snapshot);
            }
            Err(err) => {
                eprintln!("Couldn't decode snapshot!");
            },
        }

    }
}

impl EventSourcedEntity for ShoppingCartEntity {

    type Snapshot = Cart;

    fn snapshot_loaded(&mut self, snapshot: Self::Snapshot) {
        self.0 = snapshot;
        println!("Snapshot Loaded: {:?}", self.0);
    }
}

enum EventSourcedSession {
    New(Arc<EntityFactory>),
    Initialized(Box<dyn EventSourcedEntityHandler + Send + Sync>),
}

impl EventSourcedSession {
    fn new(factory: Arc<EntityFactory>) -> EventSourcedSession {
        EventSourcedSession::New(factory)
    }

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
                //TODO lookup service implementation by service_name
                println!("init service: {} entity_id: {}", init.service_name, init.entity_id);
                if let Some(snapshot) = init.snapshot {
                    println!("snapshot: seq_id = {}", snapshot.snapshot_sequence);
                    if let Some(snapshot_any) = snapshot.snapshot {

                        let service_name = init.service_name;

                        let bytes = bytes::Bytes::from(snapshot_any.value);

                        match &self {
                            EventSourcedSession::New(factory) => {
                                match factory.create(&service_name) {
                                    Some(entity) => {
                                        entity.snapshot_loaded(bytes);
                                        *self = EventSourcedSession::Initialized(entity);
                                    },
                                    None => {
                                        println!("Unknown service_name {}", service_name);
                                    },
                                }
                            },
                            EventSourcedSession::Initialized(entity) => {
                                println!("Entity already initialized!");
                            },
                        };
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
            command_id: 1i64, // Only for input Command
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
