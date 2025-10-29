use crate::{
    clock::Timer, log, Color, LinearEvaluator, Move, PieceSquareFeatures, Player, PlayerFactory,
    Position, Search, SetupMove, Stage,
};
use std::{str::FromStr, sync::Arc, time::Duration};

pub struct MainPlayer {
    search: Search<LinearEvaluator<PieceSquareFeatures>>,
}

impl MainPlayer {
    #[allow(clippy::new_without_default)]
    pub fn new(evaluator: &Arc<LinearEvaluator<PieceSquareFeatures>>) -> Self {
        Self {
            search: Search::new(evaluator),
        }
    }
}

impl Player for MainPlayer {
    fn make_move(&mut self, position: &Position, _timer: &Timer) -> Move {
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
                let result = self.search.search_regular(position, None, None);
                log::info!(
                    "depth {depth} score {score} \
                        root {root_moves_considered}/{root_all_moves} \
                        nodes {nodes} pv {pv}\n",
                    depth = result.depth,
                    score = result.score,
                    root_moves_considered = result.root_moves_considered,
                    root_all_moves = result.root_all_moves,
                    nodes = result.nodes,
                    pv = result.pv,
                );
                result.pv.moves[0].into()
            }
            Stage::End => panic!("Game is over"),
        }
    }
}

#[derive(Debug)]
pub struct MainPlayerFactory {
    evaluator: Arc<LinearEvaluator<PieceSquareFeatures>>,
}

impl MainPlayerFactory {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            evaluator: Arc::new(LinearEvaluator::default()),
        }
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
        Box::new(MainPlayer::new(&self.evaluator))
    }
}
