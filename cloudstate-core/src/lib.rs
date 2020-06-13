
use bytes::Bytes;

//TODO rename because it's also used for snapshots and events
// CloudstateMessage
pub trait AnyMessage: Sized {
    //TODO may be use &str?
    fn decode(type_url: &str, bytes: Bytes) -> Option<Self>;

    fn encode(&self) -> Option<(String, Vec<u8>)>;
}

pub mod eventsourced;
