use ::prost::Message;
use bytes::Bytes;

pub fn add_line_item() -> AddLineItem {
    AddLineItem {
        user_id: "user_id".to_owned(),
        product_id: "product_id".to_owned(),
        name: "name".to_owned(),
        quantity: 1,
    }
}

pub fn encode<T: Message>(msg: &T) -> Bytes {
    let mut buf = vec![];
    msg.encode(&mut buf).unwrap();
    Bytes::from(buf)
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddLineItem {
    #[prost(string, tag = "1")]
    pub user_id: std::string::String,
    #[prost(string, tag = "2")]
    pub product_id: std::string::String,
    #[prost(string, tag = "3")]
    pub name: std::string::String,
    #[prost(int32, tag = "4")]
    pub quantity: i32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoveLineItem {
    #[prost(string, tag = "1")]
    pub user_id: std::string::String,
    #[prost(string, tag = "2")]
    pub product_id: std::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShoppingCart {
    #[prost(string, tag = "1")]
    pub user_id: std::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LineItem {
    #[prost(string, tag = "1")]
    pub product_id: std::string::String,
    #[prost(string, tag = "2")]
    pub name: std::string::String,
    #[prost(int32, tag = "3")]
    pub quantity: i32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Cart {
    #[prost(message, repeated, tag = "1")]
    pub items: ::std::vec::Vec<LineItem>,
}

