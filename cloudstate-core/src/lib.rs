
use bytes::Bytes;

pub trait AnyMessage: Sized {
    fn decode(type_url: &str, bytes: Bytes) -> Option<Self>;

    fn encode(&self) -> Option<(String, Vec<u8>)>;
}

pub mod eventsourced;
