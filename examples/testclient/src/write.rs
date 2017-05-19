use std::time::Instant;
use std::io;
use std::cell::RefCell;
use futures::Future;
use tokio_service::Service;
use eventstore_tcp::EventStoreClient;
use eventstore_tcp::builder::WriteEventsBuilder;
use {Config, Command, print_elapsed};

pub struct Write {
    builder: RefCell<Option<WriteEventsBuilder>>,
    started: Option<Instant>,
}

impl Write {
    pub fn new(builder: WriteEventsBuilder) -> Self {
        Write { builder: RefCell::new(Some(builder)), started: None }
    }
}

impl Command for Write {
    fn init(&mut self) { self.started = Some(Instant::now()); }

    fn execute(&self, config: &Config, client: EventStoreClient) -> Box<Future<Item = (), Error = io::Error>> {
        use eventstore_tcp::AdaptedMessage;

        let started = self.started.clone().unwrap();
        let package = self.builder.borrow_mut().take().unwrap().build_package(config.credentials.clone(), None);
        let verbose = config.verbose;
        let send = client.call(package);

        send.and_then(move |resp| {
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
        }).boxed()
    }
}
