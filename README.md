# Tokio-based EventStore client API written in Rust

[EventStore](https://geteventstore.com/) is an open-source event database.
This project aims to be a driver with similar features to the original .NET driver but at the moment it has the simplest operations implemented:

 * read an event
 * read a stream forward or backward
 * write events

## Unimplemented features in order of importance:

 1. read events from `$all` stream
 2. volatile subscriptions:
   * refactoring to use `tokio_proto::streaming::multiplex` instead of `tokio_proto::multiplex`
   * current messages are ok as headers, but appeared events could probably be body chunks
   * maintaining a subscription by pumping events to a `futures::sink::Sink`, detecting overflows and dropping the subscription
 3. Less of directly using the protobuf messages in the API
 4. Cleaning up the message builders
 5. Hide the use of `Package` from users
 6. Nice API which would not require users to run the `tokio_core::reactor::Core``

## "Perhaps later" features:

 1. persistent subscriptions (haven't researched this yet)
 2. competing consumers (?)
 3. long running transactions (???)

# Usage

Currently the best example on how to use the driver is the `examples/testclient` which aims to be a simple command line client for EventStore.

# Building

`cargo build` will handle building, and testclient becomes usable after building it in it's own directory: `cd examples/testclient && cargo run -- --help`.

# Testing

Integration tests in `tests/` currently download and run test-specific in-memory instance of the EventStore, and this all can take a while.

# Rebuilding the `client_messages`

 1. Obtain [ClientMessageDTOs.proto](https://github.com/EventStore/EventStore/blob/master/src/Protos/ClientAPI/ClientMessageDtos.proto) or (raw link)[https://raw.githubusercontent.com/EventStore/EventStore/master/src/Protos/ClientAPI/ClientMessageDtos.proto]
 2. Checkout latest `quick-protobuf`: `git clone https://github.com/tafia/quick-protobuf`
 3. `cd quick-protobuf/codegen`
 4. `cargo run $es_proto_checkout/ClientMessageDTOs.proto`
 5. `mv $es_proto_checkout/{ClientMessageDTOs,src/messages}.rs`

