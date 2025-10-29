use crate::{
    constants::{CHECK_TIMEOUT_NODES, MAX_SEARCH_DEPTH},
    movegen,
    smallvec::SmallVec,
    EvaluatedPosition, Evaluator, Position, RegularMove, Score, Stage,
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
        let mut pv = Variation::new();
        let mut best_score = Score::loss(0);

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let alpha = Score::loss(1);
            let beta = Score::win(1);
            let result = self
                .qsearch(&epos2, beta.forward(), alpha.forward(), &mut stats)
                .unwrap();
            let score = result.score.back();
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
            && best_score > Score::loss(depth)
            && best_score < Score::win(depth)
        {
            let alpha = Score::loss(depth + 1);
            let beta = Score::win(depth + 1);

            let epos2 = eposition.make_regular_move(moves[0]).unwrap();
            let Ok(result) =
                self.search(&epos2, beta.forward(), alpha.forward(), depth, &mut stats)
            else {
                break;
            };
            depth += 1;
            root_moves_considered = 1;
            pv = result.pv.add_front(moves[0]);
            best_score = result.score.back();

            while root_moves_considered < moves.len() {
                if best_score >= beta {
                    root_moves_considered = moves.len();
                    break;
                }
                let mov = moves[root_moves_considered];
                let epos2 = eposition.make_regular_move(mov).unwrap();
                let Ok(result) = self.search(
                    &epos2,
                    beta.forward(),
                    best_score.forward(),
                    depth - 1,
                    &mut stats,
                ) else {
                    break 'iterative_deepening;
                };
                root_moves_considered += 1;
                let score = result.score.back();
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
        if depth == 0 {
            return self.qsearch(eposition, alpha, beta, stats);
        }

        stats.new_node()?;

        let mut result = PVResult {
            score: Score::loss(0),
            pv: Variation::new(),
        };

        if eposition.position().stage() == Stage::End {
            return Ok(result);
        }

        result.score = Score::loss(2);
        result.pv.truncated = true;

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            if result.score >= beta {
                break;
            }
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let alpha2 = alpha.max(result.score);
            let result2 =
                self.search(&epos2, beta.forward(), alpha2.forward(), depth - 1, stats)?;
            let score = result2.score.back();
            if score > result.score {
                result.score = score;
                result.pv = result2.pv.add_front(mov);
            }
        }

        Ok(result)
    }

    fn qsearch(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        _alpha: Score,
        _beta: Score,
        stats: &mut SearchStats,
    ) -> Result<PVResult, Timeout> {
        stats.new_node()?;

        let mut result = PVResult {
            score: Score::loss(0),
            pv: Variation::new(),
        };

        if eposition.position().stage() == Stage::End {
            return Ok(result);
        }

        result.score = Score::from_eval(eposition.evaluate());
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
            if self.nodes.is_multiple_of(CHECK_TIMEOUT_NODES) && Instant::now() >= deadline {
                return Err(Timeout);
            }
        }
        Ok(())
    }
}

pub struct Variation {
    pub moves: SmallVec<RegularMove, { Self::MAX_LEN }>,
    pub truncated: bool,
}

impl Variation {
    const MAX_LEN: usize = 100;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            moves: SmallVec::new(),
            truncated: false,
        }
    }

    pub fn add_front(&self, mov: RegularMove) -> Self {
        let mut res = Self::new();
        res.moves.push(mov);
        for &mov in self.moves.iter() {
            if res.moves.len() >= Self::MAX_LEN {
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
