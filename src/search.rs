use crate::{
    constants::{Hyperparameters, CHECK_TIMEOUT_NODES, MAX_SEARCH_DEPTH},
    movegen,
    smallvec::SmallVec,
    ttable::{TTable, TTableEntry, TTableScoreType},
    EvaluatedPosition, Evaluator, Position, RegularMove, Score, ScoreExpanded, Stage,
};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    sync::Arc,
    time::Instant,
};

/// Align to 64 bytes to avoid false sharing.
#[repr(align(64))]
pub struct Search<E> {
    hyperparameters: Hyperparameters,
    evaluator: Arc<E>,
    ttable: TTable,
}

impl<E: Evaluator> Search<E> {
    pub fn new(hyperparameters: &Hyperparameters, evaluator: &Arc<E>) -> Self {
        Self {
            hyperparameters: hyperparameters.clone(),
            evaluator: Arc::clone(evaluator),
            ttable: TTable::new(hyperparameters.ttable_size),
        }
    }

    pub fn search(
        &mut self,
        position: &Position,
        max_depth: Option<u16>,
        deadline: Option<Instant>,
    ) -> SearchResult {
        match position.stage() {
            Stage::Setup => panic!("search does not support setup"),
            Stage::Regular => {}
            Stage::End(outcome) => {
                return SearchResult {
                    score: outcome.to_score(position.move_number()),
                    pv: Variation::empty(),
                    depth: u16::MAX,
                    root_moves_considered: 0,
                    root_all_moves: 0,
                    nodes: 0,
                };
            }
        }

        self.ttable.new_epoch();
        let mut stats = SearchStats::new();
        let max_depth = max_depth.unwrap_or(MAX_SEARCH_DEPTH);
        let eposition = EvaluatedPosition::new(&self.evaluator, position.clone());

        let (mut search_result, moves) = self.search_one_ply(&eposition, &mut stats);
        let mut moves: Vec<RegularMove> = moves.into_iter().map(|(mov, _)| mov).collect();

        stats.deadline = deadline;

        let final_depth;
        let root_moves_considered;

        let mut depth = 1;
        'iterative_deepening: loop {
            if search_result.inf_depth {
                final_depth = u16::MAX;
                root_moves_considered = moves.len();
                break;
            }
            depth += 1;
            if depth > max_depth {
                final_depth = max_depth;
                root_moves_considered = moves.len();
                break;
            }
            let epos2 = eposition.make_regular_move(moves[0]).unwrap();
            let Ok(result) = self.search_alpha_beta(
                &epos2,
                -Score::IMMEDIATE_WIN,
                Score::IMMEDIATE_WIN,
                depth - 1,
                &mut stats,
            ) else {
                final_depth = depth - 1;
                root_moves_considered = moves.len();
                break;
            };
            search_result.pv = result.pv.add_front(moves[0]);
            search_result.score = -result.score;
            search_result.inf_depth = result.inf_depth;

            for move_idx in 1..moves.len() {
                let mov = moves[move_idx];
                let epos2 = eposition.make_regular_move(mov).unwrap();
                let Ok(result) = self.search_alpha_beta(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    -search_result.score,
                    depth - 1,
                    &mut stats,
                ) else {
                    search_result.inf_depth = false;
                    final_depth = depth;
                    root_moves_considered = move_idx;
                    break 'iterative_deepening;
                };
                let score = -result.score;
                search_result.inf_depth &= result.inf_depth;
                if score > search_result.score {
                    search_result.score = score;
                    search_result.pv = result.pv.add_front(mov);
                    moves[0..move_idx].rotate_right(1);
                }
            }
        }

        SearchResult {
            score: search_result.score,
            pv: search_result.pv,
            depth: final_depth,
            root_moves_considered,
            root_all_moves: moves.len(),
            nodes: stats.nodes,
        }
    }

    // Returns moves sorted by score.
    fn search_one_ply(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        stats: &mut SearchStats,
    ) -> (SearchResultInternal, Vec<(RegularMove, Score)>) {
        let mut moves: Vec<(RegularMove, Score)> = Vec::new();
        let mut search_result = SearchResultInternal {
            score: -Score::IMMEDIATE_WIN,
            inf_depth: true,
            pv: Variation::empty(),
        };

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result = self
                .search_alpha_beta(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    0,
                    stats,
                )
                .unwrap();
            search_result.inf_depth &= result.inf_depth;
            let score = -result.score;
            moves.push((mov, score));
            if score > search_result.score {
                search_result.score = score;
                search_result.pv = result.pv.add_front(mov);
                let last = moves.len() - 1;
                moves.swap(0, last);
            }
        }
        assert!(!moves.is_empty(), "Stalemate");
        moves[1..].sort_by_key(|&(_, score)| -score);

        (search_result, moves)
    }

    /// Recursive search function.
    fn search_alpha_beta(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: u16,
        stats: &mut SearchStats,
    ) -> Result<SearchResultInternal, Timeout> {
        stats.new_node()?;
        let move_number = eposition.position().move_number();

        // Check whether game ended.
        if let Stage::End(outcome) = eposition.position().stage() {
            return Ok(SearchResultInternal {
                score: outcome.to_score(move_number),
                inf_depth: true,
                pv: Variation::empty(),
            });
        }

        // Endgame distance pruning.
        {
            let best_win = ScoreExpanded::Win(move_number + 1).into();
            if best_win <= alpha {
                return Ok(SearchResultInternal {
                    score: best_win,
                    inf_depth: true,
                    pv: Variation::empty_truncated(),
                });
            }
            let worst_loss = ScoreExpanded::Loss(move_number + 2).into();
            if worst_loss >= beta {
                return Ok(SearchResultInternal {
                    score: worst_loss,
                    inf_depth: true,
                    pv: Variation::empty_truncated(),
                });
            }
        }

        let mut tt_move = None;

        // Transposition table lookup.
        if depth >= self.hyperparameters.min_ttable_depth {
            if let Some(ttentry) = self.ttable.get(eposition.position().hash()) {
                // Transposition table cutoff.
                if ttentry.depth >= depth {
                    let score = ttentry.score.to_absolute(move_number);
                    let cutoff = match ttentry.score_type {
                        TTableScoreType::None => false,
                        TTableScoreType::Exact => true,
                        TTableScoreType::LowerBound => score >= beta,
                        TTableScoreType::UpperBound => score <= alpha,
                    };
                    if cutoff {
                        return Ok(SearchResultInternal {
                            score,
                            inf_depth: ttentry.depth == u16::MAX,
                            pv: Variation::empty_truncated(),
                        });
                    }
                }
                tt_move = ttentry.mov;
            }
        }

        let result =
            self.search_alpha_beta_real_work(eposition, alpha, beta, depth, tt_move, stats)?;

        if depth >= self.hyperparameters.min_ttable_depth {
            let score_type = if result.score >= beta {
                TTableScoreType::LowerBound
            } else if result.score <= alpha {
                TTableScoreType::UpperBound
            } else {
                TTableScoreType::Exact
            };

            self.ttable.set(
                eposition.position().hash(),
                TTableEntry {
                    depth: if result.inf_depth { u16::MAX } else { depth },
                    mov: result.pv.moves.first().copied(),
                    score_type,
                    score: result.score.to_relative(move_number),
                },
            );
        }

        Ok(result)
    }

    // No early cutoff, we have to do real work.
    fn search_alpha_beta_real_work(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: u16,
        tt_move: Option<RegularMove>,
        stats: &mut SearchStats,
    ) -> Result<SearchResultInternal, Timeout> {
        // Leaf node.
        if depth == 0 {
            return Ok(SearchResultInternal {
                score: ScoreExpanded::Eval(eposition.evaluate()).into(),
                inf_depth: false,
                pv: Variation::empty(),
            });
        }

        // Transposition table move first, then all other moves.
        let moves = tt_move.into_iter().chain(
            movegen::regular_pseudomoves(eposition.position()).filter(|&mov| Some(mov) != tt_move),
        );

        let mut result = SearchResultInternal {
            score: -Score::IMMEDIATE_WIN,
            inf_depth: true,
            pv: Variation::empty(),
        };

        for mov in moves {
            let Ok(epos2) = eposition.make_regular_move(mov) else {
                // Illegal move. Could be a hash collision in the transposition table
                // or invalid killer move.
                continue;
            };
            let result2 =
                self.search_alpha_beta(&epos2, -beta, -alpha.max(result.score), depth - 1, stats)?;
            let score = -result2.score;
            result.inf_depth &= result2.inf_depth;
            if score > result.score {
                result.score = score;
                if result.score > alpha {
                    result.pv = result2.pv.add_front(mov);
                }
                if result.score >= beta {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// No deadline. Returns moves sorted by score.
    pub fn search_top_variations(
        &mut self,
        position: &Position,
        max_depth: u16,
        max_eval_diff: i32,
    ) -> Vec<TopVariation> {
        assert_eq!(position.stage(), Stage::Regular);
        assert!(max_eval_diff >= 0);
        self.ttable.new_epoch();
        let mut stats = SearchStats::new();
        let eposition = EvaluatedPosition::new(&self.evaluator, position.clone());

        let mut variations: Vec<TopVariation> = Vec::new();

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result = self
                .search_alpha_beta(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    0,
                    &mut stats,
                )
                .unwrap();
            let score = -result.score;
            variations.push(TopVariation {
                variation: result.pv.add_front(mov),
                score,
            });
        }

        variations.sort_by_key(|v| -v.score);

        for depth in 2..=max_depth {
            let mov = variations[0].variation.moves[0];
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result = self
                .search_alpha_beta(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    depth - 1,
                    &mut stats,
                )
                .unwrap();
            variations[0] = TopVariation {
                variation: result.pv.add_front(mov),
                score: -result.score,
            };

            for move_idx in 1..variations.len() {
                let mov = variations[move_idx].variation.moves[0];
                let epos2 = eposition.make_regular_move(mov).unwrap();
                let result = self
                    .search_alpha_beta(
                        &epos2,
                        -Score::IMMEDIATE_WIN,
                        -variations[0].score.offset(-max_eval_diff).prev(),
                        depth - 1,
                        &mut stats,
                    )
                    .unwrap();
                let score = -result.score;
                variations[move_idx] = TopVariation {
                    variation: result.pv.add_front(mov),
                    score,
                };
                if score > variations[0].score {
                    variations.swap(0, move_idx);
                }
            }
        }

        let threshold = variations[0].score.offset(-max_eval_diff);
        variations
            .into_iter()
            .take_while(|v| v.score >= threshold)
            .collect()
    }
}

pub struct SearchResult {
    pub score: Score,
    pub pv: Variation,
    pub depth: u16,
    pub root_moves_considered: usize,
    pub root_all_moves: usize,
    pub nodes: u64,
}

pub struct TopVariation {
    pub score: Score,
    pub variation: Variation,
}

struct SearchResultInternal {
    score: Score,
    inf_depth: bool,
    pv: Variation,
}

struct SearchStats {
    deadline: Option<Instant>,
    nodes: u64,
}

impl SearchStats {
    pub fn new() -> Self {
        Self {
            deadline: None,
            nodes: 0,
        }
    }

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
    pub moves: SmallVec<RegularMove, { Self::MAX_LENGTH }>,
    pub truncated: bool,
}

impl Variation {
    pub const MAX_LENGTH: usize = 100;

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
            if res.moves.len() >= Self::MAX_LENGTH {
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
