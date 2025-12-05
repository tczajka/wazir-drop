use crate::{
    constants::{
        Depth, Hyperparameters, Ply, CHECK_TIMEOUT_NODES, MAX_SEARCH_DEPTH, NUM_KILLER_MOVES,
        PLY_DRAW,
    },
    either::Either,
    history::History,
    movegen,
    smallvec::SmallVec,
    ttable::{TTable, TTableEntry, TTableScoreType},
    variation::LongVariation,
    EmptyVariation, EvaluatedPosition, Evaluator, ExtendableVariation, Move, NonEmptyVariation,
    PVTable, Position, Score, ScoreExpanded, Stage, Variation,
};
use std::{cmp::Reverse, sync::Arc, time::Instant};

pub struct Search<E> {
    hyperparameters: Hyperparameters,
    evaluator: Arc<E>,
    ttable: TTable,
    pvtable: PVTable,
    killer_moves: Vec<[Option<Move>; NUM_KILLER_MOVES]>,
}

impl<E: Evaluator> Search<E> {
    pub fn new(hyperparameters: &Hyperparameters, evaluator: &Arc<E>) -> Self {
        Self {
            hyperparameters: hyperparameters.clone(),
            evaluator: Arc::clone(evaluator),
            ttable: TTable::new(hyperparameters.ttable_size),
            pvtable: PVTable::new(hyperparameters.pvtable_size),
            killer_moves: vec![[None; NUM_KILLER_MOVES]; PLY_DRAW as usize],
        }
    }

    pub fn search(
        &mut self,
        position: &Position,
        max_depth: Option<Depth>,
        deadline: Option<Instant>,
        multi_move_threshold: Option<i32>,
    ) -> SearchResult {
        let mut instance = SearchInstance::new(self);
        instance.search(position, max_depth, deadline, multi_move_threshold)
    }
}

/// This doesn't work for setup positions.
struct SearchInstance<'a, E: Evaluator> {
    hyperparameters: Hyperparameters,
    evaluator: &'a E,
    ttable: &'a mut TTable,
    pvtable: &'a mut PVTable,
    killer_moves: &'a mut [[Option<Move>; NUM_KILLER_MOVES]],
    deadline: Option<Instant>,
    nodes: u64,
    root_moves: Vec<RootMove>,
    depth: Depth,
    root_moves_considered: usize,
    root_moves_exact_score: usize,
    pv: LongVariation,
    history: History,
}

impl<'a, E: Evaluator> SearchInstance<'a, E> {
    fn new(search: &'a mut Search<E>) -> Self {
        Self {
            hyperparameters: search.hyperparameters.clone(),
            evaluator: &search.evaluator,
            ttable: &mut search.ttable,
            pvtable: &mut search.pvtable,
            killer_moves: &mut search.killer_moves,
            deadline: None,
            nodes: 0,
            root_moves: Vec::new(),
            depth: 0,
            root_moves_considered: 0,
            root_moves_exact_score: 0,
            pv: LongVariation::empty(),
            history: History::new(0),
        }
    }

    fn search(
        &mut self,
        position: &Position,
        max_depth: Option<Depth>,
        deadline: Option<Instant>,
        multi_move_threshold: Option<i32>,
    ) -> SearchResult {
        assert!(multi_move_threshold.is_none() || deadline.is_none());

        let score = match position.stage() {
            Stage::Setup => panic!("SearchInstance::search does not support setup"),
            Stage::Regular => {
                self.search_root(position, max_depth, deadline, multi_move_threshold);
                self.root_moves[0].score
            }
            Stage::End(outcome) => outcome.to_score(position.ply()),
        };

        let top_moves = match multi_move_threshold {
            Some(multi_move_threshold) => {
                let threshold = score.offset(-multi_move_threshold);
                self.root_moves[..self.root_moves_exact_score]
                    .iter()
                    .take_while(|root_move| root_move.score >= threshold)
                    .map(|root_move| ScoredMove {
                        mov: root_move.mov,
                        score: root_move.score,
                    })
                    .collect()
            }
            None => Vec::new(),
        };

        SearchResult {
            score,
            pv: self.pv.clone(),
            top_moves,
            depth: self.depth,
            root_moves_considered: self.root_moves_considered,
            num_root_moves: self.root_moves.len(),
            nodes: self.nodes,
        }
    }

    fn search_root(
        &mut self,
        position: &Position,
        max_depth: Option<Depth>,
        deadline: Option<Instant>,
        multi_move_threshold: Option<i32>,
    ) {
        self.generate_root_captures_of_wazir(position);
        if let Some(root_move) = self.root_moves.first() {
            self.depth = Depth::MAX;
            self.pv = LongVariation::empty().add_front(root_move.mov);
            return;
        }

        self.generate_root_moves(position);

        if self.root_moves.is_empty() {
            self.generate_root_suicides(position);
            let Some(root_move) = self.root_moves.first() else {
                panic!("Stalemate");
            };
            self.depth = Depth::MAX;
            self.pv = LongVariation::empty().add_front(root_move.mov);
            return;
        }

        self.ttable.new_epoch();
        self.pvtable.new_epoch();
        self.history = History::new(position.ply());

        let eposition = EvaluatedPosition::new(self.evaluator, position.clone());
        self.search_shallow(&eposition);

        let max_depth = max_depth.unwrap_or(MAX_SEARCH_DEPTH);
        self.deadline = deadline;
        // Ignore timeout.
        _ = self.iterative_deepening(&eposition, max_depth, multi_move_threshold);
    }

    fn generate_root_captures_of_wazir(&mut self, position: &Position) {
        let score = ScoreExpanded::Win(position.ply() + 1).into();
        for mov in movegen::captures_of_wazir(position) {
            self.root_moves.push(RootMove {
                mov,
                score,
                nodes: 0,
            });
        }
        self.root_moves_considered = self.root_moves.len();
        self.root_moves_exact_score = self.root_moves.len();
    }

    fn generate_root_moves(&mut self, position: &Position) {
        for mov in movegen::moves(position) {
            self.root_moves.push(RootMove {
                mov,
                score: -Score::INFINITE,
                nodes: 0,
            });
        }
    }

    fn generate_root_suicides(&mut self, position: &Position) {
        let loss_ply = position.ply() + 2;
        let score = if loss_ply <= PLY_DRAW {
            ScoreExpanded::Loss(loss_ply).into()
        } else {
            Score::DRAW
        };

        for mov in movegen::pseudomoves(position) {
            self.root_moves.push(RootMove {
                mov,
                score,
                nodes: 0,
            });
        }
        self.root_moves_considered = self.root_moves.len();
        self.root_moves_exact_score = self.root_moves.len();
    }

    fn search_shallow(&mut self, eposition: &EvaluatedPosition<E>) {
        let hash = eposition.position().hash();
        self.history.push(hash);
        for move_idx in 0..self.root_moves.len() {
            let mov = self.root_moves[move_idx].mov;
            let epos2 = eposition.make_move(mov).unwrap();
            let nodes_start = self.nodes;
            let result = self
                .quiescence_search::<LongVariation>(&epos2, -Score::INFINITE, Score::INFINITE)
                .unwrap();
            let score = -result.score;
            let root_move = &mut self.root_moves[move_idx];
            root_move.nodes += self.nodes - nodes_start;
            root_move.score = score;
            if move_idx == 0 || score > self.root_moves[0].score {
                self.root_moves.swap(0, move_idx);
                self.pv = result.pv.add_front(mov).truncate();
            }
        }
        self.history.pop();
        self.depth = 1;
        self.root_moves_considered = self.root_moves.len();
        self.root_moves_exact_score = self.root_moves.len();
        self.sort_root_moves();
    }

    fn sort_root_moves(&mut self) {
        let (exact, other) = self.root_moves.split_at_mut(self.root_moves_exact_score);
        let (_, exact_other) = exact.split_first_mut().unwrap();
        exact_other.sort_by_key(|root_move| Reverse(root_move.score));
        other.sort_by_key(|root_move| Reverse(root_move.nodes));
    }

    fn iterative_deepening(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        max_depth: Depth,
        multi_move_threshold: Option<i32>,
    ) -> Result<(), Timeout> {
        let hash = eposition.position().hash();
        self.history.push(hash);
        while self.depth < max_depth {
            self.iterative_deepening_iteration(eposition, multi_move_threshold)?;
        }
        self.history.pop();
        Ok(())
    }

    fn iterative_deepening_iteration(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        multi_move_threshold: Option<i32>,
    ) -> Result<(), Timeout> {
        let mut completed_depth;
        // First move.
        {
            let next_depth = self.depth + 1;
            let mov = self.root_moves[0].mov;
            let epos2 = eposition.make_move(mov).unwrap();
            let nodes_start = self.nodes;
            let result = self.search_alpha_beta::<LongVariation>(
                &epos2,
                -Score::INFINITE,
                Score::INFINITE,
                next_depth - 1,
            )?;
            self.depth = next_depth;
            self.pv = result.pv.add_front(mov);
            let root_move = &mut self.root_moves[0];
            root_move.score = -result.score;
            root_move.nodes += self.nodes - nodes_start;
            completed_depth = result.depth.saturating_add(1);
            self.root_moves_considered = 1;
            self.root_moves_exact_score = 1;
        }

        // Other moves.
        while self.root_moves_considered < self.root_moves.len() {
            let mov = self.root_moves[self.root_moves_considered].mov;
            let epos2 = eposition.make_move(mov).unwrap();

            // PVS: Try null window first.
            let alpha = match multi_move_threshold {
                Some(multi_move_threshold) => self.root_moves[0]
                    .score
                    .offset(-multi_move_threshold)
                    .prev(),
                None => self.root_moves[0].score,
            };
            let nodes_start = self.nodes;
            let result_null_window = self.search_alpha_beta::<EmptyVariation>(
                &epos2,
                -alpha.next(),
                -alpha,
                self.depth - 1,
            )?;
            let score = -result_null_window.score;
            let root_move = &mut self.root_moves[self.root_moves_considered];
            root_move.nodes += self.nodes - nodes_start;
            root_move.score = score;

            if score > alpha {
                // Full window search.
                let nodes_start = self.nodes;
                let result = self.search_alpha_beta::<LongVariation>(
                    &epos2,
                    -Score::INFINITE,
                    -alpha,
                    self.depth - 1,
                )?;
                let score = -result.score;
                let root_move = &mut self.root_moves[self.root_moves_considered];
                root_move.nodes += self.nodes - nodes_start;
                root_move.score = score;
                completed_depth = completed_depth.min(result.depth.saturating_add(1));
                if score > alpha {
                    self.root_moves
                        .swap(self.root_moves_exact_score, self.root_moves_considered);
                    if score > self.root_moves[0].score {
                        self.root_moves.swap(0, self.root_moves_exact_score);
                        self.pv = result.pv.add_front(mov);
                    }
                    self.root_moves_exact_score += 1;
                }
            } else {
                completed_depth = completed_depth.min(result_null_window.depth.saturating_add(1));
            }
            self.root_moves_considered += 1;
        }
        self.depth = completed_depth;
        self.sort_root_moves();
        Ok(())
    }

    /// Recursive search function.
    fn search_alpha_beta<V: ExtendableVariation>(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: Depth,
    ) -> Result<SearchResultInternal<V>, Timeout> {
        if depth == 0 {
            let result = self.quiescence_search::<V>(eposition, alpha, beta)?;
            return Ok(SearchResultInternal {
                score: result.score,
                depth: 0,
                pv: result.pv,
                repetition_ply: Ply::MAX,
            });
        }

        self.new_node()?;
        let position = eposition.position();
        let ply = position.ply();

        // Prune guaranteed draws or endgames (including lower/upper bounds)
        let earliest_win = ply + 3; // if we deliver checkmate this move
        let best_possible = if earliest_win > PLY_DRAW {
            Score::DRAW
        } else {
            ScoreExpanded::Win(earliest_win).into()
        };

        let in_check = movegen::in_check(position, position.to_move());
        let earliest_loss = if in_check {
            ply + 2 // if we are already checkmated
        } else {
            ply + 4 // if we get checkmated next move (ignore zugzwang)
        };
        let worst_possible = if earliest_loss > PLY_DRAW {
            Score::DRAW
        } else {
            ScoreExpanded::Loss(earliest_loss).into()
        };

        if best_possible == worst_possible || best_possible <= alpha {
            return Ok(SearchResultInternal {
                score: best_possible,
                depth: Depth::MAX,
                pv: V::empty_truncated(),
                repetition_ply: Ply::MAX,
            });
        }

        if worst_possible >= beta {
            return Ok(SearchResultInternal {
                score: worst_possible,
                depth: Depth::MAX,
                pv: V::empty_truncated(),
                repetition_ply: Ply::MAX,
            });
        }

        // Check for repetition.
        let hash = position.hash();
        if let Some(repetition_ply) = self.history.find(hash) {
            return Ok(SearchResultInternal {
                score: Score::DRAW,
                depth: Depth::MAX,
                pv: V::empty_truncated(),
                repetition_ply,
            });
        }

        // Transposition table lookup.
        let mut tt_move = None;
        if depth >= self.hyperparameters.min_depth_ttable {
            if let Some(ttentry) = self.ttable.get(hash) {
                // Transposition table cutoff.
                if ttentry.depth >= depth {
                    let score = ttentry.score.to_absolute(ply);
                    let cutoff = match ttentry.score_type {
                        TTableScoreType::None => false,
                        TTableScoreType::Exact => true,
                        TTableScoreType::LowerBound => score >= beta,
                        TTableScoreType::UpperBound => score <= alpha,
                    };
                    if cutoff {
                        let mut pv = V::empty_truncated();
                        if ttentry.score_type == TTableScoreType::Exact {
                            if let Some(v) = V::pvtable_get(self.pvtable, hash) {
                                pv = v;
                            }
                        }
                        return Ok(SearchResultInternal {
                            score,
                            depth: ttentry.depth,
                            pv,
                            repetition_ply: Ply::MAX,
                        });
                    }
                }
                tt_move = ttentry.mov;
            }
        }

        // Search deeper.
        self.history.push(hash);
        // Search with V::Extended so that we have a TT move.
        let result = self.search_alpha_beta_deeper::<V::Extended>(
            eposition, alpha, beta, depth, in_check, tt_move,
        )?;
        self.history.pop();
        let mov = result.pv.first();
        let pv = result.pv.truncate();

        // Store killer move if beta cutoff and not a capture.
        if result.score >= beta {
            if let Some(mov) = mov {
                if mov.captured.is_none() {
                    let killer_moves = &mut self.killer_moves[ply as usize];
                    let index = (0..NUM_KILLER_MOVES - 1)
                        .find(|&index| killer_moves[index] == Some(mov))
                        .unwrap_or(NUM_KILLER_MOVES - 1);
                    killer_moves[index] = Some(mov);
                    killer_moves[0..=index].rotate_right(1);
                }
            }
        }

        // Save in transposition table.
        if depth >= self.hyperparameters.min_depth_ttable {
            let score_type = if result.score >= beta {
                if result.repetition_ply >= ply || result.score > Score::DRAW {
                    TTableScoreType::LowerBound
                } else {
                    TTableScoreType::None
                }
            } else if result.score <= alpha {
                if result.repetition_ply >= ply || result.score < Score::DRAW {
                    TTableScoreType::UpperBound
                } else {
                    TTableScoreType::None
                }
            } else if result.repetition_ply >= ply {
                TTableScoreType::Exact
            } else if result.score < Score::DRAW {
                TTableScoreType::UpperBound
            } else if result.score > Score::DRAW {
                TTableScoreType::LowerBound
            } else {
                TTableScoreType::None
            };

            if score_type == TTableScoreType::Exact {
                V::pvtable_set(self.pvtable, hash, pv.clone());
            }
            if mov.is_some() || score_type != TTableScoreType::None {
                self.ttable.set(
                    hash,
                    TTableEntry {
                        depth: result.depth,
                        mov,
                        score_type,
                        score: result.score.to_relative(ply),
                    },
                );
            }
        }

        Ok(SearchResultInternal {
            score: result.score,
            depth: result.depth,
            pv,
            repetition_ply: result.repetition_ply,
        })
    }

    // No early cutoff, we have search deeper.
    fn search_alpha_beta_deeper<V: NonEmptyVariation>(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: Depth,
        in_check: bool,
        tt_move: Option<Move>,
    ) -> Result<SearchResultInternal<V>, Timeout> {
        let position = eposition.position();

        // ZFastest loss is at ply+2 if we are checkmated.
        let mut result = SearchResultInternal {
            score: ScoreExpanded::Loss(position.ply() + 2).into(),
            depth: Depth::MAX,
            pv: V::empty_truncated(),
            repetition_ply: Ply::MAX,
        };

        let tt_move = tt_move.into_iter().map(InternalMove::extra);

        let moves = if in_check {
            Either::Left(tt_move.chain(movegen::check_evasions(position).map(InternalMove::new)))
        } else {
            let null_move = if depth >= self.hyperparameters.reduction_null_move
                && self.history.last_cut() != Some(position.ply())
            {
                Some(InternalMove::Null)
            } else {
                None
            }
            .into_iter();

            let captures_and_checks = movegen::captures_checks(position)
                .chain(movegen::captures_non_checks(position))
                .chain(movegen::jumps_checks(position))
                .chain(movegen::drops_checks(position))
                .map(InternalMove::new);

            let killers = self.killer_moves[position.ply() as usize]
                .into_iter()
                .flatten()
                .map(InternalMove::extra);

            let quiet_moves = movegen::jumps_non_checks(position)
                .chain(movegen::drops_non_checks(position))
                .map(InternalMove::new);

            Either::Right(
                null_move
                    .chain(tt_move)
                    .chain(captures_and_checks)
                    .chain(killers)
                    .chain(quiet_moves),
            )
        };

        let mut extra_moves = SmallVec::<Move, { 1 + NUM_KILLER_MOVES }>::new();

        for internal_move in moves {
            match internal_move {
                InternalMove::Move { mov, extra } => {
                    if extra_moves.contains(&mov) {
                        continue;
                    }

                    let Ok(epos2) = eposition.make_move(mov) else {
                        // Illegal move. Could be a hash collision in the transposition table
                        // or invalid killer move.
                        if extra {
                            continue;
                        } else {
                            panic!("Illegal move in search_alpha_beta_deeper");
                        }
                    };

                    if extra {
                        if movegen::in_check(epos2.position(), position.to_move()) {
                            // Skip suicide move.
                            continue;
                        }
                        extra_moves.push(mov);
                    }

                    let alpha2 = alpha.max(result.score);

                    // Try null window first.
                    let result_null_window = if beta > alpha2.next() {
                        self.search_alpha_beta::<EmptyVariation>(
                            &epos2,
                            -alpha2.next(),
                            -alpha2,
                            depth.saturating_sub(1),
                        )?
                    } else {
                        SearchResultInternal::<EmptyVariation> {
                            score: -Score::INFINITE,
                            depth: Depth::MAX,
                            pv: EmptyVariation::empty_truncated(),
                            repetition_ply: Ply::MAX,
                        }
                    };

                    if -result_null_window.score > alpha2 {
                        let result2 = self.search_alpha_beta::<V::Truncated>(
                            &epos2,
                            -beta,
                            -alpha2,
                            depth.saturating_sub(1),
                        )?;
                        let score = -result2.score;
                        if score > result.score {
                            result.score = score;
                            if score > alpha {
                                result.pv = result2.pv.add_front(mov);
                            }
                            if score >= beta {
                                result.depth = result2.depth.saturating_add(1);
                                result.repetition_ply = result2.repetition_ply;
                                break;
                            }
                        }
                        result.depth = result.depth.min(result2.depth.saturating_add(1));
                        result.repetition_ply = result.repetition_ply.min(result2.repetition_ply);
                    } else {
                        result.depth = result.depth.min(result_null_window.depth.saturating_add(1));
                        result.repetition_ply =
                            result.repetition_ply.min(result_null_window.repetition_ply);
                    }
                }
                InternalMove::Null => {
                    self.history.cut();
                    let epos2 = eposition.make_null_move().unwrap();
                    let result2 = self.search_alpha_beta::<EmptyVariation>(
                        &epos2,
                        -beta,
                        -beta.prev(),
                        depth - self.hyperparameters.reduction_null_move,
                    )?;
                    self.history.uncut();
                    if -result2.score >= beta {
                        return Ok(SearchResultInternal {
                            score: beta,
                            depth: result2
                                .depth
                                .saturating_add(self.hyperparameters.reduction_null_move),
                            pv: V::empty_truncated(),
                            // Repetitions don't count accross null move.
                            repetition_ply: Ply::MAX,
                        });
                    }
                }
            }
        }

        Ok(result)
    }

    /// Quiescence search.
    fn quiescence_search<V: ExtendableVariation>(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
    ) -> Result<SearchResultQuiescence<V>, Timeout> {
        self.new_node()?;

        // Assume we're not going to checkmate the opponent in quiescence.
        if alpha >= Score::WIN_MAX_PLY {
            return Ok(SearchResultQuiescence {
                score: Score::WIN_MAX_PLY,
                pv: V::empty_truncated(),
            });
        }

        let position = eposition.position();
        let ply = position.ply();
        let in_check = movegen::in_check(position, position.to_move());

        let mut result;
        let moves;

        if in_check {
            // Fastest loss is at ply+2 if we are checkmated.
            // Fastest win is at ply+3 (checkmate in 1).
            if ply + 2 > PLY_DRAW || ply + 3 > PLY_DRAW && alpha >= Score::DRAW {
                return Ok(SearchResultQuiescence {
                    score: Score::DRAW,
                    pv: V::empty_truncated(),
                });
            }

            // If we are checkmated, we lose in 2 ply.
            result = SearchResultQuiescence {
                score: ScoreExpanded::Loss(position.ply() + 2).into(),
                pv: V::empty_truncated(),
            };
            moves = Either::Left(movegen::check_evasions(position));
        } else {
            // Fastest win is at ply+3 (checkmate in 1).
            // Fastest loss is at ply+4 (we get checkmated next move).
            if ply + 3 > PLY_DRAW || ply + 4 > PLY_DRAW && beta <= Score::DRAW {
                return Ok(SearchResultQuiescence {
                    score: Score::DRAW,
                    pv: V::empty_truncated(),
                });
            }

            // We can at least stand pat, no need to eval.
            if beta <= -Score::WIN_MAX_PLY {
                return Ok(SearchResultQuiescence {
                    score: -Score::WIN_MAX_PLY,
                    pv: V::empty_truncated(),
                });
            }

            // Stand pat.
            result = SearchResultQuiescence {
                score: ScoreExpanded::Eval(eposition.evaluate()).into(),
                pv: V::empty(),
            };
            if result.score >= beta {
                return Ok(result);
            }
            moves = Either::Right(
                movegen::captures_checks(eposition.position())
                    .chain(movegen::captures_non_checks(eposition.position())),
            );
        }

        for mov in moves {
            let epos2 = eposition
                .make_move(mov)
                .expect("Illegal move in quiescence search");

            let alpha2 = alpha.max(result.score);
            let result2 = self.quiescence_search::<V>(&epos2, -beta, -alpha2)?;
            let score = -result2.score;
            if score > result.score {
                result.score = score;
                if score > alpha {
                    result.pv = result2.pv.add_front(mov).truncate();
                }
                if score >= beta {
                    break;
                }
            }
        }

        Ok(result)
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
    // Only used for multi-move searches.
    pub top_moves: Vec<ScoredMove>,
    pub depth: Depth,
    pub root_moves_considered: usize,
    pub num_root_moves: usize,
    pub nodes: u64,
}

pub struct ScoredMove {
    pub mov: Move,
    pub score: Score,
}

struct RootMove {
    mov: Move,
    score: Score,
    nodes: u64,
}

enum InternalMove {
    Move { mov: Move, extra: bool },
    Null,
}

impl InternalMove {
    fn new(mov: Move) -> Self {
        Self::Move { mov, extra: false }
    }

    fn extra(mov: Move) -> Self {
        Self::Move { mov, extra: true }
    }
}

struct SearchResultInternal<V> {
    score: Score,
    depth: Depth,
    pv: V,
    // Smallest ply when the position was repeated
    repetition_ply: Ply,
}

struct SearchResultQuiescence<V> {
    score: Score,
    pv: V,
}

#[derive(Debug)]
struct Timeout;
