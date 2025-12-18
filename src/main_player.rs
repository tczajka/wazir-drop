use crate::{
    book,
    clock::Timer,
    constants::{Hyperparameters, Ply, PLY_AFTER_SETUP, PLY_DRAW, TIME_MARGIN},
    log, AnyMove, Color, Deadlines, DefaultEvaluator, Evaluator, History, Player, PlayerFactory,
    Position, Search, SetupMove, Stage,
};
use std::{sync::Arc, time::Duration};

struct MainPlayer<E: Evaluator> {
    hyperparameters: Hyperparameters,
    search: Search<E>,
    red_setup: Option<SetupMove>,
    position: Position,
    history: History,
}

impl<E: Evaluator> MainPlayer<E> {
    fn time_allocation(&self, ply: Ply, time_left: Duration, timer: &Timer) -> Deadlines {
        let mut weight = 1.0;
        let mut total_weight = 0.0;
        let mut p = ply;
        while p < PLY_DRAW {
            total_weight += weight;
            let reduction = if p < PLY_AFTER_SETUP {
                self.hyperparameters.time_reduction_per_setup_move
            } else if p < self.hyperparameters.late_ply {
                self.hyperparameters.time_reduction_per_move
            } else {
                self.hyperparameters.time_reduction_per_late_move
            };
            weight *= 1.0 - reduction;
            p += 2;
        }
        let to_allocate = time_left.saturating_sub(TIME_MARGIN);
        let fraction = 1.0 / total_weight;
        let soft_fraction = fraction * self.hyperparameters.soft_time_fraction;
        let next_depth_fraction = fraction * self.hyperparameters.start_next_depth_fraction;
        let panic_fraction = (fraction * self.hyperparameters.panic_multiplier)
            .min(self.hyperparameters.panic_max_remaining)
            .max(fraction);
        let panic_soft_fraction = panic_fraction * self.hyperparameters.soft_time_fraction;
        Deadlines {
            hard: timer.instant_at(time_left.saturating_sub(to_allocate.mul_f64(fraction))),
            soft: timer.instant_at(time_left.saturating_sub(to_allocate.mul_f64(soft_fraction))),
            start_next_depth: timer
                .instant_at(time_left.saturating_sub(to_allocate.mul_f64(next_depth_fraction))),
            panic_hard: timer
                .instant_at(time_left.saturating_sub(to_allocate.mul_f64(panic_fraction))),
            panic_soft: timer
                .instant_at(time_left.saturating_sub(to_allocate.mul_f64(panic_soft_fraction))),
        }
    }

    fn move_made(&mut self, mov: AnyMove) {
        self.position = self.position.make_any_move(mov).expect("Invalid move");
        match mov {
            AnyMove::Setup(mov) => {
                if mov.color == Color::Red {
                    self.red_setup = Some(mov);
                }
                self.history.push_position_irreversible(&self.position);
            }
            AnyMove::Regular(_) => {
                self.history.push_position(&self.position);
            }
        }
    }
}

impl<E: Evaluator> Player for MainPlayer<E> {
    fn opponent_move(&mut self, _position: &Position, mov: AnyMove, _timer: &Timer) {
        self.move_made(mov);
    }

    fn make_move(&mut self, position: &Position, timer: &Timer) -> AnyMove {
        let time_left = timer.get();
        let deadlines = self.time_allocation(position.ply(), time_left, timer);
        let mov = match position.stage() {
            Stage::Setup => match position.to_move() {
                Color::Red => book::red_setup().into(),
                Color::Blue => {
                    let red_setup = self.red_setup.expect("Red setup not found");
                    if let Some(mov) = book::blue_setup(red_setup) {
                        return mov.into();
                    }
                    let result = self.search.search_blue_setup(
                        red_setup,
                        None,
                        Some(deadlines),
                        &book::blue_setup_moves(),
                    );
                    let elapsed = time_left.saturating_sub(timer.get());
                    log::info!(
                        "d={depth} {root_moves_considered}/{root_all_moves} \
                                s={score} n={knodes}k kns={knps:.0} t={t}ms pv={setup} {pv}",
                        depth = result.depth,
                        root_moves_considered = result.root_moves_considered,
                        root_all_moves = result.num_root_moves,
                        score = result.score.to_relative(position.ply()),
                        knodes = result.nodes / 1000,
                        knps = result.nodes as f64 / elapsed.as_secs_f64() / 1000.0,
                        setup = result.mov,
                        t = elapsed.as_millis(),
                        pv = result.pv,
                    );
                    result.mov.into()
                }
            },
            Stage::Regular => {
                let result = self.search.search(
                    position,
                    None, /* max_depth */
                    Some(deadlines),
                    None,  /* multi_move_threshold */
                    false, /* is_score_important */
                    &self.history,
                );
                let elapsed = time_left.saturating_sub(timer.get());
                log::info!(
                    "d={depth} {root_moves_considered}/{root_all_moves} \
                        s={score} \
                        n={knodes}k kns={knps:.0} t={t}ms pv={pv}",
                    depth = result.depth,
                    root_moves_considered = result.root_moves_considered,
                    root_all_moves = result.num_root_moves,
                    score = result.score.to_relative(position.ply()),
                    knodes = result.nodes / 1000,
                    knps = result.nodes as f64 / elapsed.as_secs_f64() / 1000.0,
                    t = elapsed.as_millis(),
                    pv = result.pv,
                );
                result.pv.moves[0].into()
            }
            Stage::End(_) => panic!("Game is over"),
        };
        self.move_made(mov);
        mov
    }
}

#[derive(Debug)]
pub struct MainPlayerFactory<E: Evaluator> {
    hyperparameters: Hyperparameters,
    evaluator: Arc<E>,
}

impl<E: Evaluator> MainPlayerFactory<E> {
    pub fn new(hyperparameters: &Hyperparameters, evaluator: &Arc<E>) -> Self {
        Self {
            hyperparameters: hyperparameters.clone(),
            evaluator: evaluator.clone(),
        }
    }
}

impl Default for MainPlayerFactory<DefaultEvaluator> {
    fn default() -> Self {
        Self::new(
            &Hyperparameters::default(),
            &Arc::new(DefaultEvaluator::default()),
        )
    }
}

impl<E: Evaluator> PlayerFactory for MainPlayerFactory<E> {
    fn create(
        &self,
        _game_id: &str,
        _color: Color,
        opening: &[AnyMove],
        _time_limit: Option<Duration>,
    ) -> Box<dyn crate::Player> {
        let position = Position::initial();
        let history = History::new_from_position(&position);
        let mut player = MainPlayer {
            hyperparameters: self.hyperparameters.clone(),
            search: Search::new(&self.hyperparameters, &self.evaluator),
            red_setup: None,
            position,
            history,
        };
        for mov in opening {
            player.move_made(*mov);
        }
        Box::new(player)
    }
}
