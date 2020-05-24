use bytes::Bytes;
use std::marker::PhantomData;
use crate::CommandDecoder;

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
// NOTE: it can't be used by the server side as is because it has associated types.
//  Such traits can't be used as trait objects.
pub trait EventSourcedEntity {

    // Entity can only have one type of snapshot thus it's an associated type instead of a trait's type parameter
    type Command : CommandDecoder;
    type Response : CommandDecoder;

    type Snapshot : CommandDecoder;
    type Event;

    fn restore(&mut self, snapshot: Self::Snapshot);

    // This method is called by server and need to bind to the entity typed and delegate call to the user implementation
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes) {
        if let Some(snapshot) = <Self::Snapshot as CommandDecoder>::decode(type_url, bytes) {
            println!("Received snapshot!");
            self.restore(snapshot);
        } else {
            eprintln!("Couldn't decode snapshot!");
        }
    }

    // should be private
    fn command_received(&mut self, type_url: String, bytes: Bytes) -> Option<(String, Bytes)> {
        println!("Handing received command {}", &type_url);
        if let Some(cmd) = <Self::Command as CommandDecoder>::decode(type_url, bytes) {

            let mut context = CommandHandlerContext {
                events: vec![],
            };

            let response_opt = self.handle_command(cmd, &mut context);

            // apply events
            for evt in context.events {
                self.handle_event(evt);
            }
            //TODO return an effect to be sent to Akka

            //TODO encode response_opt and return
            if let Some(resp) = response_opt {
                <Self::Response as CommandDecoder>::encode(&resp)
            } else {
                None
            }
        } else {
            None
        }
    }

    //TODO consider changing the signature to return emitted events, error, or effects explicitly without using the context
    fn handle_command(&self, command: Self::Command, context: &mut impl HandleCommandContext<Event=Self::Event>) -> Option<Self::Response>;

    fn handle_event(&mut self, event: Self::Event);
}

// this is untyped entity handler interface for the server implementation
pub trait EventSourcedEntityHandler {
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes);
    fn command_received(&mut self, type_url: String, bytes: Bytes) -> Option<(String, Bytes)>;
}

// This provides automatic implementation of EventSourcedEntityHandler for the server from the user's EventSourcedEntity implementation
impl<T> EventSourcedEntityHandler for T
    where T: EventSourcedEntity {

    #[inline]
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes) {
        self.snapshot_received(type_url, bytes)
    }

    #[inline]
    fn command_received(&mut self, type_url: String, bytes: Bytes) -> Option<(String, Bytes)> {
        // can't decode command here because a real type is needed that is an associated type
        // but associated types don't work with trait objects
        self.command_received(type_url, bytes)
    }
}

