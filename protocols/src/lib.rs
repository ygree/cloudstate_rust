pub mod cloudstate {
    tonic::include_proto!("cloudstate");
    pub mod eventsourced {
        tonic::include_proto!("cloudstate.eventsourced");
    }
}

pub mod shoppingcart {
    pub mod persistence {
        tonic::include_proto!("com.example.shoppingcart.persistence");
    }
}
