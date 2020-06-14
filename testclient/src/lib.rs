extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate json;
extern crate eventstore_tcp;

use std::net::SocketAddr;
use std::io;
use std::time::Duration;
use futures::Future;
use tokio_core::reactor::Core;

use eventstore_tcp::{EventStoreClient, UsernamePassword};

mod ping;
pub use ping::Ping;

mod write;
pub use write::Write;

pub mod read;
pub use read::Read;

mod delete;
pub use delete::Delete;

pub struct Config {
    pub addr: SocketAddr,
    pub credentials: Option<UsernamePassword>,
    pub verbose: bool,
}

pub struct Runner(Config);

impl Runner {
    pub fn new(c: Config) -> Self {
        Runner(c)
    }

    pub fn run<C: Command>(&self, mut c: C) -> Result<(), io::Error> {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        c.init();

        let job = EventStoreClient::connect(&self.0.addr, &handle)
            .and_then(|client| c.execute(&self.0, client));

        core.run(job)
    }
}

pub trait Command: Send {
    fn init(&mut self);
    fn execute(&self, config: &Config, client: EventStoreClient) -> Box<dyn Future<Item = (), Error = io::Error>>;
}

fn print_elapsed(subject: &str, d: Duration) {
    println!("{} {}.{:06}ms", subject, d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000, d.subsec_nanos() % 1_000_000);
}
