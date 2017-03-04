//! This module contains the non-success containing result enumerations for ones found in the
//! protocol description.

mod write_events;
pub use self::write_events::WriteEventsFailure;

mod read_event;
pub use self::read_event::ReadEventFailure;

mod read_stream;
pub use self::read_stream::ReadStreamFailure;

mod read_all;
pub use self::read_all::ReadAllFailure;
