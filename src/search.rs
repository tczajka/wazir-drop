use crate::{
    constants::{CHECK_TIMEOUT_NODES, MAX_MOVES_IN_GAME, MAX_SEARCH_DEPTH},
    movegen,
    smallvec::SmallVec,
    EvaluatedPosition, Evaluator, Outcome, Position, RegularMove, Score, Stage,
};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    sync::Arc,
    time::Instant,
};

pub struct Search<E> {
    evaluator: Arc<E>,
}

impl<E: Evaluator> Search<E> {
    pub fn new(evaluator: &Arc<E>) -> Self {
        Self {
            evaluator: Arc::clone(evaluator),
        }
    }

    pub fn search_regular(
        &mut self,
        position: &Position,
        max_depth: Option<usize>,
        deadline: Option<Instant>,
    ) -> SearchRegularResult {
        assert_eq!(position.stage(), Stage::Regular);
        let mut stats = SearchStats {
            deadline: None,
            nodes: 0,
        };
        let max_depth = max_depth.unwrap_or(MAX_SEARCH_DEPTH);
        let eposition = EvaluatedPosition::new(&self.evaluator, position.clone());

        let mut moves: Vec<(RegularMove, Score)> = Vec::new();
        let mut pv = Variation::empty();
        let mut best_score = Score::loss(0);

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result = self
                .search(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    0,
                    &mut stats,
                )
                .unwrap();
            let score = -result.score;
            moves.push((mov, score));
            if score > best_score {
                best_score = score;
                pv = result.pv.add_front(mov);
                let last = moves.len() - 1;
                moves.swap(0, last);
            }
        }
        assert!(!moves.is_empty(), "Stalemate");
        moves[1..].sort_by_key(|&(_, score)| -score);
        let mut moves: Vec<RegularMove> = moves.into_iter().map(|(mov, _)| mov).collect();
        let mut depth: usize = 1;
        let mut root_moves_considered = moves.len();

        stats.deadline = deadline;

        'iterative_deepening: while depth < max_depth
            && best_score > -Score::WIN_TOO_LONG
            && best_score < Score::WIN_TOO_LONG
        {
            let epos2 = eposition.make_regular_move(moves[0]).unwrap();
            let Ok(result) = self.search(
                &epos2,
                -Score::IMMEDIATE_WIN,
                Score::IMMEDIATE_WIN,
                depth,
                &mut stats,
            ) else {
                break;
            };
            depth += 1;
            root_moves_considered = 1;
            pv = result.pv.add_front(moves[0]);
            best_score = -result.score;

            while root_moves_considered < moves.len() {
                let mov = moves[root_moves_considered];
                let epos2 = eposition.make_regular_move(mov).unwrap();
                let Ok(result) = self.search(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    -best_score,
                    depth - 1,
                    &mut stats,
                ) else {
                    break 'iterative_deepening;
                };
                root_moves_considered += 1;
                let score = -result.score;
                if score > best_score {
                    best_score = score;
                    pv = result.pv.add_front(mov);
                    moves[0..root_moves_considered].rotate_right(1);
                }
            }
        }

        SearchRegularResult {
            score: best_score,
            pv,
            depth,
            root_moves_considered,
            root_all_moves: moves.len(),
            nodes: stats.nodes,
        }
    }

    fn search(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: usize,
        stats: &mut SearchStats,
    ) -> Result<PVResult, Timeout> {
        stats.new_node()?;

        // Leaf node.
        if depth == 0 {
            return Ok(PVResult {
                score: Score::from_eval(eposition.evaluate()),
                pv: Variation::empty(),
            });
        }

        let move_number = eposition.position().move_number();

        // Check whether game ended.
        if let Stage::End(outcome) = eposition.position().stage() {
            let score = match outcome {
                Outcome::Draw => Score::from_eval(0),
                _ => Score::loss(move_number),
            };
            return Ok(PVResult {
                score,
                pv: Variation::empty(),
            });
        }

        // Endgame distance pruning.
        {
            let best_win = Score::win(move_number + 1);
            if best_win <= alpha {
                return Ok(PVResult {
                    score: best_win,
                    pv: Variation::empty_truncated(),
                });
            }
            let worst_loss = Score::loss(move_number + 2);
            if worst_loss >= beta {
                return Ok(PVResult {
                    score: worst_loss,
                    pv: Variation::empty_truncated(),
                });
            }
        }

        // Try all moves.
        let mut result = PVResult {
            score: -Score::IMMEDIATE_WIN,
            pv: Variation::empty(),
        };

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result2 = self.search(&epos2, -beta, -alpha.max(result.score), depth - 1, stats)?;
            let score = -result2.score;
            if score > result.score {
                result.score = score;
                result.pv = result2.pv.add_front(mov);
                if result.score >= beta {
                    break;
                }
            }
        }

        Ok(result)
    }
}

pub struct SearchRegularResult {
    pub score: Score,
    pub pv: Variation,
    pub depth: usize,
    pub root_moves_considered: usize,
    pub root_all_moves: usize,
    pub nodes: u64,
}

struct PVResult {
    score: Score,
    pv: Variation,
}

struct SearchStats {
    deadline: Option<Instant>,
    nodes: u64,
}

impl SearchStats {
    pub fn new_node(&mut self) -> Result<(), Timeout> {
        self.nodes += 1;
        if let Some(deadline) = self.deadline {
            if self.nodes % CHECK_TIMEOUT_NODES == 0 && Instant::now() >= deadline {
                return Err(Timeout);
            }
        }
        Ok(())
    }
}

pub struct Variation {
    pub moves: SmallVec<RegularMove, MAX_MOVES_IN_GAME>,
    pub truncated: bool,
}

impl Variation {
    pub fn empty() -> Self {
        Self {
            moves: SmallVec::new(),
            truncated: false,
        }
    }

    pub fn empty_truncated() -> Self {
        Self {
            moves: SmallVec::new(),
            truncated: true,
        }
    }

    pub fn add_front(&self, mov: RegularMove) -> Self {
        let mut res = Self::empty();
        res.moves.push(mov);
        for &mov in self.moves.iter() {
            if res.moves.len() >= MAX_MOVES_IN_GAME {
                res.truncated = true;
                break;
            }
            res.moves.push(mov);
        }

        if self.truncated {
            res.truncated = true;
        }

        res
    }
}

impl Deref for Variation {
    type Target = [RegularMove];

    fn deref(&self) -> &Self::Target {
        &self.moves
    }
}

impl Display for Variation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (index, &mov) in self.moves.iter().enumerate() {
            if index != 0 {
                write!(f, " ")?;
            }
            write!(f, "{mov}")?;
        }
        if self.truncated {
            write!(f, " (trunc)")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Timeout;
