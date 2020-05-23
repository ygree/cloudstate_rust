
use bytes::Bytes;

//TODO rename because it's also used for snapshots and events
pub trait CommandDecoder : Sized {
    fn decode(type_url: String, bytes: Bytes) -> Option<Self>;

    // fn encode(&self) -> Option<(String, Bytes)>;
}

pub mod eventsourced;
