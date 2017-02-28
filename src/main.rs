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

    let matches = App::new("testclient")
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
        .arg(Arg::with_name("verbose")
                 .short("v")
                 .long("verbose")
                 .takes_value(false)
                 .help("Output verbose timings"))
        .subcommand(SubCommand::with_name("ping")
                        .about("Send a ping to the host after possibly authenticating depending on the options"))
        .subcommand(SubCommand::with_name("write")
                        .about("Write single event to a given stream")
                        .arg(Arg::with_name("stream_id")
                                .value_name("STREAM-ID")
                                .required(true)
                                .index(1)
                                .help("Stream id to write to"))
                        .arg(Arg::with_name("expected_version")
                                .value_name("EXPECTED_VERSION")
                                .required(true)
                                .index(2)
                                .help("Acceptable values: any|created|n where n >= -2"))
                        .arg(Arg::with_name("type")
                                .value_name("TYPE")
                                .required(true)
                                .index(3)
                                .help("Type name of the event"))
                        .arg(Arg::with_name("data")
                                .value_name("DATA")
                                .required(true)
                                .index(4)
                                .help("Raw bytes of data."))
                        .arg(Arg::with_name("metadata")
                                .value_name("METADATA")
                                .required(false)
                                .index(5)
                                .help("Raw bytes of metadata, optional"))
                        .arg(Arg::with_name("json")
                                .short("j")
                                .long("json")
                                .takes_value(false)
                                .help("Flags the data and metadata (when given) as json values")))
        .get_matches();

    let addr = {
        let s = format!("{}:{}", matches.value_of("hostname").unwrap_or("127.0.0.1"), matches.value_of("port").unwrap_or("1113"));
        s.parse::<SocketAddr>().expect("Failed to parse host:port as SocketAddr")
    };

    let verbose = matches.is_present("verbose");

    let res = if let Some(matches) = matches.subcommand_matches("ping") {
        ping(addr, verbose)
    } else if let Some(w) = matches.subcommand_matches("write") {

        let mut builder = Builder::write_events();
        builder.stream_id(w.value_of("stream_id").unwrap().to_owned())
            .expected_version(match w.value_of("expected_version").unwrap() {
                "any" => ExpectedVersion::Any,
                "created" => ExpectedVersion::NewStream,
                n => ExpectedVersion::Exact(StreamVersion::from_opt(n.parse().unwrap()).expect("Stream version out of bounds"))
            })
            .require_master(w.is_present("require_master"));

        {
            let mut event = builder.new_event();

            event = event.data(w.value_of("data").unwrap().as_bytes().iter().cloned().collect::<Vec<_>>())
                .data_content_type(w.is_present("json"))
                .event_type(w.value_of("type").unwrap().to_owned());

            event = if let Some(x) = w.value_of("metadata") {
                event.metadata(x.as_bytes().iter().cloned().collect::<Vec<_>>())
                    .metadata_content_type(w.is_present("json"))
            } else {
                event
            };

            event.done();
        }

        write(addr, verbose, builder.build_package(None, None))
    } else {
        println!("Subcommand is required.\n\n{}", matches.usage());
        process::exit(1);
    };

    if let Err(e) = res {
        println!("Failure: {:?}", e);
        process::exit(1);
    }
}

use std::time::Instant;

fn ping(addr: SocketAddr, verbose: bool) -> Result<(), io::Error> {
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
            if verbose {
                print_elapsed("connected in  ", connected - started);
                print_elapsed("ready to send ", send_began - connected);
                print_elapsed("received in   ", received - send_began);
                print_elapsed("total         ", started.elapsed());
            } else {
                print_elapsed("pong received in", started.elapsed());
            }

            Ok(())
        });

    core.run(job)
}

fn write(addr: SocketAddr, verbose: bool, pkg: Package) -> Result<(), io::Error> {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = EventStoreClient::connect(&addr, &handle);
    let started = Instant::now();

    let job = client.and_then(|client| {
        client.call(pkg)
    }).and_then(|resp| {
        match resp.message {
            Message::WriteEventsCompleted(Ok(success)) => {
                print_elapsed("Success in", started.elapsed());
                if verbose {
                    println!("{:#?}", success);
                }
                Ok(())
            },
            Message::WriteEventsCompleted(Err(reason)) => {
                Err(io::Error::new(io::ErrorKind::Other, format!("Writing failed: {:?}", reason)))
            },
            x => {
                Err(io::Error::new(io::ErrorKind::Other, format!("Unexpected response: {:?}", x)))
            }
        }
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
