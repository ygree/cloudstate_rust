

#[cfg(test)]
mod tests {
    use ::prost::Message;
    use bytes::Bytes;
    use server_spike::CommandDecoder;
    use command_macro_derive::CommandDecoder;

    #[derive(CommandDecoder, Debug, PartialEq)]
    pub enum ShoppingCartCommand {
        AddLine(AddLineItem),
        RemoveLine(RemoveLineItem),
        GetCart(GetShoppingCart),
    }

    #[test]
    fn test_command_decoder() {
        let msg = add_line_item();

        let bytes = encode(&msg);

        let result = <ShoppingCartCommand as CommandDecoder>::decode("AddLineItem".to_owned(), bytes);

        assert_eq!(result, Some(ShoppingCartCommand::AddLine(msg)));
    }

    #[test]
    fn test_command_decoder_with_incorrect_type() {
        let msg = add_line_item();

        let bytes = encode(&msg);

        let result = <ShoppingCartCommand as CommandDecoder>::decode("wrong-type".to_owned(), bytes);

        assert_eq!(result, None);
    }

    fn add_line_item() -> AddLineItem {
        AddLineItem {
            user_id: "user_id".to_owned(),
            product_id: "product_id".to_owned(),
            name: "name".to_owned(),
            quantity: 1,
        }
    }

    fn encode<T: Message>(msg: &T) -> Bytes {
        let mut buf = vec![];
        msg.encode(&mut buf);
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

}
