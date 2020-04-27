
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

    let mut registry = EntityRegistry(vec![]);
    registry.add_entity("shopcart", ShoppingCartEntity::default);
    registry.add_entity("shopcart2", ShoppingCartEntity::default);

    let server = EventSourcedServerImpl(Arc::new(registry));

    let svc = EventSourcedServer::new(server);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

pub type MaybeEntityHandler = Option<Box<dyn EventSourcedEntityHandler + Send + Sync>>;

type EntityHandlerFactory = Box<dyn Fn(&str) -> MaybeEntityHandler + Send + Sync>;

struct EntityRegistry(Vec<EntityHandlerFactory>);

impl EntityRegistry {

    pub fn add_entity<T, F>(&mut self, service_name: &str, creator: F)
        where T: EventSourcedEntityHandler + Send + Sync + 'static,
              F: Fn () -> T + Send + Sync + 'static
    {
        let expected_service_name = service_name.to_owned();
        let create_entity_function: EntityHandlerFactory =
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

    pub fn create(&self, service_name: &str) -> MaybeEntityHandler {
        for creator in &self.0 {
            if let Some(entity) = creator(service_name) {
                return Some(entity);
            }
        }
        return None;
    }
}

struct EventSourcedServerImpl(Arc<EntityRegistry>);

#[tonic::async_trait]
impl EventSourced for EventSourcedServerImpl {
    // These traits are required here because `EventSourced: Send + Sync + 'static`

    // it has generated a type with the first letter in lower case
    // TODO: consider fixing it
    type handleStream = Pin<Box<dyn Stream<Item = Result<EventSourcedStreamOut, Status>> + Send + Sync + 'static>>;

    //TODO https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#bidirectional-streaming-rpc

    async fn handle(&self, request: Request<Streaming<EventSourcedStreamIn>>) -> Result<Response<Self::handleStream>, Status> {
        let mut stream = request.into_inner();

        let registry = self.0.clone();

        let output = async_stream::try_stream! {
            // while let Some(message) = stream.next().await {
            // got from the examples but it doesn't work, perhaps in previous version of tonic before 0.2.0
            // TODO: maybe submit a PR?

            let mut session = EventSourcedSession::new(registry);

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

// this is untyped entity handler interface for the server implementation
trait EventSourcedEntityHandler {
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes);
    fn command_received(&self, type_url: String, bytes: Bytes);
}

// this is typed entity handler interface to be implemented by user
// NOTE: it can't be used by the server side because it has associated type
trait EventSourcedEntity {

    // Entity can only have one type of snapshot thus it's an associated type instead of a trait's type parameter
    type Snapshot : ::prost::Message + Default;

    fn snapshot_loaded(&mut self, snapshot: Self::Snapshot);

    // This method is called by server and need to bind to the entity typed and delegate call to the user implementation
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes) {
        use ::prost::Message; // import Message trait to call decode on Snapshot
        match self.decode_snapshot(bytes) {
            Ok(snapshot) => {
                println!("Decoded: {:?}", snapshot);
                self.snapshot_loaded(snapshot);
            }
            Err(err) => {
                eprintln!("Couldn't decode snapshot!");
            },
        }
    }

    fn decode_snapshot(&self, bytes: Bytes) -> Result<Self::Snapshot, DecodeError> {
        // default implementation that can be overridden if needed
        use ::prost::Message; // import Message trait to call decode on Snapshot
        // Self::Snapshot::decode(bytes)
        <Self::Snapshot as Message>::decode(bytes) // explicitly call a trait's associated method
    }

    //TODO there is a separate proto message for each type of command.
    type Command : ::prost::Message + Default;

    fn command_received(&self, type_url: String, bytes: Bytes) {
        use ::prost::Message; // import Message trait to call decode on Command
        match self.decode_command(bytes) {
            Ok(command) => {
                println!("Decoded: {:?}", command);
                // self.snapshot_loaded(snapshot);
            }
            Err(err) => {
                eprintln!("Couldn't decode snapshot!");
            },
        }


        //TODO pass a command to the command handler and expect an effect back
        //TODO call an event handler for new events
        //TODO return an effect to be sent to Akka
    }

    fn decode_command(&self, bytes: Bytes) -> Result<Self::Command, DecodeError> {
        // default implementation that can be overridden if needed
        use ::prost::Message; // import Message trait to call decode on Command
        // Self::Command::decode(bytes)
        <Self::Command as Message>::decode(bytes) // explicitly call a trait's associated method
    }
}

// This provides automatic implementation of EventSourcedEntityHandler for the server from the user's EventSourcedEntity implementation
impl<T> EventSourcedEntityHandler for T
    where T: EventSourcedEntity {

    fn snapshot_received(&mut self, type_url: String, bytes: Bytes) {
        self.snapshot_received(type_url, bytes)
    }

    fn command_received(&self, type_url: String, bytes: Bytes) {
        self.command_received(type_url, bytes)
    }
}

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

impl EventSourcedEntity for ShoppingCartEntity {

    type Snapshot = Cart;
    type Command = Cart; //TODO: need command types. There is now one command type but a separate one for each command

    fn snapshot_loaded(&mut self, snapshot: Self::Snapshot) {
        self.0 = snapshot;
        println!("Snapshot Loaded: {:?}", self.0);
    }
}

enum EventSourcedSession {
    New(Arc<EntityRegistry>),
    Initialized(Box<dyn EventSourcedEntityHandler + Send + Sync>),
}

impl EventSourcedSession {

    fn new(registry: Arc<EntityRegistry>) -> EventSourcedSession {
        EventSourcedSession::New(registry)
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

                        let type_url = snapshot_any.type_url;
                        let bytes = Bytes::from(snapshot_any.value);

                        match &self {
                            EventSourcedSession::New(entity_registry) => {
                                match entity_registry.create(&service_name) {
                                    Some(mut entity) => {
                                        entity.snapshot_received(type_url, bytes);
                                        *self = EventSourcedSession::Initialized(entity);
                                    },
                                    None => {
                                        println!("Unknown service_name {}", service_name);
                                    },
                                }
                            }
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
                match &self {
                    EventSourcedSession::Initialized(entity) => {
                        println!("Handling a command!");
                        match cmd.payload {
                            Some(payload_any) => {
                                let type_url = payload_any.type_url;
                                let bytes = Bytes::from(payload_any.value);

                                entity.command_received(type_url, bytes);
                            },
                            None => {
                                println!("Command without payload!");
                            },
                        }

                    },
                    _ => {
                        println!("Can't handle a command until the entity is initialized!");
                    },
                };

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
