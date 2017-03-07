initSidebarItems({"enum":[["ContentType","Content type of the event `data` or `metadata`."],["EventNumber","`EventNumber` is similar to `StreamVersion` and `ExpectedVersion` but is used when specifying a position to read from in the stream. Allows specifying the first or last (when reading backwards) event in addition to exact event number."],["ExpectedVersion","`ExpectedVersion` represents the different modes of optimistic locking when writing to a stream using `WriteEventsBuilder`."],["LogPosition","Global unique position in the EventStore, used when reading all events. Range -1..i64::max_value()"],["NotHandledReason",""],["ReadAllError","Failure cases of wire enum `ReadAllResult`."],["ReadDirection","The direction in which events are read."],["ReadEventError","`ReadEventError` maps to non-success of `ReadEventResult`"],["ReadStreamError","Non-success projection of the `ReadStreamResult` enum on the wire representing a failed `ReadStreamCompleted` request."],["WriteEventsFailure","Like `OperationResult` on the wire but does not have a success value. Explains the reason for failure."]],"mod":[["adapted","Adapted or refined types providing a much more oxidized API for handling the messages in the protocol."],["builder","Builders to help building requests."],["raw","Raw module contains the enumeration `RawMessage` and raw decoding and encoding functionality. There should not be need to handle `RawMessage` values directly but if there is ever a bug, using the raw messages should still work."]],"struct":[["EventRecord",""],["EventStoreClient","`tokio_service::Service` implementation of the client."],["MasterInfo",""],["Package","Frame in the protocol. On the wire, packets are embedded in frames with length prefix and suffix."],["ReadAllCompleted","Successful response to `Message::ReadAllEvents`."],["ReadAllEvents",""],["ReadStreamCompleted","Successful response to a `Message::ReadStreamEvents`."],["ResolvedIndexedEvent",""],["StreamVersion","`StreamVersion` represents the valid values for a stream version which is the same as the event number of the latest event. As such, values are non-negative integers up to `i32::max_value`. Negative values of `i32` have special meaning in the protocol, and are restricted from being used with this type."],["UsernamePassword","Username and password authentication token embedded in requests as there is no concept of session in the TCP protocol, every request must be authenticated."],["WriteEvents",""],["WriteEventsCompleted","Successful response to `Message::WriteEvents`"]]});