pub mod protocol {
    pub mod cloudstate {
        include!("protocol/cloudstate.rs");
        pub mod eventsourced {
            include!("protocol/cloudstate.eventsourced.rs");
        }
    }
}

pub mod example {
    // protobuf

    pub mod shoppingcart;
    pub mod domain;

    fn shopping_cart_descs() -> &'static [u8] {
        include_bytes!("../shoppingcart.desc")
    }
}

pub mod google {
    pub mod protobuf {
        pub mod empty;
    }
}

pub mod prost_example {
    // prost

    pub mod shoppingcart {
        include!("prost_example/shoppingcart/com.example.shoppingcart.rs");
        pub mod persistence {
            include!("prost_example/shoppingcart/com.example.shoppingcart.persistence.rs");
        }
    }
}
