use bytes::Bytes;
use crate::AnyMessage;

struct EntityHandlerFactory {
    entity_name: String,
    creator: Box<dyn Fn() -> Box<dyn EventSourcedEntityHandler + Send + Sync> + Send + Sync>,
}

pub struct EntityRegistry(Vec<EntityHandlerFactory>);

impl EntityRegistry {

    pub fn new() -> EntityRegistry {
        EntityRegistry(vec![])
    }

    pub fn eventsourced_entity<T, F>(&mut self, service_name: &str, creator: F)
        where T: EventSourcedEntityHandler + Send + Sync + 'static,
              F: Fn () -> T + Send + Sync + 'static
    {
        let entity_name = service_name.to_owned();
        let create_entity_function = EntityHandlerFactory {
            entity_name,
            creator: Box::new(move || {
                Box::new(creator())
            }),
        };
        self.0.push(create_entity_function);
    }

    pub fn create(&self, service_name: &str) -> Option<Box<dyn EventSourcedEntityHandler + Send + Sync>> {
        for factory in &self.0 {
            if factory.entity_name == service_name {
                let f = &factory.creator;
                return Some(f())
            }
        }
        return None;
    }
}

pub trait EventsourcedContext {
    type Event;

    fn emit_event(&mut self, event: Self::Event);
}

struct EventsourcedContextData<T> {
    events: Vec<T>,
}

impl<T> EventsourcedContext for EventsourcedContextData<T> {
    type Event = T;

    fn emit_event(&mut self, event: Self::Event) {
        self.events.push(event);
    }
}

pub enum Response<T: AnyMessage> {
    Reply(T),
    EmptyReply,
    // Forward,
    // NoReply,
}

// this is typed entity handler interface to be implemented by user
// NOTE: it can't be used by the server side as-is because it has associated types.
//  Such traits can't be used as trait objects.
pub trait EventSourcedEntity {

    // Entity can only have one type of snapshot thus it's an associated type instead of a trait's type parameter
    type Command : AnyMessage;
    type Event : AnyMessage;
    type Snapshot : AnyMessage;
    type Response : AnyMessage;

    fn restore(&mut self, snapshot: Self::Snapshot);

    // This method is called by server and need to bind to the entity typed and delegate call to the user implementation
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes) {
        if let Some(snapshot) = <Self::Snapshot as AnyMessage>::decode(&type_url, bytes) {
            println!("Received snapshot!");
            self.restore(snapshot);
        } else {
            eprintln!("Couldn't decode snapshot!");
        }
    }

    fn command_received(&mut self, type_url: String, bytes: Bytes) -> EntityResponse {
        println!("Handing received command {}", &type_url);
        if let Some(cmd) = <Self::Command as AnyMessage>::decode(&type_url, bytes) {

            let mut context = EventsourcedContextData {
                events: vec![],
            };

            let result = self.handle_command(cmd, &mut context);

            let events = context.events.iter().flat_map(|e| {
                match <Self::Event as AnyMessage>::encode(&e) {
                    Some((type_id, bytes)) => Some((type_id, Bytes::from(bytes))),
                    _ => None,
                }
            }).collect();

            // apply events
            for evt in context.events {
                self.handle_event(evt);
            }
            //TODO return an effect to be sent to Akka

            let action: EntityAction = match result {
                Ok(Response::Reply(resp)) => {
                    match <Self::Response as AnyMessage>::encode(&resp) {
                        Some((type_url, bytes)) => {
                            EntityAction::Reply {
                                type_url,
                                bytes
                            }
                        }
                        _ => {
                            //TODO log an error
                            EntityAction::Failure {
                                msg: "Server error: couldn't encode the response".to_owned()
                            }
                        }
                    }
                },
                Ok(Response::EmptyReply) => EntityAction::EmptyReply,
                Err(msg) => {
                    EntityAction::Failure {
                        msg
                    }
                }
            };

            EntityResponse {
                action,
                events
            }
        } else {
            println!("Couldn't decode command {}", type_url);
            EntityResponse {
                action: EntityAction::Failure {
                    msg: "Server error: couldn't encode the response".to_owned()
                },
                events: vec![],
            }
        }
    }

    fn handle_command(&self, command: Self::Command, context: &mut impl EventsourcedContext<Event=Self::Event>) -> Result<Response<Self::Response>, String>;

    fn event_received(&mut self, type_url: String, bytes: Bytes) {
        println!("Handing received event {}", &type_url);

        if let Some(evt) = <Self::Event as AnyMessage>::decode(&type_url, bytes) {
            self.handle_event(evt);
        }
    }

    fn handle_event(&mut self, event: Self::Event);
}

//TODO maybe rename to ClientAction but it will overlap with the prototype name?
pub enum EntityAction {
    Reply {
        type_url: String,
        bytes: Vec<u8>,
    },
    EmptyReply,
    Failure {
        msg: String,
    },
    //TODO Forward,
}

pub struct EntityResponse {
    pub action: EntityAction,
    pub events: Vec<(String, Bytes)>,
// side_effects: vec![], //TODO side effects
// snapshot: None, //TODO snapshot
}


// this is untyped entity handler interface for the server implementation
pub trait EventSourcedEntityHandler {
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes);
    fn command_received(&mut self, type_url: String, bytes: Bytes) -> EntityResponse;
    fn event_received(&mut self, type_url: String, bytes: Bytes);
}

// This provides automatic implementation of EventSourcedEntityHandler for the server from the user's EventSourcedEntity implementation
impl<T> EventSourcedEntityHandler for T
    where T: EventSourcedEntity {

    #[inline]
    fn snapshot_received(&mut self, type_url: String, bytes: Bytes) {
        self.snapshot_received(type_url, bytes)
    }

    #[inline]
    fn command_received(&mut self, type_url: String, bytes: Bytes) -> EntityResponse {
        // can't decode command here because a real type is needed that is an associated type
        // but associated types don't work with trait objects
        self.command_received(type_url, bytes)
    }

    #[inline]
    fn event_received(&mut self, type_url: String, bytes: Bytes) {
        // can't decode event here because a real type is needed that is an associated type
        // but associated types don't work with trait objects
        self.event_received(type_url, bytes)
    }
}

