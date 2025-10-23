use crate::{
    impl_from_str_for_parsable,
    parser::{self, Parser, ParserExt},
    Move, PlayerFactory, ShortMove,
};
use std::{
    fmt::{self, Display, Formatter},
    time::Duration,
};

#[derive(Debug, Clone)]
pub enum CliCommand {
    TimeLimit(Duration),
    Opening(Vec<Move>),
    Start,
    OpponentMove(ShortMove),
    Quit,
}

impl CliCommand {
    pub fn parser() -> impl Parser<Output = Self> {
        parser::exact(b"Time ")
            .ignore_then(parser::u32())
            .map(|ms| CliCommand::TimeLimit(Duration::from_millis(ms.into())))
            .or(parser::exact(b"Opening")
                .ignore_then(parser::exact(b" ").ignore_then(Move::parser()).repeat(0..))
                .map(CliCommand::Opening))
            .or(parser::exact(b"Start").map(|_| CliCommand::Start))
            .or(parser::exact(b"Quit").map(|_| CliCommand::Quit))
            .or(ShortMove::parser().map(CliCommand::OpponentMove))
    }
}

impl_from_str_for_parsable!(CliCommand);

impl Display for CliCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CliCommand::TimeLimit(duration) => write!(f, "Time {}", duration.as_millis())?,
            CliCommand::Opening(moves) => {
                write!(f, "Opening")?;
                for mov in moves {
                    write!(f, " {mov}")?;
                }
            }
            CliCommand::Start => write!(f, "Start")?,
            CliCommand::OpponentMove(mov) => write!(f, "{mov}")?,
            CliCommand::Quit => write!(f, "Quit")?,
        }
        Ok(())
    }
}

pub fn cli(_player_factory: &dyn PlayerFactory) {}
