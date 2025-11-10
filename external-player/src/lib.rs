use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};
use wazir_drop::{
    CliCommand, Color, AnyMove, Player, PlayerFactory, Position, ShortMove,
    clock::Timer,
    movegen,
    parser::{self, ParserExt},
};

#[derive(Debug)]
pub struct ExternalPlayer {
    subprocess: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl ExternalPlayer {
    pub fn new(
        path: &Path,
        log_path: &Path,
        color: Color,
        opening: &[AnyMove],
        time_limit: Option<Duration>,
    ) -> io::Result<Self> {
        let log_file = File::create(log_path)?;
        let mut subprocess = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(log_file)
            .spawn()?;
        let stdin = BufWriter::new(subprocess.stdin.take().unwrap());
        let stdout = BufReader::new(subprocess.stdout.take().unwrap());
        let mut this = Self {
            subprocess,
            stdin,
            stdout,
        };
        if let Some(time_limit) = time_limit {
            this.send_command(CliCommand::TimeLimit(time_limit));
        }
        if !opening.is_empty() {
            this.send_command(CliCommand::Opening(opening.to_vec()));
        }
        if color == Color::Red {
            this.send_command(CliCommand::Start);
        }
        Ok(this)
    }

    fn try_send_command(&mut self, command: CliCommand) -> Result<(), io::Error> {
        writeln!(self.stdin, "{command}")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn send_command(&mut self, command: CliCommand) {
        self.try_send_command(command)
            .unwrap_or_else(|e| panic!("Failed to send command: {e}"));
    }

    fn read_move(&mut self) -> ShortMove {
        let mut line = Vec::new();
        _ = self
            .stdout
            .read_until(b'\n', &mut line)
            .unwrap_or_else(|e| panic!("Failed to read line: {e}"));
        ShortMove::parser()
            .then_ignore(parser::endl())
            .parse_all(&line)
            .unwrap_or_else(|_| panic!("Can't parse move: {}", String::from_utf8_lossy(&line)))
    }
}

impl Player for ExternalPlayer {
    fn opponent_move(&mut self, _position: &Position, mov: AnyMove, _timer: &Timer) {
        self.send_command(CliCommand::OpponentMove(mov.into()));
    }

    fn make_move(&mut self, position: &Position, _timer: &Timer) -> AnyMove {
        let short_move = self.read_move();
        movegen::any_move_from_short_move(position, short_move)
            .unwrap_or_else(|_| panic!("Invalid move: {short_move}"))
    }
}

impl Drop for ExternalPlayer {
    fn drop(&mut self) {
        _ = self.try_send_command(CliCommand::Quit);
        _ = self
            .subprocess
            .wait()
            .unwrap_or_else(|e| panic!("Failed to wait for external player to quit: {e}"));
    }
}

#[derive(Debug)]
pub struct ExternalPlayerFactory {
    name: String,
    path: PathBuf,
    log_dir: PathBuf,
}

impl ExternalPlayerFactory {
    pub fn new(name: &str, path: &Path, log_dir: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
            log_dir: log_dir.to_path_buf(),
        }
    }
}

impl PlayerFactory for ExternalPlayerFactory {
    fn create(
        &self,
        game_id: &str,
        color: Color,
        opening: &[AnyMove],
        time_limit: std::option::Option<Duration>,
    ) -> Box<dyn Player> {
        let log_path = self
            .log_dir
            .join(format!("{name}-{game_id}-{color}.log", name = self.name));
        let player = match ExternalPlayer::new(&self.path, &log_path, color, opening, time_limit) {
            Ok(player) => player,
            Err(e) => panic!("Failed to run external player: {e}"),
        };
        Box::new(player)
    }
}
