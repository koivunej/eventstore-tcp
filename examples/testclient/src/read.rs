use std::io;
use std::str;
use std::cell::RefCell;
use futures::Future;
use tokio_service::Service;
use json;
use eventstore_tcp::{EventStoreClient, Builder, ReadDirection, UsernamePassword, Package, AdaptedMessage, EventNumber, LogPosition, StreamVersion};
use eventstore_tcp::adapted::{ReadEventError, ReadAllError, ReadAllCompleted, ReadStreamError, ReadStreamCompleted};
use eventstore_tcp::raw::{EventRecord};
use {Config, Command};

#[derive(Debug, Clone)]
pub enum Position {
    First,
    Exact(StreamVersion),
    Log(LogPosition, LogPosition),
    Last
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
pub enum ReadMode {
    ForwardOnce { position: Position, count: u8 },
    Backward { position: Position, count: u8 }
}

#[derive(Debug, Clone)]
pub enum OutputMode {
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

pub struct Read {
    stream_id: String,
    read_mode: RefCell<Option<ReadMode>>,
    output_mode: RefCell<Option<OutputMode>>
}

impl Read {
    pub fn new(stream_id: String, read_mode: ReadMode, output_mode: OutputMode) -> Self {
        Read { stream_id, read_mode: RefCell::new(Some(read_mode)), output_mode: RefCell::new(Some(output_mode)) }
    }
}

impl Command for Read {
    fn init(&mut self) { }

    fn execute(&self, config: &Config, client: EventStoreClient) -> Box<Future<Item = (), Error = io::Error>> {
        let verbose = config.verbose;
        let output = self.output_mode.borrow_mut().take().unwrap();

        let package = self.read_mode.borrow_mut().take().unwrap().into_request(&self.stream_id, config.credentials.clone());
        let send = client.call(package);

        send.and_then(move |resp| {
            let mut stdout = io::stdout();
            let mut stderr = io::stderr();
            output.format(verbose, resp.message.try_adapt().unwrap(), &mut stdout, &mut stderr)
        }).boxed()
    }
}

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
    fn into_request(self, stream_id: &str, cred: Option<UsernamePassword>) -> Package {
        use self::ReadMode::*;

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
                    .build_package(cred, None)
            },
            ForwardOnce { position, count } | Backward { position, count } => {
                if stream_id == "$all" {
                    let (commit, prepare) = position.into_log_position();
                    Builder::read_all_events()
                        .direction(dir)
                        .positions(commit, prepare)
                        .max_count(count)
                        .build_package(cred, None)
                } else {
                    Builder::read_stream_events()
                        .direction(dir)
                        .stream_id(stream_id.to_owned())
                        .from_event_number(position.into_event_number())
                        .max_count(count)
                        .build_package(cred, None)
                }
            },
        }
    }
}

