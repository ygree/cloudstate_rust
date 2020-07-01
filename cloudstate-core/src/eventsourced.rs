use bytes::Bytes;
use crate::AnyMessage;

pub struct EntityRegistry {
    pub event_sourced_entities: Vec<EventSourcedEntityDescriptor>,
}

pub struct EventSourcedEntityDescriptor {
    pub service_name: String,
    pub persistence_id : String,
    handler_factory: Box<dyn Fn() -> Box<dyn EventSourcedEntityHandler + Send + Sync> + Send + Sync>,
}

impl EntityRegistry {

    pub fn new() -> EntityRegistry {
        EntityRegistry {
            event_sourced_entities: vec![],
        }
    }

    pub fn register_event_sourced_entity<F, H>(&mut self, service_name: &str, persistence_id: &str, handler_factory: F)
        where F: Fn () -> H + Send + Sync + 'static,
              H: EventSourcedEntityHandler + Send + Sync + 'static
    {
        if self.event_sourced_entities.iter().find(|v| v.service_name == service_name).is_some() {
            panic!("Event sourced entity {} already registered!", service_name);
        }

        let entity_name = service_name.to_owned();
        let persistence_id = persistence_id.to_owned();

        let create_entity_function = EventSourcedEntityDescriptor {
            service_name: entity_name,
            persistence_id,
            handler_factory: Box::new(move || {
                Box::new(handler_factory())
            }),
        };
        self.event_sourced_entities.push(create_entity_function);
    }

    pub fn create(&self, entity_name: &str) -> Option<Box<dyn EventSourcedEntityHandler + Send + Sync>> {
        for factory in &self.event_sourced_entities {
            if factory.service_name == entity_name {
                let f = &factory.handler_factory;
                return Some(f())
            }
        }
        return None;
    }
}

pub trait CommandContext<T: AnyMessage> {
    fn emit_event(&mut self, event: T);
}

struct CommandContextData<T> {
    events: Vec<T>,
    snapshot_every: i64,
}

impl<T: AnyMessage> CommandContext<T> for CommandContextData<T> {

    fn emit_event(&mut self, event: T) {
        //TODO how to call an event handler from here?
        // it might be useful if we would like to apply events
        // as soon as they are emitted
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

    fn snapshot_every(&self) -> i64 {
        100 //TODO get default from config
    }

    // This method is called by server and need to bind to the entity typed and delegate call to the user implementation
    fn snapshot_received(&mut self, type_url: &str, bytes: Bytes) {
        if let Some(snapshot) = <Self::Snapshot as AnyMessage>::decode(&type_url, bytes) {
            println!("Received snapshot!");
            self.handle_snapshot(snapshot);
        } else {
            eprintln!("Couldn't decode snapshot!");
        }
    }

    fn handle_snapshot(&mut self, snapshot: Self::Snapshot);

    fn command_received(&mut self, type_url: &str, bytes: Bytes) -> EntityResponse {
        println!("Handing received command {}", &type_url);
        if let Some(cmd) = <Self::Command as AnyMessage>::decode(&type_url, bytes) {

            let mut context = CommandContextData::<Self::Event> {
                events: vec![],
                snapshot_every: self.snapshot_every(),
                //TODO pass event_handler to be called immediately on emit_event
            };

            // self.event_received()

            let result = self.handle_command(cmd, &mut context);

            let events: Vec<(String, Bytes)> = match result {
                Ok(_) => {
                    context.events.iter().flat_map(|e| {
                        match <Self::Event as AnyMessage>::encode(&e) {
                            Some((type_id, bytes)) => Some((type_id, Bytes::from(bytes))),
                            _ => None,
                        }
                    }).collect()
                },
                Err(_) => {
                    vec![]
                },
            };

            for (type_url, bytes) in events.iter() {
                self.event_received(&type_url, bytes.clone());
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

    fn handle_command(&self, command: Self::Command, context: &mut impl CommandContext<Self::Event>) -> Result<Response<Self::Response>, String>;

    fn event_received(&mut self, type_url: &str, bytes: Bytes) {
        println!("Handling received event {}", type_url);

        if let Some(evt) = <Self::Event as AnyMessage>::decode(type_url, bytes) {
            self.handle_event(evt);
        }
        //TODO what to do if can't deserialize event?
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
    fn snapshot_received(&mut self, type_url: &str, bytes: Bytes);
    fn command_received(&mut self, type_url: &str, bytes: Bytes) -> EntityResponse;
    fn event_received(&mut self, type_url: &str, bytes: Bytes);
}

// This provides automatic implementation of EventSourcedEntityHandler for the server from the user's EventSourcedEntity implementation
impl<T> EventSourcedEntityHandler for T
    where T: EventSourcedEntity {

    #[inline]
    fn snapshot_received(&mut self, type_url: &str, bytes: Bytes) {
        self.snapshot_received(type_url, bytes)
    }

    #[inline]
    fn command_received(&mut self, type_url: &str, bytes: Bytes) -> EntityResponse {
        // can't decode command here because a real type is needed that is an associated type
        // but associated types don't work with trait objects
        self.command_received(type_url, bytes)
    }

    #[inline]
    fn event_received(&mut self, type_url: &str, bytes: Bytes) {
        // can't decode event here because a real type is needed that is an associated type
        // but associated types don't work with trait objects
        self.event_received(type_url, bytes)
    }
}

