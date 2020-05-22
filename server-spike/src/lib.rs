use protocols::protocol::cloudstate::{eventsourced::{
    EventSourcedStreamIn, EventSourcedStreamOut, EventSourcedReply,
    event_sourced_stream_in, event_sourced_stream_out,
    event_sourced_server::EventSourced,
}, entity_discovery_server::{
    EntityDiscovery,
    EntityDiscoveryServer,
}, ProxyInfo, EntitySpec, UserFunctionError, Entity, ServiceInfo};
use tonic::{Status, Streaming, Response, Request};
use std::pin::Pin;
// use futures_core::Stream; // TODO: it caused compile issues
use futures::Stream;
use bytes::Bytes;

pub type MaybeEntityHandler = Option<Box<dyn EventSourcedEntityHandler + Send + Sync>>;

type EntityHandlerFactory = Box<dyn Fn(&str) -> MaybeEntityHandler + Send + Sync>;

//TODO try to implement an alternative fully typed registry to avoid allocations
pub struct EntityRegistry(pub Vec<EntityHandlerFactory>);

impl EntityRegistry {

    pub fn add_entity_type<T>(&mut self, service_name: &str, _entity: PhantomData<T>)
        where T: EventSourcedEntityHandler + Default + Send + Sync + 'static
    {
        self.add_entity(service_name, || <T as Default>::default());
    }

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

pub struct EntityDiscoveryServerImpl;

#[tonic::async_trait]
impl EntityDiscovery for EntityDiscoveryServerImpl {

    async fn discover(&self, request: Request<ProxyInfo>) -> Result<Response<EntitySpec>, Status> {
        let info = request.into_inner();
        println!("---> EntityDiscovery.discover : request.message = {:?}", info);

        let fd = protocols::example::shoppingcart::file_descriptor_proto();
        let services: &[ServiceDescriptorProto] = fd.get_service();

        println!("---> service : {:?}", services[0].get_name()); //TODO: BOOM! it's empty!
        //TODO: Try to generate file descriptor properly. Should be possible to do with protoc or protobuf_codegen_pure
        // https://github.com/stepancheg/rust-protobuf/issues/292#issuecomment-392607319


        //TODO see Java impl for reference: io.cloudstate.javasupport.impl.EntityDiscoveryImpl#discover

        // Try to resolve: Descriptor dependency [google/protobuf/empty.proto] not found, dependency path: [shoppingcart/shoppingcart.proto]
        let empty_fd = protocols::google::protobuf::empty::file_descriptor_proto();

        let mut ds = FileDescriptorSet::new();
        ds.set_file(RepeatedField::from_vec(vec![fd.clone(), empty_fd.clone()])); //TODO set proper FileDescriptorProto-s

        let ds_bytes: Result<Vec<u8>, _> = ds.write_to_bytes();


        // services[0].cached_size

        //TODO check that request.into_inner().supported_entity_types contains entity_type
        // if not log an error

        /*

protoc --include_imports \
    --proto_path=. \
    --proto_path=protocol \
    --proto_path=frontend \
    --descriptor_set_out=user-function.desc \
    example/shoppingcart/shoppingcart.proto

protoc --proto_path=./ \
    --proto_path=protocol \
    --descriptor_set_out=user-function.desc \
    example/shoppingcart/shoppingcart.proto

         */


        let reply = EntitySpec {
            // proto: descr.to_vec(), // what if we just send it as is? Nope: InvalidProtocolBufferException: While parsing a protocol message, the input ended unexpectedly in the middle of a field.
            proto: ds_bytes.unwrap(), //TODO should be generated
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

// this is untyped entity handler interface for the server implementation
pub trait EventSourcedEntityHandler {
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes);
    fn command_received(&mut self, type_url: String, bytes: Bytes);
}

pub trait HandleCommandContext {
    type Event;

    fn emit_event(&mut self, event: Self::Event);

    //TODO implement fail()
}

struct CommandHandlerContext<T> {
    events: Vec<T>,
}

impl<T> HandleCommandContext for CommandHandlerContext<T> {
    type Event = T;

    fn emit_event(&mut self, event: Self::Event) {
        self.events.push(event);
    }
}

// this is typed entity handler interface to be implemented by user
// NOTE: it can't be used by the server side because it has associated types
pub trait EventSourcedEntity {

    // Entity can only have one type of snapshot thus it's an associated type instead of a trait's type parameter
    type Snapshot : ::protobuf::Message + Default;
    type Command : CommandDecoder;
    type Event;

    fn restore(&mut self, snapshot: Self::Snapshot);

    // This method is called by server and need to bind to the entity typed and delegate call to the user implementation
    fn snapshot_received(&mut self, _type_url: String, bytes: Bytes) {
        match self.decode_snapshot(bytes) {
            Ok(snapshot) => {
                println!("Decoded: {:?}", snapshot);
                self.restore(snapshot);
            }
            Err(_err) => {
                eprintln!("Couldn't decode snapshot!");
            },
        }
    }

    fn decode_snapshot(&self, bytes: Bytes) -> Result<Self::Snapshot, ProtobufError> {
        // default implementation that can be overridden if needed
        protobuf::parse_from_carllerche_bytes::<Self::Snapshot>(&bytes)
    }

    // should be private
    fn command_received(&mut self, type_url: String, bytes: Bytes) {
        if let Some(cmd) = <Self::Command as CommandDecoder>::decode(type_url, bytes) {

            let mut context = CommandHandlerContext {
                events: vec![],
            };

            self.handle_command(cmd, &mut context);

            // apply events
            for evt in context.events {
                self.handle_event(evt);
            }

            //TODO return an effect to be sent to Akka
        }
    }

    //TODO consider changing the signature to return emitted events, error, or effects explicitly without using the context
    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>);

    fn handle_event(&mut self, event: Self::Event);
}

// This provides automatic implementation of EventSourcedEntityHandler for the server from the user's EventSourcedEntity implementation
impl<T> EventSourcedEntityHandler for T
    where T: EventSourcedEntity {

    #[inline]
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes) {
        self.snapshot_received(type_url, bytes)
    }

    #[inline]
    fn command_received(&mut self, type_url: String, bytes: Bytes) {
        // can't decode command here because a real type is needed that is an associated type
        // but associated types don't work with trait objects
        self.command_received(type_url, bytes)
    }
}

use std::sync::Arc;
use std::marker::PhantomData;
use protobuf::{ProtobufError, RepeatedField, Message};
use protobuf::descriptor::{ServiceDescriptorProto, FileDescriptorSet};

pub trait CommandDecoder : Sized {
    fn decode(type_url: String, bytes: Bytes) -> Option<Self>;

    // fn encode(&self) -> Option<(String, Bytes)>;
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
        // use protocols::example::shoppingcart::persistence::*;

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
                            EventSourcedSession::Initialized(_entity) => {
                                println!("Entity already initialized!");
                            },
                        };
                    }
                }
            },
            Message::Event(_evt) => {
                println!("evt")
                //TODO decode event similar to command
            },
            Message::Command(cmd) => {
                match self {
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
