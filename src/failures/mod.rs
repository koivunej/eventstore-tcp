//! This module contains the non-success containing result enumerations for ones found in the
//! protocol description.

mod operation;
pub use self::operation::OperationFailure;

mod read_event;
pub use self::read_event::ReadEventFailure;

mod read_stream;
pub use self::read_stream::ReadStreamFailure;
