use crate::{
    clock::Timer,
    constants::{Hyperparameters, TIME_MARGIN},
    log, AnyMove, Color, DefaultEvaluator, Evaluator, Player, PlayerFactory, Position, Search,
    SetupMove, Stage,
};
use std::{str::FromStr, sync::Arc, time::Duration};

struct MainPlayer<E: Evaluator> {
    hyperparameters: Hyperparameters,
    search: Search<E>,
}

impl<E: Evaluator> Player for MainPlayer<E> {
    fn make_move(&mut self, position: &Position, timer: &Timer) -> AnyMove {
        match position.stage() {
            Stage::Setup => {
                let mov = SetupMove::from_str("AAAAAAWANDDDDFFA").unwrap();
                SetupMove {
                    color: position.to_move(),
                    ..mov
                }
                .into()
            }
            Stage::Regular => {
                // TODO: Use more time when approaching 100 moves.
                let time_left = timer.get();
                let fraction = 1.0 / self.hyperparameters.time_alloc_decay_moves;
                let deadline = timer.instant_at(
                    TIME_MARGIN + (time_left.saturating_sub(TIME_MARGIN)).mul_f64(1.0 - fraction),
                );

                let result = self.search.search(
                    position,
                    None, /* max_depth */
                    Some(deadline),
                    None, /* multi_move_threshold */
                );
                let elapsed = time_left.saturating_sub(timer.get());
                log::info!(
                    "depth={depth} score={score} \
                        root={root_moves_considered}/{root_all_moves} \
                        nodes={nodes} knps={knps:.0} pv={pv}",
                    depth = result.depth,
                    score = result.score.to_relative(position.ply()),
                    root_moves_considered = result.root_moves_considered,
                    root_all_moves = result.num_root_moves,
                    nodes = result.nodes,
                    knps = result.nodes as f64 / elapsed.as_secs_f64() / 1000.0,
                    pv = result.pv,
                );
                result.pv.moves[0].into()
            }
            Stage::End(_) => panic!("Game is over"),
        }
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
        _opening: &[AnyMove],
        _time_limit: Option<Duration>,
    ) -> Box<dyn crate::Player> {
        Box::new(MainPlayer {
            hyperparameters: self.hyperparameters.clone(),
            search: Search::new(&self.hyperparameters, &self.evaluator),
        })
    }
}
