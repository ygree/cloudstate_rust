use protocols::protocol::cloudstate::{eventsourced::{
    EventSourcedStreamIn, EventSourcedStreamOut, EventSourcedReply,
    event_sourced_stream_in, event_sourced_stream_out,
    event_sourced_server::EventSourced,
}, entity_discovery_server::EntityDiscovery, ProxyInfo, EntitySpec, UserFunctionError, Entity, ServiceInfo, ClientAction,
  client_action::Action
};
use tonic::{Status, Streaming, Response, Request};
use std::pin::Pin;
// use futures_core::Stream; // TODO: it caused compile issues
use futures::Stream;
use bytes::Bytes;
use std::sync::Arc;
use cloudstate_core::eventsourced::{EntityAction, EntityRegistry, EventSourcedEntityHandler, EntityResponse};

pub struct EntityDiscoveryServerImpl {
    pub descriptor_set: Vec<u8>,
}

#[tonic::async_trait]
impl EntityDiscovery for EntityDiscoveryServerImpl {

    async fn discover(&self, request: Request<ProxyInfo>) -> Result<Response<EntitySpec>, Status> {
        let info = request.into_inner();
        println!("---> EntityDiscovery.discover : request.message = {:?}", info);

        //TODO see Java impl for reference: io.cloudstate.javasupport.impl.EntityDiscoveryImpl#discover

        //TODO check that request.into_inner().supported_entity_types contains entity_type
        // if not log an error

        let reply = EntitySpec {
            proto: self.descriptor_set.clone(),
            entities: vec![
                Entity {
                    entity_type: "cloudstate.eventsourced.EventSourced".to_owned(),
                    service_name: "com.example.shoppingcart.ShoppingCart".to_owned(), //TODO ???
                    persistence_id: "shopping_cart".to_owned(),
                }
            ],
            service_info: Some(
                ServiceInfo {
                    service_name: "shopping-cart".to_owned(), //TODO should be provided from the service builder / descriptor
                    service_version: "0.1".to_owned(),
                    service_runtime: "rustc 1.43.0 (4fb7144ed 2020-04-20)".to_owned(), //TODO use rust version
                    support_library_name: "cloudstate".to_owned(),
                    support_library_version: "0.1".to_owned(), //TODO use version of the library
                }
            ),
        };

        Ok(Response::new(reply))
    }

    async fn report_error(&self, request: Request<UserFunctionError>) -> Result<Response<()>, Status> {
        println!("---> EntityDiscovery.report_error: error = {:?}", request.into_inner());

        Ok(Response::new(()))
    }
}

pub struct EventSourcedServerImpl(pub Arc<EntityRegistry>);

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

        match known_msg {
            Message::Init(init) => {
                println!("init service: {} entity_id: {}", init.service_name, init.entity_id);
                match &self {
                    EventSourcedSession::New(entity_registry) => {
                        let service_name = init.service_name;
                        match entity_registry.create(&service_name) {
                            Some(mut entity) => {
                                if let Some(snapshot) = init.snapshot {
                                    println!("snapshot: seq_id = {}", snapshot.snapshot_sequence);

                                    if let Some(snapshot_any) = snapshot.snapshot {
                                        let type_url = snapshot_any.type_url;
                                        let bytes = Bytes::from(snapshot_any.value);
                                        entity.snapshot_received(type_url, bytes);
                                    }
                                } else {
                                    println!("No initial snapshot provided!");
                                }
                                *self = EventSourcedSession::Initialized(entity);
                            },
                            None => {
                                println!("Unknown service_name {}", service_name);
                            },
                        }
                    }
                    EventSourcedSession::Initialized(_entity) => {
                        println!("Entity already initialized!");
                    },
                };
                None
            },
            Message::Event(evt) => {
                match self {
                    EventSourcedSession::Initialized(entity) => {
                        if let Some(event_any) = evt.payload {
                            let type_url = event_any.type_url;
                            println!("Handling event: {}", &type_url);
                            let bytes = Bytes::from(event_any.value);
                            entity.event_received(type_url, bytes);
                        }
                    },
                    _ => {
                        println!("Can't handle a event until the entity is initialized!");
                    },
                }
                None
            },
            Message::Command(cmd) => {
                match self {
                    EventSourcedSession::Initialized(entity) => {
                        match cmd.payload {
                            Some(payload_any) => {
                                let type_url = payload_any.type_url;
                                println!("Handling command: {}", type_url);
                                let bytes = Bytes::from(payload_any.value);
                                let entity_resp: EntityResponse = entity.command_received(type_url, bytes);

                                let client_action = match entity_resp.action {
                                    EntityAction::Reply { type_url, bytes } => {
                                        ClientAction { // TODO maybe extract client action local factory?
                                            action: Some(
                                                Action::Reply(
                                                    protocols::protocol::cloudstate::Reply {
                                                        payload: Some(
                                                            ::prost_types::Any {
                                                                type_url,
                                                                value: bytes
                                                            }
                                                        )
                                                    }
                                                )
                                            )
                                        }
                                    },
                                    EntityAction::EmptyReply => {
                                        // TODO construct only once
                                        let mut buf = vec![];
                                        use ::prost::Message;
                                        ().encode(&mut buf).unwrap();
                                        let type_url = "type.googleapis.com/google.protobuf.Empty".to_owned();

                                        ClientAction { // TODO maybe extract client action local factory?
                                            action: Some(
                                                Action::Reply(
                                                    protocols::protocol::cloudstate::Reply {
                                                        payload: Some(
                                                            ::prost_types::Any {
                                                                type_url,
                                                                value: buf
                                                            }
                                                        )
                                                    }
                                                )
                                            )
                                        }
                                    },
                                    EntityAction::Failure { msg } => {
                                        ClientAction { // TODO maybe extract client action local factory?
                                            action: Some(
                                                Action::Failure(
                                                    protocols::protocol::cloudstate::Failure {
                                                        command_id: cmd.id,
                                                        description: msg
                                                    }
                                                )
                                            )
                                        }
                                    },
                                };

                                let events: Vec<_> = entity_resp.events.into_iter().map(
                                    |(tp, bs)| {
                                        //TODO extract method to construct Any?
                                        ::prost_types::Any {
                                            type_url: tp,
                                            value: bs.to_vec() //TODO get rid for the bytes type
                                        }
                                    }
                                ).collect();

                                use event_sourced_stream_out::Message::*;

                                let reply = EventSourcedReply {
                                    command_id: cmd.id,
                                    client_action: Some(client_action),
                                    side_effects: vec![], //TODO side effects
                                    events,
                                    snapshot: None, //TODO snapshot
                                };
                                let out_msg = EventSourcedStreamOut {
                                    message: Some(Reply(reply)),
                                };
                                Some(out_msg)

                            },
                            None => {
                                println!("Command without payload!");
                                None
                            },
                        }

                    },
                    _ => {
                        println!("Can't handle a command until the entity is initialized!");
                        None
                    },
                }
            },
        }

    }
}
