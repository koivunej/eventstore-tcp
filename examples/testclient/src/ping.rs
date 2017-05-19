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

        let verbose = config.verbose;
        let started = self.started.unwrap();
        let connected = Instant::now();
        let ping = client.call(Builder::ping().build_package(config.credentials.clone(), None));
        let send_began = Instant::now();

        ping.map(move |resp| (resp, started, connected, send_began))
            .and_then(move |(pong, started, connected, send_began)| {
                let received = Instant::now();
                match pong.message.try_adapt().unwrap() {
                    AdaptedMessage::Pong => Ok((started, connected, send_began, received)),
                    msg => Err(io::Error::new(io::ErrorKind::Other, format!("Unexpected response: {:?}", msg)))
                }
            }).and_then(move |(started, connected, send_began, received)| {
                if verbose {
                    print_elapsed("connected in  ", connected - started);
                    print_elapsed("ready to send ", send_began - connected);
                    print_elapsed("received in   ", received - send_began);
                    print_elapsed("total         ", started.elapsed());
                } else {
                    print_elapsed("pong received in", started.elapsed());
                }
                Ok(())
            }).boxed()
    }
}

