use std::time::Instant;
use std::io;
use futures::Future;
use tokio_service::Service;
use eventstore_tcp::{EventStoreClient, Builder};
use {Config, Command, print_elapsed};

pub struct Ping {
    started: Option<Instant>,
}

impl Ping {
    pub fn new() -> Self {
        Ping { started: None }
    }
}

impl Command for Ping {
    fn init(&mut self) {
        self.started = Some(Instant::now());
    }

    fn execute(&self, config: &Config, client: EventStoreClient) -> Box<Future<Item = (), Error = io::Error>> {
        use eventstore_tcp::AdaptedMessage;

        let started = self.started.unwrap();
        let ping = client.call(Builder::ping().build_package(config.credentials.clone(), None));

        Box::new(ping.map(move |resp| (resp, started))
            .and_then(move |(pong, started)| {
                match pong.message.try_adapt().unwrap() {
                    AdaptedMessage::Pong => Ok(started),
                    msg => Err(io::Error::new(io::ErrorKind::Other, format!("Unexpected response: {:?}", msg)))
                }
            }).and_then(move |started| {
                print_elapsed("pong received in", started.elapsed());
                Ok(())
            }))
    }
}

