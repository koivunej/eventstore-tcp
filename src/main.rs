extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate uuid;
extern crate clap;
extern crate es_proto;

use std::io;
use std::net::SocketAddr;
use std::thread;
use std::process;
use uuid::Uuid;

use futures::{Future, IntoFuture, Stream, Sink};
use futures::future::{BoxFuture, Either};
use tokio_core::reactor::{Core, Handle};
use tokio_service::Service;
use futures::sync::{oneshot, mpsc};

use clap::{Arg, App, SubCommand};

use es_proto::{EventStoreClient, Package, Message, Builder, ExpectedVersion, StreamVersion, UsernamePassword};

fn main() {

    let matches = App::new("test-client")
        .version("0.1.0")
        .about("Test client similar to EventStore.TestClient in EventStore binary distribution")
        .arg(Arg::with_name("hostname")
                .short("h")
                .long("host")
                .value_name("HOST")
                .takes_value(true)
                .help("The name of the host to connect to, default: 127.0.0.1"))
        .arg(Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .takes_value(true)
                .help("The port to connect to, default: 1113"))
        .subcommand(SubCommand::with_name("ping")
                        .about("Send a ping to the host after possibly authenticating depending on the options"))
        .subcommand(SubCommand::with_name("write")
                        .about("Write single event to a given stream")
                        .arg(Arg::with_name("stream_id")
                                .value_name("STREAM-ID")
                                .required(true)
                                .index(1))
                        .arg(Arg::with_name("expected_version")
                                .value_name("EXPECTED_VERSION")
                                .required(true)
                                .index(2)
                                .help("Acceptable values: any|created|n where n >= -2"))
                        .arg(Arg::with_name("data")
                                .value_name("DATA")
                                .required(true)
                                .index(3)
                                .help("Raw bytes of data."))
                        .arg(Arg::with_name("metadata")
                                .value_name("METADATA")
                                .required(false)
                                .index(4)
                                .help("Raw bytes of metadata, optional."))
                        .arg(Arg::with_name("json")
                                .short("j")
                                .long("json")
                                .takes_value(false)
                                .help("Flags the data and metadata (when given) as json values"))
                        .arg(Arg::with_name("type")
                                .short("t")
                                .long("type")
                                .value_name("TYPE")
                                .takes_value(true)
                                .help("Marks the data as being of given type, required if data is json")))
        .get_matches();

    let addr = {
        let s = format!("{}:{}", matches.value_of("HOST").unwrap_or("127.0.0.1"), matches.value_of("PORT").unwrap_or("1113"));
        s.parse::<SocketAddr>().expect("Failed to parse host:port as SocketAddr")
    };

    let res = if let Some(_) = matches.subcommand_matches("ping") {
        ping(addr)
    } else if let Some(write) = matches.subcommand_matches("write") {
        unimplemented!()
    } else {
        println!("Subcommand is required.\n\n{}", matches.usage());
        process::exit(1);
    };

    /*
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
    */
}

fn ping(addr: SocketAddr) -> Result<(), io::Error> {
    use std::time::Instant;

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let started = Instant::now();

    let job = EventStoreClient::connect(&addr, &handle)
        .map(|client| (client, Instant::now()))
        .and_then(|(client, connected)| {

            let f = client.call(Builder::ping().build_package(None, None));
            let send_began = Instant::now();

            f.map(move |resp| (resp, connected, send_began))
        }).and_then(|(pong, connected, send_began)| {
            let received = Instant::now();
            match pong.message {
                Message::Pong => Ok((connected, send_began, received)),
                msg => Err(io::Error::new(io::ErrorKind::Other, format!("Unexpected response: {:?}", msg)))
            }
        }).and_then(move |(connected, send_began, received)| {
            let conn_in = connected - started;
            let send_began_in = send_began - connected;
            let rx_in = received - send_began;

            print_elapsed("connected in  ", conn_in);
            print_elapsed("ready to send ", send_began_in);
            print_elapsed("received in   ", rx_in);
            print_elapsed("total         ", started.elapsed());

            Ok(())
        });

    core.run(job)
}

use std::time::Duration;

fn print_elapsed(subject: &str, d: Duration) {
    println!("{} {}.{:06}ms", subject, d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000, d.subsec_nanos() % 1_000_000);
}

/*
fn nice(addr: &SocketAddr, handle: &Handle) {
    let client = EventStoreClient::connect(&addr, &handle);
    client.and_then(|client| {

    });
}*/

/*
struct Client {
    tx: mpsc::Sender<(Package, oneshot::Sender<Result<Package, io::Error>>)>,
}

impl Client {
    fn new(addr: &SocketAddr, handle: &Handle) -> Client {
        let (tx, rx) = mpsc::channel::<(Package, oneshot::Sender<Result<Package, io::Error>>)>(4);
        let client = EventStoreClient::connect(addr, handle);
        let task = client.and_then(move |client| {
            rx.for_each(move |(pkg, tx)| {
                client.call(pkg).then(|res| {
                    let ret = match res {
                        Ok(_) => Ok(()),
                        Err(_) => Err(())
                    };
                    tx.complete(res);
                    ret
                })
            }).map_err(|_| io::Error::new(io::ErrorKind::Other, "dunno?"))
        });
        handle.spawn(task.then(|res| match res { Ok(_) => Ok(()), Err(_) => Err(()) }));
        Client { tx: tx }
    }

    fn call(&self, msg: Message) -> MessageFuture {
        let (tx, rx) = oneshot::channel();

        self.tx.send(());
    }
}*/

// connecting
// ready
// heartbeat
// disconnected
