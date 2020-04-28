pub mod protocol {
    pub mod cloudstate {
        include!("protocol/cloudstate.rs");
        pub mod eventsourced {
            include!("protocol/cloudstate.eventsourced.rs");
        }
    }
}

pub mod example {
    pub mod shoppingcart {
        include!("example/shoppingcart/com.example.shoppingcart.rs");
        pub mod persistence {
            include!("example/shoppingcart/com.example.shoppingcart.persistence.rs");
        }
    }
}
