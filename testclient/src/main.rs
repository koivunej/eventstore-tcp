extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate uuid;

#[macro_use]
extern crate clap;
extern crate eventstore_tcp;

extern crate testclient;

use std::convert::TryFrom;
use std::env;
use std::net::SocketAddr;
use std::process;
use std::str;

use clap::{Arg, App, SubCommand, ArgMatches};

use eventstore_tcp::{Builder, ExpectedVersion, StreamVersion, ContentType, UsernamePassword};
use testclient::{Config, Runner, Ping, Write, Read, Delete};

fn env_credentials() -> Option<UsernamePassword> {
    let username = env::var("ES_USERNAME");
    let password = env::var("ES_PASSWORD");

    match (username, password) {
        (Ok(u), Ok(p)) => Some(UsernamePassword::new(u, p)),
        _ => None
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
        .subcommand(SubCommand::with_name("delete")
                        .about("Delete a stream")
                        .arg(Arg::with_name("stream_id")
                                .value_name("STREAM-ID")
                                .required(true)
                                .index(1)
                                .help("Stream id to write to"))
                        .arg(Arg::with_name("expected_version")
                                .value_name("EXPECTED_VERSION")
                                .required(true)
                                .index(2)
                                .help("Acceptable values: any|created|n where n >= 0"))
                        .arg(Arg::with_name("hard_delete")
                                .takes_value(false)
                                .long("hard-delete")
                                .help("Schedules the stream events to be deleted instead of soft deleting"))
                        .arg(Arg::with_name("require_master")
                                .takes_value(false)
                                .short("m")
                                .long("require-master")
                                .help("Disables intra-cluster request forwarding")))
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
                                .help("Acceptable values: any|created|n where n >= 0"))
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
                                .help("Flags the data and metadata (when given) as json values"))
                        .arg(Arg::with_name("require_master")
                                .takes_value(false)
                                .short("m")
                                .long("require-master")
                                .help("Disables intra-cluster request forwarding")))
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

    let config = Config {
        addr: addr,
        credentials: env_credentials(),
        verbose: matches.is_present("verbose"),
    };

    let runner = Runner::new(config);

    let res = if let Some(_) = matches.subcommand_matches("ping") {
        runner.run(Ping::new())
    } else if let Some(w) = matches.subcommand_matches("write") {
        let write = prepare_write(w);
        runner.run(write)
    } else if let Some(r) = matches.subcommand_matches("read") {
        let read = prepare_read(r);
        runner.run(read)
    } else if let Some(d) = matches.subcommand_matches("delete") {
        let delete = prepare_delete(d);
        runner.run(delete)
    } else {
        println!("Subcommand is required.\n\n{}", matches.usage());
        process::exit(1);
    };

    if let Err(e) = res {
        println!("Failure: {:?}", e);
        process::exit(1);
    }
}

fn prepare_write<'a>(args: &ArgMatches<'a>) -> Write {
    let content_type = if args.is_present("json") { ContentType::Json } else { ContentType::Bytes };
    let mut builder = Builder::write_events();
    builder.stream_id(args.value_of("stream_id").unwrap().to_owned())
        .expected_version(match args.value_of("expected_version").unwrap() {
            "any" => ExpectedVersion::Any,
            "created" => ExpectedVersion::NoStream,
            n => ExpectedVersion::Exact(StreamVersion::try_from(n.parse::<u32>().unwrap()).unwrap())
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

    Write::new(builder)
}

fn prepare_read<'a>(r: &ArgMatches<'a>) -> Read {
    use testclient::read::{ReadMode, OutputMode, Position};
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

    Read::new(stream_id.to_owned(), mode, output)
}

fn prepare_delete<'a>(d: &ArgMatches<'a>) -> Delete {
    let mut builder = Builder::delete_stream();
    builder.stream_id(d.value_of("stream_id").unwrap().to_string())
        .expected_version(match d.value_of("expected_version").unwrap() {
            "any" => ExpectedVersion::Any,
            "created" => ExpectedVersion::NoStream,
            n => ExpectedVersion::Exact(StreamVersion::try_from(n.parse::<u32>().unwrap()).unwrap())
        })
        .hard_delete(d.is_present("hard_delete"))
        .require_master(d.is_present("require_master"));

    Delete::new(builder)
}
