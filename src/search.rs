use crate::{
    constants::{
        Depth, Eval, Hyperparameters, CHECK_TIMEOUT_NODES, INFINITE_DEPTH, MAX_SEARCH_DEPTH,
    },
    movegen,
    ttable::{TTable, TTableEntry, TTableScoreType},
    variation::LongVariation,
    EmptyVariation, EvaluatedPosition, Evaluator, ExtendableVariation, NonEmptyVariation, PVTable,
    Position, RegularMove, Score, ScoreExpanded, Stage, Variation,
};
use std::{sync::Arc, time::Instant};

pub struct Search<E> {
    hyperparameters: Hyperparameters,
    evaluator: Arc<E>,
    ttable: TTable,
    pvtable: PVTable,
}

impl<E: Evaluator> Search<E> {
    pub fn new(hyperparameters: &Hyperparameters, evaluator: &Arc<E>) -> Self {
        Self {
            hyperparameters: hyperparameters.clone(),
            evaluator: Arc::clone(evaluator),
            ttable: TTable::new(hyperparameters.ttable_size),
            pvtable: PVTable::new(hyperparameters.pvtable_size),
        }
    }

    pub fn search(
        &mut self,
        position: &Position,
        max_depth: Option<Depth>,
        deadline: Option<Instant>,
    ) -> SearchResult {
        let mut instance = self.instance();
        instance.search(position, max_depth, deadline)
    }

    pub fn search_top_variations(
        &mut self,
        position: &Position,
        max_depth: Depth,
        max_eval_diff: Eval,
    ) -> Vec<TopVariation> {
        let mut instance = self.instance();
        instance.search_top_variations(position, max_depth, max_eval_diff)
    }

    fn instance(&mut self) -> SearchInstance<'_, E> {
        SearchInstance {
            hyperparameters: self.hyperparameters.clone(),
            evaluator: &self.evaluator,
            ttable: &mut self.ttable,
            pvtable: &mut self.pvtable,
            deadline: None,
            nodes: 0,
        }
    }
}

struct SearchInstance<'a, E: Evaluator> {
    hyperparameters: Hyperparameters,
    evaluator: &'a E,
    ttable: &'a mut TTable,
    pvtable: &'a mut PVTable,
    deadline: Option<Instant>,
    nodes: u64,
}

impl<E: Evaluator> SearchInstance<'_, E> {
    fn search(
        &mut self,
        position: &Position,
        max_depth: Option<Depth>,
        deadline: Option<Instant>,
    ) -> SearchResult {
        match position.stage() {
            Stage::Setup => panic!("search does not support setup"),
            Stage::Regular => {}
            Stage::End(outcome) => {
                return SearchResult {
                    score: outcome.to_score(position.move_number()),
                    pv: LongVariation::empty(),
                    depth: INFINITE_DEPTH,
                    root_moves_considered: 0,
                    root_all_moves: 0,
                    nodes: 0,
                };
            }
        }

        self.ttable.new_epoch();
        self.pvtable.new_epoch();
        let max_depth = max_depth.unwrap_or(MAX_SEARCH_DEPTH);
        let eposition = EvaluatedPosition::new(self.evaluator, position.clone());

        let (mut search_result, moves) = self.search_one_ply(&eposition);
        let mut moves: Vec<RegularMove> = moves.into_iter().map(|(mov, _)| mov).collect();

        self.deadline = deadline;

        let mut depth = 1;
        let mut root_moves_considered = moves.len();

        // iterative deepening loop
        _ = || -> Result<(), Timeout> {
            loop {
                if search_result.inf_depth {
                    depth = INFINITE_DEPTH;
                    return Ok(());
                }
                if depth >= max_depth {
                    return Ok(());
                }
                let epos2 = eposition.make_regular_move(moves[0]).unwrap();
                let result = self.search_alpha_beta::<LongVariation>(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    depth,
                )?;
                search_result.pv = result.pv.add_front(moves[0]);
                search_result.score = -result.score;
                search_result.inf_depth = result.inf_depth;
                depth += 1;
                root_moves_considered = 1;

                while root_moves_considered < moves.len() {
                    let mov = moves[root_moves_considered];
                    let epos2 = eposition.make_regular_move(mov).unwrap();

                    // PVS: Try null window first.
                    let result = self.search_alpha_beta::<EmptyVariation>(
                        &epos2,
                        -search_result.score.next(),
                        -search_result.score,
                        depth - 1,
                    )?;
                    search_result.inf_depth &= result.inf_depth;
                    let score = -result.score;
                    if score > search_result.score {
                        // Full window search.
                        let result = self.search_alpha_beta::<LongVariation>(
                            &epos2,
                            -Score::IMMEDIATE_WIN,
                            -search_result.score,
                            depth - 1,
                        )?;
                        let score = -result.score;
                        search_result.inf_depth &= result.inf_depth;
                        if score > search_result.score {
                            search_result.score = score;
                            search_result.pv = result.pv.add_front(mov);
                            moves[0..=root_moves_considered].rotate_right(1);
                        }
                    }
                    root_moves_considered += 1;
                }
            }
        }();

        SearchResult {
            score: search_result.score,
            pv: search_result.pv,
            depth,
            root_moves_considered,
            root_all_moves: moves.len(),
            nodes: self.nodes,
        }
    }

    // Returns moves sorted by score.
    fn search_one_ply(
        &mut self,
        eposition: &EvaluatedPosition<E>,
    ) -> (
        SearchResultInternal<LongVariation>,
        Vec<(RegularMove, Score)>,
    ) {
        let mut moves: Vec<(RegularMove, Score)> = Vec::new();
        let mut search_result = SearchResultInternal {
            score: -Score::IMMEDIATE_WIN,
            inf_depth: true,
            pv: LongVariation::empty(),
        };

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result = self
                .search_alpha_beta::<LongVariation>(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    0,
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
    fn search_alpha_beta<V: ExtendableVariation>(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: Depth,
    ) -> Result<SearchResultInternal<V>, Timeout> {
        self.new_node()?;
        let move_number = eposition.position().move_number();

        // Check whether game ended.
        if let Stage::End(outcome) = eposition.position().stage() {
            return Ok(SearchResultInternal {
                score: outcome.to_score(move_number),
                inf_depth: true,
                pv: V::empty(),
            });
        }

        // Endgame distance pruning.
        {
            let best_win = ScoreExpanded::Win(move_number + 1).into();
            if best_win <= alpha {
                return Ok(SearchResultInternal {
                    score: best_win,
                    inf_depth: true,
                    pv: V::empty_truncated(),
                });
            }
            let worst_loss = ScoreExpanded::Loss(move_number + 2).into();
            if worst_loss >= beta {
                return Ok(SearchResultInternal {
                    score: worst_loss,
                    inf_depth: true,
                    pv: V::empty_truncated(),
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
                    let mut pv = V::empty_truncated();
                    let cutoff = match ttentry.score_type {
                        TTableScoreType::None => false,
                        TTableScoreType::Exact => {
                            if let Some(v) =
                                V::pvtable_get(self.pvtable, eposition.position().hash())
                            {
                                pv = v;
                            }
                            true
                        }
                        TTableScoreType::LowerBound => score >= beta,
                        TTableScoreType::UpperBound => score <= alpha,
                    };
                    if cutoff {
                        return Ok(SearchResultInternal {
                            score,
                            inf_depth: ttentry.depth == INFINITE_DEPTH,
                            pv,
                        });
                    }
                }
                tt_move = ttentry.mov;
            }
        }

        // Search with V::Extended so that we have a TT move.
        let result = self
            .search_alpha_beta_real_work::<V::Extended>(eposition, alpha, beta, depth, tt_move)?;
        let mov = result.pv.first();
        let pv = result.pv.truncate();

        if depth >= self.hyperparameters.min_ttable_depth {
            let score_type = if result.score >= beta {
                TTableScoreType::LowerBound
            } else if result.score <= alpha {
                TTableScoreType::UpperBound
            } else {
                V::pvtable_set(self.pvtable, eposition.position().hash(), pv.clone());
                TTableScoreType::Exact
            };

            self.ttable.set(
                eposition.position().hash(),
                TTableEntry {
                    depth: if result.inf_depth {
                        INFINITE_DEPTH
                    } else {
                        depth
                    },
                    mov,
                    score_type,
                    score: result.score.to_relative(move_number),
                },
            );
        }

        Ok(SearchResultInternal {
            score: result.score,
            inf_depth: result.inf_depth,
            pv,
        })
    }

    // No early cutoff, we have to do real work.
    fn search_alpha_beta_real_work<V: NonEmptyVariation>(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: Depth,
        tt_move: Option<RegularMove>,
    ) -> Result<SearchResultInternal<V>, Timeout> {
        // Leaf node.
        if depth <= 0 {
            return Ok(SearchResultInternal {
                score: ScoreExpanded::Eval(eposition.evaluate()).into(),
                inf_depth: false,
                pv: V::empty(),
            });
        }

        // Transposition table move first, then all other moves.
        let moves = tt_move.into_iter().chain(
            movegen::regular_pseudomoves(eposition.position()).filter(|&mov| Some(mov) != tt_move),
        );

        let mut result = SearchResultInternal {
            score: -Score::IMMEDIATE_WIN,
            inf_depth: true,
            pv: V::empty(),
        };

        for mov in moves {
            let Ok(epos2) = eposition.make_regular_move(mov) else {
                // Illegal move. Could be a hash collision in the transposition table
                // or invalid killer move.
                continue;
            };

            let alpha2 = alpha.max(result.score);

            // Try null window first.
            let null_window_pass = beta == alpha2.next() || {
                let result2 = self.search_alpha_beta::<EmptyVariation>(
                    &epos2,
                    -alpha2.next(),
                    -alpha2,
                    depth - 1,
                )?;
                result.inf_depth &= result2.inf_depth;
                -result2.score > alpha2
            };
            if null_window_pass {
                let result2 =
                    self.search_alpha_beta::<V::Truncated>(&epos2, -beta, -alpha2, depth - 1)?;
                result.inf_depth &= result2.inf_depth;
                let score = -result2.score;
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
        }

        Ok(result)
    }

    /// No deadline. Returns moves sorted by score.
    pub fn search_top_variations(
        &mut self,
        position: &Position,
        max_depth: Depth,
        max_eval_diff: Eval,
    ) -> Vec<TopVariation> {
        assert_eq!(position.stage(), Stage::Regular);
        assert!(max_eval_diff >= 0);
        self.ttable.new_epoch();
        self.pvtable.new_epoch();
        let eposition = EvaluatedPosition::new(self.evaluator, position.clone());

        let mut variations: Vec<TopVariation> = Vec::new();

        for mov in movegen::regular_pseudomoves(eposition.position()) {
            let epos2 = eposition.make_regular_move(mov).unwrap();
            let result = self
                .search_alpha_beta::<LongVariation>(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    0,
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
                .search_alpha_beta::<LongVariation>(
                    &epos2,
                    -Score::IMMEDIATE_WIN,
                    Score::IMMEDIATE_WIN,
                    depth - 1,
                )
                .unwrap();
            variations[0] = TopVariation {
                variation: result.pv.add_front(mov),
                score: -result.score,
            };

            for move_idx in 1..variations.len() {
                let mov = variations[move_idx].variation.first().unwrap();
                let epos2 = eposition.make_regular_move(mov).unwrap();
                let threshold = variations[0].score.offset(-max_eval_diff);
                // Null window first.
                let result = self
                    .search_alpha_beta::<EmptyVariation>(
                        &epos2,
                        -threshold,
                        -threshold.prev(),
                        depth - 1,
                    )
                    .unwrap();
                if -result.score >= threshold {
                    // Full window search.
                    let result = self
                        .search_alpha_beta::<LongVariation>(
                            &epos2,
                            -Score::IMMEDIATE_WIN,
                            -threshold.prev(),
                            depth - 1,
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
        }

        let threshold = variations[0].score.offset(-max_eval_diff);
        variations
            .into_iter()
            .take_while(|v| v.score >= threshold)
            .collect()
    }

    fn new_node(&mut self) -> Result<(), Timeout> {
        self.nodes += 1;
        if let Some(deadline) = self.deadline {
            if self.nodes % CHECK_TIMEOUT_NODES == 0 && Instant::now() >= deadline {
                return Err(Timeout);
            }
        }
        Ok(())
    }
}

pub struct SearchResult {
    pub score: Score,
    pub pv: LongVariation,
    pub depth: Depth,
    pub root_moves_considered: usize,
    pub root_all_moves: usize,
    pub nodes: u64,
}

pub struct TopVariation {
    pub score: Score,
    pub variation: LongVariation,
}

struct SearchResultInternal<V> {
    score: Score,
    inf_depth: bool,
    pv: V,
}

#[derive(Debug)]
struct Timeout;
