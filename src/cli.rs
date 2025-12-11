use crate::{
    clock::{Stopwatch, Timer},
    constants::DEFAULT_TIME_LIMIT,
    impl_from_str_for_parsable,
    log::{self, Level},
    movegen,
    parser::{self, Parser, ParserExt},
    platform, AnyMove, Color, PlayerFactory, Position, ShortMove,
};
use std::{
    fmt::{self, Display, Formatter},
    io::{self, BufRead, Write},
    process::ExitCode,
    time::Duration,
};

#[derive(Debug, Clone)]
pub enum CliCommand {
    TimeLimit(Duration),
    Opening(Vec<AnyMove>),
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
                .ignore_then(
                    parser::exact(b" ")
                        .ignore_then(AnyMove::parser())
                        .repeat(0..),
                )
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

#[derive(Debug)]
enum CliError {
    IoError(io::Error),
    InvalidCommand(Vec<u8>),
    TimeCommandTooLate,
    OpeningCommandTooLate,
    StartCommandTooLate,
    InvalidOpeningMove(AnyMove),
    InvalidPlayerMove(AnyMove),
    InvalidOpponentMove(ShortMove),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CliError::IoError(e) => write!(f, "IO error: {e}"),
            CliError::InvalidCommand(bytes) => {
                write!(f, "Invalid command: {}", String::from_utf8_lossy(bytes))
            }
            CliError::TimeCommandTooLate => write!(f, "Time command too late"),
            CliError::OpeningCommandTooLate => write!(f, "Opening command too late"),
            CliError::StartCommandTooLate => write!(f, "Start command too late"),
            CliError::InvalidOpeningMove(mov) => write!(f, "Invalid opening move: {mov}"),
            CliError::InvalidPlayerMove(mov) => write!(f, "Invalid player move: {mov}"),
            CliError::InvalidOpponentMove(short_move) => {
                write!(f, "Invalid opponent move: {short_move}")
            }
        }
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError::IoError(e)
    }
}

pub fn run_cli(player_factory: &dyn PlayerFactory) -> ExitCode {
    if let Err(e) = run_internal(player_factory) {
        log::always!("Error: {e}");
        log::flush();
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run_internal(player_factory: &dyn PlayerFactory) -> Result<(), CliError> {
    log::init(Level::Info);
    log::info!("Platform: {}", platform::platform_description());
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut opening = Vec::new();
    let mut position = Position::initial();
    let mut time_limit = None;
    let mut timer = Timer::new(DEFAULT_TIME_LIMIT);
    let mut player = None;
    let mut command_buffer = Vec::new();
    let mut opp_stopwatch: Option<Stopwatch> = None;

    loop {
        log::flush();
        command_buffer.clear();
        let command_len = stdin.read_until(b'\n', &mut command_buffer)?;
        if command_len == 0 {
            log::info!("EOF");
            break;
        }
        let command = CliCommand::parser()
            .then_ignore(parser::endl())
            .parse_all(&command_buffer)
            .map_err(|_| CliError::InvalidCommand(command_buffer.clone()))?;

        match command {
            CliCommand::TimeLimit(duration) => {
                if player.is_some() || time_limit.is_some() {
                    return Err(CliError::TimeCommandTooLate);
                }
                log::info!("time limit {t}", t = duration.as_millis());
                time_limit = Some(duration);
                timer = Timer::new(duration);
            }
            CliCommand::Opening(moves) => {
                if player.is_some() || !opening.is_empty() {
                    return Err(CliError::OpeningCommandTooLate);
                }
                opening = moves;
                for &mov in &opening {
                    log::info!("opening {mov}");
                    position = position
                        .make_any_move(mov)
                        .map_err(|_| CliError::InvalidOpeningMove(mov))?;
                }
            }
            CliCommand::Start => {
                if player.is_some() {
                    return Err(CliError::StartCommandTooLate);
                }
                timer.start();
                player = Some(player_factory.create("", Color::Red, &opening, time_limit));
                log::info!("init {} ms", timer.get().as_millis());
            }
            CliCommand::OpponentMove(short_move) => {
                timer.start();
                let mov = movegen::any_move_from_short_move(&position, short_move)
                    .map_err(|_| CliError::InvalidOpponentMove(short_move))?;

                let mut opp_time = Duration::ZERO;
                if let Some(opp_stopwatch) = opp_stopwatch.as_mut() {
                    opp_stopwatch.stop();
                    opp_time = opp_stopwatch.get();
                }
                log::info!(
                    "{ply}. opp {mov} {t} ms",
                    ply = position.ply() + 1,
                    t = opp_time.as_millis()
                );

                if player.is_none() {
                    player = Some(player_factory.create("", Color::Blue, &opening, time_limit));
                    log::info!("init {t} ms", t = timer.get().as_millis());
                }

                player
                    .as_mut()
                    .unwrap()
                    .opponent_move(&position, mov, &timer);
                position = position.make_any_move(mov).unwrap();
            }
            CliCommand::Quit => {
                log::info!("quit");
                break;
            }
        }

        let Some(player) = player.as_mut() else {
            continue;
        };

        let mov = player.make_move(&position, &timer);
        let short_move = ShortMove::from(mov);
        position = position
            .make_any_move(mov)
            .map_err(|_| CliError::InvalidPlayerMove(mov))?;
        timer.stop();
        log::info!(
            "{ply}. {mov} {t} ms",
            ply = position.ply(),
            t = timer.get().as_millis()
        );

        if opp_stopwatch.is_none() {
            opp_stopwatch = Some(Stopwatch::new());
        }
        opp_stopwatch.as_mut().unwrap().start();

        log::flush();
        writeln!(stdout, "{short_move}")?;
        stdout.flush()?;
    }
    log::flush();
    Ok(())
}
