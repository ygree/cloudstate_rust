pub mod protocol {
    pub mod cloudstate {
        include!("protocol/cloudstate.rs");
        pub mod eventsourced {
            include!("protocol/cloudstate.eventsourced.rs");
        }
    }
}

pub mod example {
    pub fn shopping_cart_descriptor_set() -> &'static [u8] {
        include_bytes!("shoppingcart.desc")
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
