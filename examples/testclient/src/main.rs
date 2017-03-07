extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate uuid;

#[macro_use]
extern crate clap;
extern crate json;
extern crate eventstore_tcp;

use std::io;
use std::env;
use std::net::SocketAddr;
use std::process;
use std::str;
use std::time::Duration;

use futures::Future;
use tokio_core::reactor::Core;
use tokio_service::Service;

use clap::{Arg, App, SubCommand, ArgMatches};

use eventstore_tcp::{EventStoreClient, Package, AdaptedMessage, Builder, ExpectedVersion, StreamVersion, EventNumber, ContentType, ReadDirection, ReadStreamCompleted, EventRecord, LogPosition, UsernamePassword, ReadAllCompleted, ReadEventError, ReadStreamError, ReadAllError};

#[derive(Debug)]
enum ByteParsingError {
    Utf8(str::Utf8Error),
    Json(json::Error)
}

impl From<str::Utf8Error> for ByteParsingError {
    fn from(e: str::Utf8Error) -> Self { ByteParsingError::Utf8(e) }
}

impl From<json::Error> for ByteParsingError {
    fn from(e: json::Error) -> Self { ByteParsingError::Json(e) }
}

struct JsonWrapper<'a, 'b: 'a>(&'a EventRecord<'b>);

impl<'a, 'b: 'a> JsonWrapper<'b, 'a> {
    fn parse_bytes(bytes: &'a [u8]) -> Result<json::JsonValue, ByteParsingError> {
        str::from_utf8(bytes)
            .map_err(ByteParsingError::from)
            .and_then(|s| if s.len() > 0 { Ok(json::parse(s)?) } else { Ok(json::JsonValue::new_object()) })
    }

    fn as_json(&self) -> Result<json::JsonValue, ByteParsingError> {
        let ref event = self.0;

        let (data, metadata) = Self::parse_bytes(&*event.data)
            .and_then(|data|
                event.metadata.as_ref()
                  .map(|x| Self::parse_bytes(x))
                  .unwrap_or(Ok("".to_string().into()))
                  .map(move |metadata| (data, metadata)))?;

        let mut obj = json::object::Object::new();
        obj.insert("data", data);
        obj.insert("metadata", metadata);

        Ok(obj.into())
    }
}

#[derive(Debug)]
enum ReadFailure<'a> {
    Event(ReadEventError<'a>),
    Stream(ReadStreamError<'a>),
    All(ReadAllError<'a>)
}

impl<'a> From<ReadEventError<'a>> for ReadFailure<'a> { fn from(e: ReadEventError<'a>) -> Self { ReadFailure::Event(e) } }
impl<'a> From<ReadStreamError<'a>> for ReadFailure<'a> { fn from(e: ReadStreamError<'a>) -> Self { ReadFailure::Stream(e) } }
impl<'a> From<ReadAllError<'a>> for ReadFailure<'a> { fn from(e: ReadAllError<'a>) -> Self { ReadFailure::All(e) } }

#[derive(Debug, Clone)]
enum Position {
    First,
    Exact(StreamVersion),
    Log(LogPosition, LogPosition),
    Last
}

fn env_credentials() -> Option<UsernamePassword> {
    let username = env::var("ES_USERNAME");
    let password = env::var("ES_PASSWORD");

    match (username, password) {
        (Ok(u), Ok(p)) => Some(UsernamePassword::new(u, p)),
        _ => None
    }
}

impl Position {
    fn into_log_position(self) -> (LogPosition, LogPosition) {
        match self {
            Position::Log(a, b) => (a, b),
            Position::First => (LogPosition::First, LogPosition::First),
            Position::Last => (LogPosition::Last, LogPosition::Last),
            Position::Exact(v) => panic!("Invalid position for $all, it needs to be u64,u64, first or last, not: {:?}", v)
        }
    }

    fn into_event_number(self) -> EventNumber {
        match self {
            Position::First => EventNumber::First,
            Position::Exact(x) => EventNumber::Exact(x),
            Position::Last => EventNumber::Last,
            Position::Log(commit, prepare) => panic!("Invalid position for non-$all stream read: {:?}", (commit, prepare)),
        }
    }
}

impl<'a> From<&'a str> for Position {
    fn from(s: &'a str) -> Self {
        match s {
            "first" => Position::First,
            "last" => Position::Last,
            x => {
                let mut parts = s.split(",");
                match (parts.next(), parts.next()) {
                    (Some(a), Some(b)) => {
                        let commit = LogPosition::from(a.parse::<i64>().unwrap());
                        let prepare = LogPosition::from(b.parse::<i64>().unwrap());

                        Position::Log(commit, prepare)
                    },
                    _ => {
                        Position::Exact(
                            StreamVersion::from_opt(
                                x.parse::<u32>().expect("Failed to parse position as u32"))
                            .expect("Position overflow"))
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum ReadMode {
    ForwardOnce { position: Position, count: u8 },
    Backward { position: Position, count: u8 }
}

#[derive(Debug, Clone)]
enum OutputMode {
    Debug,
    JsonOneline,
    Hex
}

impl Copy for OutputMode {}

impl<'a> From<&'a str> for OutputMode {
    fn from(s: &'a str) -> OutputMode {
        match s {
            "debug" => OutputMode::Debug,
            "json_oneline" => OutputMode::JsonOneline,
            "hex" => OutputMode::Hex,
            _ => panic!("Unsupport mode: {}", s)
        }
    }
}

impl OutputMode {

    fn format_event<'a, Out: io::Write, ErrOut: io::Write>(&self, verbose: bool, event: EventRecord<'a>, out: &mut Out, err: &mut ErrOut) -> io::Result<()> {
        match *self {
            OutputMode::Debug => {
                if verbose {
                    writeln!(out, "{:#?}", event)
                } else {
                    writeln!(out, "{:?}", event)
                }
            },
            OutputMode::JsonOneline => {
                match JsonWrapper(&event).as_json() {
                    Ok(obj) => {
                        obj.write(out)?;
                        writeln!(out, "")?;
                    },
                    Err(fail) => {
                        writeln!(
                            err,
                            "Failed to parse event {}@{}: {:?}",
                            event.event_stream_id,
                            event.event_number,
                            fail)?
                    }
                }
                Ok(())
            },
            OutputMode::Hex => {
                for b in event.data.iter() {
                    write!(out, "{:02x}", b)?;
                }

                if let Some(meta) = event.metadata {
                    write!(out, " ")?;
                    for b in meta.iter() {
                        write!(out, "{:02x}", b)?;
                    }
                }

                writeln!(out, "")?;
                Ok(())
            }
        }
    }

    fn format_events<'a, Out: io::Write, ErrOut: io::Write>(&self, verbose: bool, events: Vec<EventRecord<'a>>, out: &mut Out, err: &mut ErrOut) -> io::Result<()> {
        for event in events {
            self.format_event(verbose, event, out, err)?;
        }
        Ok(())
    }

    fn format_fail<'a, Out: io::Write, ErrOut: io::Write>(&self, verbose: bool, fail: ReadFailure<'a>, _: &mut Out, err: &mut ErrOut) -> io::Result<()> {
        if verbose {
            writeln!(err, "{}: {:#?}", "Read failed", fail)
        } else {
            writeln!(err, "{}: {:?}", "Read failed", fail)
        }
    }

    fn format<'a, Out: io::Write, ErrOut: io::Write>(&self, verbose: bool, msg: AdaptedMessage<'a>, out: &mut Out, err: &mut ErrOut) -> io::Result<()> {

        match msg {
            AdaptedMessage::ReadEventCompleted(Ok(rie)) => {
                self.format_events(verbose, vec![rie].into_iter().map(|x| x.event).collect(), out, err)
            }
            AdaptedMessage::ReadStreamEventsCompleted(_, Ok(ReadStreamCompleted { events, .. })) => {
                self.format_events(verbose, events.into_iter().map(|x| x.event).collect(), out, err)
            }
            AdaptedMessage::ReadAllEventsCompleted(_, Ok(ReadAllCompleted { events, .. })) => {
                self.format_events(verbose, events.into_iter().map(|x| x.event).collect(), out, err)
            }
            AdaptedMessage::ReadEventCompleted(Err(fail)) => {
                self.format_fail(verbose, fail.into(), out, err)
            }
            AdaptedMessage::ReadStreamEventsCompleted(_, Err(fail)) => {
                self.format_fail(verbose, fail.into(), out, err)
            }
            AdaptedMessage::ReadAllEventsCompleted(_, Err(fail)) => {
                self.format_fail(verbose, fail.into(), out, err)
            }
            x => {
                if verbose {
                    writeln!(err, "Unexpected message received: {:#?}", x)
                } else {
                    writeln!(err, "Unexpected message received: {:?}", x)
                }
            }
        }
    }
}

impl ReadMode {
    fn into_request(self, stream_id: &str) -> Package {
        use ReadMode::*;

        let dir = match self {
            ForwardOnce { .. } => ReadDirection::Forward,
            Backward { .. } => ReadDirection::Backward,
        };

        match self {
            ForwardOnce { position, count: 1 } | Backward { position, count: 1 } => {
                Builder::read_event()
                    .stream_id(stream_id.to_owned())
                    .event_number(position.into_event_number())
                    .resolve_link_tos(true)
                    .require_master(false)
                    .build_package(env_credentials(), None)
            },
            ForwardOnce { position, count } | Backward { position, count } => {
                if stream_id == "$all" {
                    let (commit, prepare) = position.into_log_position();
                    Builder::read_all_events()
                        .direction(dir)
                        .positions(commit, prepare)
                        .max_count(count)
                        .build_package(env_credentials(), None)
                } else {
                    Builder::read_stream_events()
                        .direction(dir)
                        .stream_id(stream_id.to_owned())
                        .from_event_number(position.into_event_number())
                        .max_count(count)
                        .build_package(env_credentials(), None)
                }
            },
        }
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!("\n"))
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
                        .arg(Arg::with_name("position")
                             .value_name("POS")
                             .short("p")
                             .long("position")
                             .takes_value(true)
                             .help("The event number to start from, first, last or N > 0, defaults to 0"))
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
                             .possible_values(&["debug", "json_oneline", "hex"])
                             .help("'debug' will print the structure with {:?} or {:#?} depending on verbose
'json_oneline' will parse the json and stringify on one line
'hex' will write data as hexadecimals")))
        .get_matches();

    let addr = {
        let s = format!("{}:{}",
                        matches.value_of("hostname").unwrap_or("127.0.0.1"),
                        matches.value_of("port").unwrap_or("1113"));
        s.parse::<SocketAddr>()
            .expect("Failed to parse host:port as SocketAddr. Hostname resolution is not yet implemented.")
    };

    let verbose = matches.is_present("verbose");

    let res = if let Some(_) = matches.subcommand_matches("ping") {
        ping(addr, verbose)
    } else if let Some(w) = matches.subcommand_matches("write") {
        write(addr, verbose, prepare_write(w))
    } else if let Some(r) = matches.subcommand_matches("read") {
        let stream_id = r.value_of("stream_id").unwrap();
        let count = match r.value_of("count").unwrap_or("1") {
            "all" => None,
            s => Some(s.parse().expect("Parsing count failed")),
        };

        let position: Position = r.value_of("position").unwrap_or("first").into();
        let mode = match r.value_of("mode") {
            None | Some("forward-once") =>
                ReadMode::ForwardOnce{ position, count: count.unwrap_or(10) },
            Some("backward") =>
                ReadMode::Backward{ position, count: count.unwrap_or(10) },
            Some("forward-forever") => unimplemented!(),
            _ => unreachable!(),
        };

        let output = r.value_of("output").map(OutputMode::from).unwrap_or(OutputMode::Debug);

        read(addr, verbose, output, stream_id, mode)
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

            let f = client.call(Builder::ping().build_package(env_credentials(), None));
            let send_began = Instant::now();

            f.map(move |resp| (resp, connected, send_began))
        }).and_then(|(pong, connected, send_began)| {
            let received = Instant::now();
            match pong.message.try_adapt().unwrap() {
                AdaptedMessage::Pong => Ok((connected, send_began, received)),
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

fn prepare_write<'a>(args: &ArgMatches<'a>) -> Package {
    let content_type = if args.is_present("json") { ContentType::Json } else { ContentType::Bytes };
    let mut builder = Builder::write_events();
    builder.stream_id(args.value_of("stream_id").unwrap().to_owned())
        .expected_version(match args.value_of("expected_version").unwrap() {
            "any" => ExpectedVersion::Any,
            "created" => ExpectedVersion::NewStream,
            n => ExpectedVersion::Exact(StreamVersion::from(n.parse().unwrap()))
        })
        .require_master(args.is_present("require_master"));

    {
        // some weaknesses with the moving methods
        let mut event = builder.new_event();

        event = event.data(args.value_of("data").unwrap().as_bytes().iter().cloned().collect::<Vec<_>>())
            .data_content_type(content_type)
            .event_type(args.value_of("type").unwrap().to_owned());

        event = if let Some(x) = args.value_of("metadata") {
            event.metadata(x.as_bytes().iter().cloned().collect::<Vec<_>>())
                .metadata_content_type(content_type)
        } else {
            event
        };

        event.done();
    }

    builder.build_package(env_credentials(), None)
}

fn write(addr: SocketAddr, verbose: bool, pkg: Package) -> Result<(), io::Error> {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = EventStoreClient::connect(&addr, &handle);
    let started = Instant::now();

    let job = client.and_then(|client| {
        client.call(pkg)
    }).and_then(|resp| {
        match resp.message.try_adapt().unwrap() {
            AdaptedMessage::WriteEventsCompleted(Ok(success)) => {
                print_elapsed("Success in", started.elapsed());
                if verbose {
                    println!("{:#?}", success);
                }
                Ok(())
            },
            AdaptedMessage::WriteEventsCompleted(Err(reason)) => {
                Err(io::Error::new(io::ErrorKind::Other, format!("{}", reason)))
            },
            x => {
                Err(io::Error::new(io::ErrorKind::Other, format!("Unexpected response: {:?}", x)))
            }
        }
    });

    core.run(job)
}

fn read(addr: SocketAddr, verbose: bool, output: OutputMode, stream_id: &str, mode: ReadMode) -> Result<(), io::Error> {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = EventStoreClient::connect(&addr, &handle);
    let job = client.and_then(|client| {
        client.call(mode.into_request(stream_id))
    }).and_then(|resp| {
        let mut stdout = io::stdout();
        let mut stderr = io::stderr();
        output.format(verbose, resp.message.try_adapt().unwrap(), &mut stdout, &mut stderr)
    });

    core.run(job)
}

fn print_elapsed(subject: &str, d: Duration) {
    println!("{} {}.{:06}ms", subject, d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000, d.subsec_nanos() % 1_000_000);
}
