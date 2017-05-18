/// The direction in which events are read.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReadDirection {
    /// Read from first (event 0) to the latest
    Forward,
    /// Read from latest (highest event number) to the first (event 0)
    Backward
}
