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

#[derive(Debug)]
enum ReadMode {
    ForwardOnce{ skip: usize, count: usize },
    Backward{ skip: usize, count: usize }
}

impl ReadMode {
    fn into_request(self, stream_id: &str) -> Message {
        use ReadMode::*;
        match self {
            ForwardOnce { skip, count } => {
                if count != 1 {
                    unimplemented!();
                }
                Builder::read_event()
                    .stream_id(stream_id.to_owned())
                    .event_number(StreamVersion::from_opt(skip as u32).expect("Stream version overflow"))
                    .resolve_link_tos(true)
                    .require_master(false)
                    .build_message()
            },
            Backward { .. } => unimplemented!()
        }
    }
}

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
        .subcommand(SubCommand::with_name("read")
                        .about("Read event(s) of a stream")
                        .arg(Arg::with_name("stream_id")
                                .value_name("STREAM-ID")
                                .required(true)
                                .index(1)
                                .help("Stream id to read from, or $all for every stream"))
                        .arg(Arg::with_name("count")
                             .value_name("N")
                             .short("c")
                             .long("count")
                             .takes_value(true)
                             .help("Number of events to read, 'all' or N > 0, defaults to 1"))
                        .arg(Arg::with_name("skip")
                             .value_name("SKIP")
                             .short("s")
                             .long("skip")
                             .takes_value(true)
                             .help("The event number to start from, defaults to 0"))
                        .arg(Arg::with_name("mode")
                             .value_name("MODE")
                             .short("m")
                             .long("mode")
                             .takes_value(true)
                             .possible_values(&["forward-forever", "forward-once", "backward"])
                             .help("'forward-once' reads up until the current latest,
'forward-forever' stays and awaits for new messages until count has been reached,
'backward' goes to up to first event"))
                        .arg(Arg::with_name("output")
                             .value_name("OUTPUT_MODE")
                             .short("o")
                             .long("output")
                             .takes_value(true)
                             .possible_values(&["debug", "utf8_lossy", "json_oneline", "hex"])
                             .help("'debug' will print the structure with {:?} or {:#?} depending on verbose
'utf8_lossy' will attempt to decode data as utf8
'json_oneline' will parse the json and stringify on one line
'hex' will write data as hexadecimals")))
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
    } else if let Some(r) = matches.subcommand_matches("read") {
        let stream_id = r.value_of("stream_id").unwrap();
        let count: usize = match r.value_of("count").unwrap_or("1") {
            "all" => usize::max_value(),
            s => s.parse().expect("Parsing count failed"),
        };

        let skip: usize = r.value_of("skip").unwrap_or("0").parse().expect("Parsing skip failed");
        let mode = match r.value_of("mode") {
            None | Some("forward-once") => ReadMode::ForwardOnce{skip, count},
            Some("forward-forever") => unimplemented!(),
            Some("backward") => ReadMode::Backward{skip, count},
            _ => unreachable!(),
        };

        read(addr, verbose, stream_id, mode)
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
                Err(io::Error::new(io::ErrorKind::Other, format!("{}", reason)))
            },
            x => {
                Err(io::Error::new(io::ErrorKind::Other, format!("Unexpected response: {:?}", x)))
            }
        }
    });

    core.run(job)
}

fn read(addr: SocketAddr, verbose: bool, stream_id: &str, mode: ReadMode) -> Result<(), io::Error> {
    if stream_id == "$all" {
        unimplemented!();
    }

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = EventStoreClient::connect(&addr, &handle);
    let started = Instant::now();

    let job = client.and_then(|client| {
        client.call(Package {
            authentication: None,
            correlation_id: Uuid::new_v4(),
            message: mode.into_request(stream_id),
        })
    }).and_then(|resp| {
        println!("{:#?}", resp);
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
