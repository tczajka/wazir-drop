use crate::{
    Color, LinearEvaluator, Move, PieceSquareFeatures, Player, PlayerFactory, Position, Search,
    SetupMove, Stage,
    clock::Timer,
    constants::{SearchParams, TIME_MARGIN},
    log,
};
use std::{str::FromStr, sync::Arc, time::Duration};

type MainPlayerEvaluator = LinearEvaluator<PieceSquareFeatures>;

struct MainPlayer {
    search_params: SearchParams,
    search: Search<MainPlayerEvaluator>,
}

impl Player for MainPlayer {
    fn make_move(&mut self, position: &Position, timer: &Timer) -> Move {
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
                let fraction = 1.0 / self.search_params.time_alloc_decay_moves;
                let deadline = timer.instant_at(
                    TIME_MARGIN + (time_left.saturating_sub(TIME_MARGIN)).mul_f64(1.0 - fraction),
                );

                let result = self.search.search_regular(position, None, Some(deadline));
                let elapsed = time_left.saturating_sub(timer.get());
                log::info!(
                    "depth {depth} score {score} \
                        root {root_moves_considered}/{root_all_moves} \
                        nodes {nodes} knps {knps:.0} pv {pv}",
                    depth = result
                        .depth
                        .map(|d| d.to_string())
                        .unwrap_or("inf".to_string()),
                    score = result.score.to_relative(position.move_number()),
                    root_moves_considered = result.root_moves_considered,
                    root_all_moves = result.root_all_moves,
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
pub struct MainPlayerFactory {
    search_params: SearchParams,
    evaluator: Arc<MainPlayerEvaluator>,
}

impl MainPlayerFactory {
    pub fn new(search_params: SearchParams, evaluator: &Arc<MainPlayerEvaluator>) -> Self {
        Self {
            search_params,
            evaluator: evaluator.clone(),
        }
    }
}

impl Default for MainPlayerFactory {
    fn default() -> Self {
        Self::new(
            SearchParams::default(),
            &Arc::new(MainPlayerEvaluator::default()),
        )
    }
}

impl PlayerFactory for MainPlayerFactory {
    fn create(
        &self,
        _game_id: &str,
        _color: Color,
        _opening: &[Move],
        _time_limit: Option<Duration>,
    ) -> Box<dyn crate::Player> {
        Box::new(MainPlayer {
            search_params: self.search_params.clone(),
            search: Search::new(&self.evaluator),
        })
    }
}
