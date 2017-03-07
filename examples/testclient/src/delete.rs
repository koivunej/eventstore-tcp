use std::io;
use std::cell::RefCell;
use futures::Future;
use tokio_service::Service;
use eventstore_tcp::EventStoreClient;
use eventstore_tcp::builder::DeleteStreamBuilder;
use {Config, Command};

pub struct Delete {
    builder: RefCell<Option<DeleteStreamBuilder>>,
}

impl Delete {
    pub fn new(builder: DeleteStreamBuilder) -> Self {
        Delete { builder: RefCell::new(Some(builder)), }
    }
}

impl Command for Delete {
    fn init(&mut self) { }

    fn execute(&self, config: &Config, client: EventStoreClient) -> Box<Future<Item = (), Error = io::Error>> {
        use eventstore_tcp::RawMessage;
        use eventstore_tcp::package::MessageContainer;
        use eventstore_tcp::raw::OperationResult;

        let package = self.builder.borrow_mut().take().unwrap().build_package(config.credentials.clone(), None);
        let verbose = config.verbose;
        let send = client.call(package);

        let ret = send.and_then(move |resp| {
            match resp.message {
                MessageContainer::Raw(RawMessage::DeleteStreamCompleted(body)) => {
                    if body.result.is_none() {
                        return Err(io::Error::new(io::ErrorKind::Other, format!("No result in response: {:#?}", body)));
                    }

                    match body.result.as_ref().unwrap() {
                        &OperationResult::Success => {
                            println!("Success");
                            if verbose {
                                println!("{:#?}", body);
                            }
                            Ok(())
                        },

                        other => {
                            Err(io::Error::new(io::ErrorKind::Other, format!("Deleting failed: {:?}", other)))
                        }
                    }
                },
                x => Err(io::Error::new(io::ErrorKind::Other, format!("Unexpected response: {:?}", x)))
            }
        });

        Box::new(ret)
    }
}
