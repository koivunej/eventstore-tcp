extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate uuid;
extern crate es_proto;

use std::io;
use std::net::SocketAddr;
use std::thread;
use uuid::Uuid;

use futures::{Future, IntoFuture, Stream, Sink};
use futures::future::{BoxFuture, Either};
use tokio_core::reactor::Core;
use tokio_service::Service;
use futures::sync::{oneshot, mpsc};

use es_proto::{EventStoreClient, Package, Message, Builder, ExpectedVersion, StreamVersion, UsernamePassword};

fn main() {

    let addr = "127.0.0.1:1113".parse::<SocketAddr>().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = EventStoreClient::connect(&addr, &handle);

    let job = client.and_then(|client| {
        println!("connected!");
        let nak_auth = client.call(
            Builder::authenticate().build_package(Some(UsernamePassword::new("asdfasfdasf", "asdfasdffsa")), None)
        ).and_then(|resp| {
            assert_eq!(resp.message, Message::NotAuthenticated);
            Ok(())
        });

        let write_events = client.call(
            Builder::write_events()
                .stream_id("foobar")
                .expected_version(ExpectedVersion::Exact(StreamVersion::from_opt(3).unwrap()))
                .new_event()
                    .event_type("test_event")
                    .data(vec![0xaa, 0xbb, 0xcc, 0xdd])
                    .done()
                .new_event()
                    .event_type("test_other")
                    .data(b"{ \"json-example\": true }".into_iter().cloned().collect::<Vec<u8>>())
                    .done()
                .build_package(Some(UsernamePassword::new("admin", "changeit")), None))
            .and_then(|resp| {
                match resp.message {
                    Message::WriteEventsCompleted(Ok(ref x)) => println!("success: {:?}", x),
                    Message::WriteEventsCompleted(Err(ref x)) => println!("failure: {:?}", x),
                    Message::NotAuthenticated => println!("need to authenticate"),
                    y => println!("unexpected: {:?}", y)
                }
                Ok(())
            });

        nak_auth.join(write_events)
    });

    core.run(job).unwrap();
}
