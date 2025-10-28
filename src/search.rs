use crate::{
    movegen, smallvec::SmallVec, EvaluatedPosition, Evaluator, Position, RegularMove, Score, Stage,
};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    rc::Rc,
    time::Instant,
};

pub struct Search<E> {
    evaluator: Rc<E>,
}

impl<E: Evaluator> Search<E> {
    const MAX_DEPTH: usize = 100;

    pub fn new(evaluator: &Rc<E>) -> Self {
        Self {
            evaluator: Rc::clone(evaluator),
        }
    }

    pub fn search_regular(
        &mut self,
        position: &Position,
        max_depth: Option<usize>,
        deadline: Option<Instant>,
    ) -> SearchRegularResult {
        assert_eq!(position.stage(), Stage::Regular);
        let mut stats = SearchStats { deadline, nodes: 0 };
        let max_depth = max_depth.unwrap_or(Self::MAX_DEPTH);
        let eposition = EvaluatedPosition::new(&self.evaluator, position.clone());

        let mut moves: Vec<(RegularMove, Score)> = Vec::new();
        let mut pv = Variation::new();
        let mut best_score = -Score::IMMEDIATE_WIN;

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result = self.qsearch(
                &epos2,
                -Score::IMMEDIATE_WIN,
                Score::IMMEDIATE_WIN,
                &mut stats,
            );
            let score = result.score.back();
            moves.push((mov, score));
            if score > best_score {
                best_score = score;
                pv = result.pv.add_front(mov);
                let last = moves.len() - 1;
                moves.swap(0, last);
            }
        }

        if moves.is_empty() {
            panic!("Stalemate");
        }

        SearchRegularResult {
            score: best_score,
            pv,
            depth: 1,
            root_moves_considered: moves.len(),
            root_all_moves: moves.len(),
            nodes: stats.nodes,
        }
    }

    fn qsearch(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        stats: &mut SearchStats,
    ) -> PVResult {
        if eposition.position().stage() == Stage::End {
            return PVResult {
                score: -Score::IMMEDIATE_WIN,
                pv: Variation::new(),
            };
        }

        PVResult {
            score: Score::from_eval(eposition.evaluate()),
            pv: Variation::new(),
        }
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
