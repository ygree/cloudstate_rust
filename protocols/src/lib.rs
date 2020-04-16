pub mod cloudstate {
    tonic::include_proto!("cloudstate");
    pub mod eventsourced {
        tonic::include_proto!("cloudstate.eventsourced");
    }
}
