extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate uuid;
extern crate es_proto;

use std::net::SocketAddr;
use uuid::Uuid;

use futures::Future;
use tokio_core::reactor::Core;
use tokio_service::Service;

use es_proto::{EventStoreClient, Package, Message};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:1113".parse::<SocketAddr>().unwrap();

    let client = EventStoreClient::connect(&addr, &handle);

    let job = client.and_then(|client| {
        let corr = Uuid::new_v4();
        let pkg = Package { authentication: None, correlation_id: corr, message: Message::HeartbeatRequest };
        println!("sending: {:#?}", pkg);
        client.call(pkg)
    }).and_then(|rsp| {
        println!("got back: {:#?}", rsp);
        Ok(())
    });

    core.run(job).unwrap();
}

