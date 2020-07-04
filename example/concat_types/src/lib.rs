#[derive(Clone, Debug, PartialEq)]
pub enum ConcatCommand {
    Exit,
    Clear,
    Concat(String),
    Print,
    Time(std::time::Instant)
}