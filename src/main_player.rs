use crate::{
    clock::Timer,
    constants::{Hyperparameters, Ply, PLY_DRAW, TIME_MARGIN},
    log, AnyMove, Color, Deadlines, DefaultEvaluator, Evaluator, Player, PlayerFactory, Position,
    Search, SetupMove, Stage,
};
use std::{str::FromStr, sync::Arc, time::Duration};

struct MainPlayer<E: Evaluator> {
    hyperparameters: Hyperparameters,
    search: Search<E>,
}

impl<E: Evaluator> MainPlayer<E> {
    fn time_allocation(&self, ply: Ply, time_left: Duration, timer: &Timer) -> Deadlines {
        let mut weight = 1.0;
        let mut total_weight = 0.0;
        let mut p = ply;
        while p < PLY_DRAW {
            total_weight += weight;
            let reduction = if p < self.hyperparameters.late_ply {
                self.hyperparameters.time_reduction_per_move
            } else {
                self.hyperparameters.time_reduction_per_late_move
            };
            weight *= 1.0 - reduction;
            p += 2;
        }
        let fraction = 1.0 / total_weight;
        let time_left = time_left.saturating_sub(TIME_MARGIN);
        Deadlines {
            hard: timer.instant_at(time_left.mul_f64(1.0 - fraction) + TIME_MARGIN),
            soft: timer.instant_at(
                time_left.mul_f64(1.0 - fraction * self.hyperparameters.soft_time_fraction)
                    + TIME_MARGIN,
            ),
            start_next_depth: timer.instant_at(
                time_left.mul_f64(1.0 - fraction * self.hyperparameters.start_next_depth_fraction)
                    + TIME_MARGIN,
            ),
        }
    }
}

impl<E: Evaluator> Player for MainPlayer<E> {
    fn make_move(&mut self, position: &Position, timer: &Timer) -> AnyMove {
        let time_left = timer.get();
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
                let deadlines = self.time_allocation(position.ply(), time_left, timer);
                let result = self.search.search(
                    position,
                    None, /* max_depth */
                    Some(deadlines),
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
