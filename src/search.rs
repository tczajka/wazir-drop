use crate::{
    constants::{
        Depth, Eval, Hyperparameters, Ply, CHECK_TIMEOUT_NODES, DEPTH_INCREMENT, MAX_SEARCH_DEPTH,
        NUM_KILLER_MOVES, ONE_PLY, PLY_DRAW,
    },
    either::Either,
    history::History,
    log, movegen,
    smallvec::SmallVec,
    ttable::{TTable, TTableEntry, TTableScoreType},
    variation::LongVariation,
    Color, EmptyVariation, EvaluatedPosition, Evaluator, ExtendableVariation, Move,
    NonEmptyVariation, OneMoveVariation, PVTable, Position, Score, ScoreExpanded, SetupMove, Stage,
    Variation,
};
use std::{cmp::Reverse, iter, sync::Arc, time::Instant};

pub struct Search<E> {
    hyperparameters: Hyperparameters,
    evaluator: Arc<E>,
    ttable: TTable,
    pvtable: PVTable,
    killer_moves: Vec<[Option<Move>; NUM_KILLER_MOVES]>,
}

#[derive(Debug, Copy, Clone)]
pub struct Deadlines {
    pub hard: Instant,
    pub soft: Instant,
    pub start_next_depth: Instant,
    pub panic_hard: Instant,
    pub panic_soft: Instant,
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
        deadlines: Option<Deadlines>,
        multi_move_threshold: Option<i32>,
        is_score_important: bool,
        history: &History,
    ) -> SearchResult {
        let mut instance = SearchInstance::new(
            self,
            position,
            max_depth,
            deadlines,
            multi_move_threshold,
            history,
        );
        instance.search(is_score_important)
    }

    pub fn search_blue_setup(
        &mut self,
        red: SetupMove,
        max_depth: Option<Depth>,
        deadlines: Option<Deadlines>,
        possible_moves: &[SetupMove],
    ) -> SearchResultBlueSetup {
        let mut position = Position::initial();
        let mut history = History::new(position.hash());
        position = position.make_setup_move(red).unwrap();
        history.push_irreversible(position.hash());
        let mut instance =
            SearchInstance::new(self, &position, max_depth, deadlines, None, &history);
        instance.search_blue_setup(possible_moves)
    }
}

/// This doesn't work for setup positions.
struct SearchInstance<'a, E: Evaluator> {
    hyperparameters: Hyperparameters,
    evaluator: &'a E,
    ttable: &'a mut TTable,
    pvtable: &'a mut PVTable,
    killer_moves: &'a mut [[Option<Move>; NUM_KILLER_MOVES]],
    root_position: Position,
    max_depth: Depth,
    deadlines: Option<Deadlines>,
    multi_move_threshold: Option<i32>,
    hard_deadline: Option<Instant>,
    nodes: u64,
    root_moves: Vec<RootMove>,
    root_moves_setup: Vec<SetupMove>,
    depth: Depth,
    root_moves_considered: usize,
    root_moves_exact_score: usize,
    pv: LongVariation,
    history: History,
    blue_setup_score: Score,
    red_contempt: Eval,
}

impl<'a, E: Evaluator> SearchInstance<'a, E> {
    fn new(
        search: &'a mut Search<E>,
        position: &Position,
        max_depth: Option<Depth>,
        deadlines: Option<Deadlines>,
        multi_move_threshold: Option<i32>,
        history: &History,
    ) -> Self {
        assert!(multi_move_threshold.is_none() || deadlines.is_none());
        let contempt = (search.hyperparameters.contempt * search.evaluator.scale()) as Eval;
        let red_contempt = match position.to_move() {
            Color::Red => contempt,
            Color::Blue => -contempt,
        };
        Self {
            hyperparameters: search.hyperparameters.clone(),
            evaluator: &search.evaluator,
            ttable: &mut search.ttable,
            pvtable: &mut search.pvtable,
            killer_moves: &mut search.killer_moves,
            root_position: position.clone(),
            max_depth: max_depth.unwrap_or(MAX_SEARCH_DEPTH),
            deadlines,
            multi_move_threshold,
            hard_deadline: None,
            nodes: 0,
            root_moves: Vec::new(),
            root_moves_setup: Vec::new(),
            depth: 0,
            root_moves_considered: 0,
            root_moves_exact_score: 0,
            pv: LongVariation::empty(),
            history: history.clone(),
            blue_setup_score: Score::DRAW,
            red_contempt,
        }
    }

    fn search(&mut self, is_score_important: bool) -> SearchResult {
        let score = match self.root_position.stage() {
            Stage::Setup => panic!("SearchInstance::search does not support setup"),
            Stage::Regular => {
                self.search_root(is_score_important);
                self.root_moves[0].score
            }
            Stage::End(outcome) => outcome.to_score(self.root_position.ply()),
        };

        let top_moves = match self.multi_move_threshold {
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

    fn search_root(&mut self, is_score_important: bool) {
        self.generate_root_captures_of_wazir();
        if let Some(root_move) = self.root_moves.first() {
            self.depth = Depth::MAX;
            self.pv = LongVariation::empty().add_front(root_move.mov);
            return;
        }

        self.generate_root_moves();

        if self.root_moves.is_empty() {
            self.generate_root_suicides();
            let Some(root_move) = self.root_moves.first() else {
                panic!("Stalemate");
            };
            self.depth = Depth::MAX;
            self.pv = LongVariation::empty().add_front(root_move.mov);
            return;
        }

        if self.root_moves.len() == 1 && !is_score_important {
            log::info!("only one choice");
            self.depth = 0;
            self.root_moves_considered = 1;
            self.root_moves_exact_score = 1;
            self.pv = LongVariation::empty_truncated().add_front(self.root_moves[0].mov);
            return;
        }

        self.ttable.new_epoch();
        self.pvtable.new_epoch();

        let eposition = EvaluatedPosition::new(self.evaluator, self.root_position.clone());

        // Ignore timeout.
        _ = self.iterative_deepening(&eposition);
    }

    fn generate_root_captures_of_wazir(&mut self) {
        let score = ScoreExpanded::Win(self.root_position.ply() + 1).into();
        for mov in movegen::captures_of_wazir(&self.root_position) {
            self.root_moves.push(RootMove {
                mov,
                score,
                futile: false,
            });
        }
        self.root_moves_considered = self.root_moves.len();
        self.root_moves_exact_score = self.root_moves.len();
    }

    fn generate_root_moves(&mut self) {
        let in_check = movegen::in_check(&self.root_position, self.root_position.to_move());
        let mut futile = false;
        for move_candidate in
            self.generate_move_candidates(&self.root_position, in_check, false, None, false)
        {
            match move_candidate {
                MoveCandidate::Move { mov, extra: _extra } => {
                    self.root_moves.push(RootMove {
                        mov,
                        score: Score::DRAW,
                        futile,
                    });
                }
                MoveCandidate::Futility => {
                    futile = true;
                }
                MoveCandidate::Null => {
                    panic!("Null move in root moves");
                }
            }
        }
    }

    fn generate_root_suicides(&mut self) {
        let loss_ply = self.root_position.ply() + 2;
        let score = if loss_ply <= PLY_DRAW {
            ScoreExpanded::Loss(loss_ply).into()
        } else {
            Score::DRAW
        };

        for mov in movegen::pseudomoves(&self.root_position) {
            self.root_moves.push(RootMove {
                mov,
                score,
                futile: false,
            });
        }
        self.root_moves_considered = self.root_moves.len();
        self.root_moves_exact_score = self.root_moves.len();
    }

    fn iterative_deepening(&mut self, eposition: &EvaluatedPosition<E>) -> Result<(), Timeout> {
        // In case we can't finish depth 1 search for a single move, use the first generated move.
        self.pv = LongVariation::empty().add_front(self.root_moves[0].mov);
        self.search_shallow(eposition)?;
        while self.depth < self.max_depth {
            if let Some(ds) = self.deadlines.as_ref() {
                if Instant::now() >= ds.start_next_depth {
                    log::info!("ndto"); // next depth timeout
                    break;
                }
            }
            self.iterative_deepening_iteration(eposition)?;
        }
        Ok(())
    }

    fn search_shallow(&mut self, eposition: &EvaluatedPosition<E>) -> Result<(), Timeout> {
        self.hard_deadline = self.deadlines.as_ref().map(|ds| ds.hard);
        self.depth = ONE_PLY;
        self.root_moves_considered = 0;
        self.root_moves_exact_score = 0;
        while self.root_moves_considered < self.root_moves.len() {
            if let Some(ds) = self.deadlines.as_ref() {
                if Instant::now() >= ds.soft {
                    log::info!("ssto"); // shallow soft timeout
                    return Err(Timeout);
                }
            }
            let mov = self.root_moves[self.root_moves_considered].mov;
            let epos2 = eposition.make_move(mov).unwrap();
            self.history.push(epos2.position().hash());
            let result = self.search_alpha_beta::<LongVariation>(
                &epos2,
                -Score::INFINITE,
                Score::INFINITE,
                0,
                NodeType::PV,
            )?;
            self.history.pop();
            let score = -result.score;
            let root_move = &mut self.root_moves[self.root_moves_considered];
            root_move.score = score;
            if self.root_moves_considered == 0 || score > self.root_moves[0].score {
                self.root_moves[0..=self.root_moves_considered].rotate_right(1);
                self.pv = result.pv.add_front(mov).truncate();
            }
            self.root_moves_considered += 1;
            self.root_moves_exact_score = self.root_moves_considered;
        }
        self.sort_root_moves();
        Ok(())
    }

    fn sort_root_moves(&mut self) {
        self.root_moves[1..self.root_moves_exact_score]
            .sort_by_key(|root_move| Reverse(root_move.score));
    }

    fn iterative_deepening_iteration(
        &mut self,
        eposition: &EvaluatedPosition<E>,
    ) -> Result<(), Timeout> {
        let mut completed_depth;
        let panic_threshold = match ScoreExpanded::from(self.root_moves[0].score) {
            ScoreExpanded::Win(_) => Score::WIN_MAX_PLY,
            ScoreExpanded::Loss(_) => -Score::INFINITE,
            ScoreExpanded::Eval(eval) => ScoreExpanded::Eval(
                eval - (self.evaluator.scale() * self.hyperparameters.panic_eval_threshold) as Eval,
            )
            .into(),
        };
        // First move.
        {
            self.hard_deadline = self.deadlines.as_ref().map(|ds| ds.hard);
            let next_depth = self.depth + DEPTH_INCREMENT;
            let depth_diff = ONE_PLY;
            let mov = self.root_moves[0].mov;
            let epos2 = eposition.make_move(mov).unwrap();
            self.history.push(epos2.position().hash());
            let result = self.search_alpha_beta::<LongVariation>(
                &epos2,
                -Score::INFINITE,
                Score::INFINITE,
                next_depth.saturating_sub(depth_diff),
                NodeType::PV,
            )?;
            self.history.pop();
            self.depth = next_depth;
            self.pv = result.pv.add_front(mov);
            self.root_moves[0].score = -result.score;
            completed_depth = result.depth.saturating_add(depth_diff);
            self.root_moves_considered = 1;
            self.root_moves_exact_score = 1;
        }

        // Other moves.
        while self.root_moves_considered < self.root_moves.len() {
            if let Some(ds) = self.deadlines.as_ref() {
                let is_panic = self.root_moves[0].score < panic_threshold;
                let soft_deadline = if is_panic { ds.panic_soft } else { ds.soft };
                if Instant::now() >= soft_deadline {
                    log::info!("sto"); // soft timeout
                    return Err(Timeout);
                }
                self.hard_deadline = Some(if is_panic { ds.panic_hard } else { ds.hard });
            } else {
                self.hard_deadline = None;
            }

            let mov = self.root_moves[self.root_moves_considered].mov;
            let epos2 = eposition.make_move(mov).unwrap();
            self.history.push(epos2.position().hash());

            let alpha = match self.multi_move_threshold {
                Some(multi_move_threshold) => self.root_moves[0]
                    .score
                    .offset(-multi_move_threshold)
                    .prev(),
                None => self.root_moves[0].score,
            };

            // Late move reduction.
            if self.root_moves_considered >= self.hyperparameters.late_move_reduction_start
                && self.root_moves[self.root_moves_considered].futile
            {
                let depth_diff = 2 * ONE_PLY;
                let result = self.search_alpha_beta::<EmptyVariation>(
                    &epos2,
                    -alpha.next(),
                    -alpha,
                    self.depth.saturating_sub(depth_diff),
                    NodeType::Cut,
                )?;
                let score = -result.score;
                self.root_moves[self.root_moves_considered].score = score;
                if score <= alpha {
                    completed_depth = completed_depth.min(result.depth.saturating_add(depth_diff));
                    self.root_moves_considered += 1;
                    self.history.pop();
                    continue;
                }
            }

            let depth_diff = ONE_PLY;

            // Null window.
            let result = self.search_alpha_beta::<EmptyVariation>(
                &epos2,
                -alpha.next(),
                -alpha,
                self.depth.saturating_sub(depth_diff),
                NodeType::Cut,
            )?;
            let score = -result.score;
            self.root_moves[self.root_moves_considered].score = score;

            if score <= alpha {
                completed_depth = completed_depth.min(result.depth.saturating_add(depth_diff));
                self.root_moves_considered += 1;
                self.history.pop();
                continue;
            }

            // Full window search.
            let result = self.search_alpha_beta::<LongVariation>(
                &epos2,
                -Score::INFINITE,
                -alpha,
                self.depth.saturating_sub(depth_diff),
                NodeType::PV,
            )?;
            self.history.pop();
            let score = -result.score;
            self.root_moves[self.root_moves_considered].score = score;
            completed_depth = completed_depth.min(result.depth.saturating_add(depth_diff));
            if score > alpha {
                self.root_moves[self.root_moves_exact_score..=self.root_moves_considered]
                    .rotate_right(1);
                self.root_moves_exact_score += 1;
                if score > self.root_moves[0].score {
                    self.root_moves[0..self.root_moves_exact_score].rotate_right(1);
                    self.pv = result.pv.add_front(mov);
                }
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
        node_type: NodeType,
    ) -> Result<SearchResultInternal<V>, Timeout> {
        let position = eposition.position();
        let ply = position.ply();
        assert_eq!(self.history.ply(), ply);

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
        if let Some(repetition_ply) = self.history.find_repetition() {
            let repetition_ply = if repetition_ply <= self.root_position.ply() {
                Ply::MAX
            } else {
                repetition_ply
            };
            return Ok(SearchResultInternal {
                score: Score::DRAW,
                depth: Depth::MAX,
                pv: V::empty_truncated(),
                repetition_ply,
            });
        }

        if depth == 0 {
            return self.quiescence_search::<V>(eposition, alpha, beta);
        }

        self.new_node()?;

        // Transposition table lookup.
        let mut tt_move = None;
        let hash = position.hash();
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
        // Search with V::Extended so that we have a TT move.
        let result = self.search_alpha_beta_deeper::<V::Extended>(
            eposition, alpha, beta, depth, in_check, tt_move, node_type,
        )?;
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
    #[allow(clippy::too_many_arguments)]
    fn search_alpha_beta_deeper<V: NonEmptyVariation>(
        &mut self,
        eposition: &EvaluatedPosition<E>,
        alpha: Score,
        beta: Score,
        depth: Depth,
        in_check: bool,
        mut tt_move: Option<Move>,
        node_type: NodeType,
    ) -> Result<SearchResultInternal<V>, Timeout> {
        let position = eposition.position();

        // Fastest loss is at ply+2 if we are checkmated.
        let mut result = SearchResultInternal {
            score: ScoreExpanded::Loss(position.ply() + 2).into(),
            depth: Depth::MAX,
            pv: V::empty_truncated(),
            repetition_ply: Ply::MAX,
        };

        // Internal iterative deepening.
        if depth >= self.hyperparameters.iid_min_depth
            && matches!(node_type, NodeType::PV | NodeType::Cut)
            && tt_move.is_none()
        {
            let result = self.search_alpha_beta_deeper::<OneMoveVariation>(
                eposition,
                alpha,
                beta,
                depth - self.hyperparameters.iid_reduction,
                in_check,
                None,
                node_type,
            )?;
            tt_move = result.pv.first();
        }

        let move_candidates = self.generate_move_candidates(
            position,
            in_check,
            depth >= self.hyperparameters.reduction_null_move
                && !self.history.last_move_irreversible(),
            tt_move,
            true,
        );

        let mut extra_moves = SmallVec::<Move, { 1 + NUM_KILLER_MOVES }>::new();

        let mut move_index = 0;
        let mut enable_late_move_reduction = false;

        let mut lazy_eval: Option<Eval> = None;

        for move_candidate in move_candidates {
            match move_candidate {
                MoveCandidate::Move { mov, extra } => {
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

                    self.history.push(epos2.position().hash());
                    let cur_move_index = move_index;
                    move_index += 1;
                    let alpha2 = alpha.max(result.score);

                    // Try late move first.
                    if enable_late_move_reduction
                        && cur_move_index >= self.hyperparameters.late_move_reduction_start
                    {
                        let depth_diff = 2 * ONE_PLY;
                        let depth2 = depth.saturating_sub(depth_diff);
                        let result2 = self.search_alpha_beta::<V::Truncated>(
                            &epos2,
                            -alpha2.next(),
                            -alpha2,
                            depth2,
                            NodeType::Cut,
                        )?;
                        if -result2.score <= alpha2 {
                            result.depth =
                                result.depth.min(result2.depth.saturating_add(depth_diff));
                            result.repetition_ply =
                                result.repetition_ply.min(result2.repetition_ply);
                            self.history.pop();
                            continue;
                        }
                    }

                    let depth_diff = ONE_PLY;
                    let depth2 = depth.saturating_sub(depth_diff);

                    // Try null window.
                    if node_type == NodeType::PV && cur_move_index != 0 {
                        let result2 = self.search_alpha_beta::<EmptyVariation>(
                            &epos2,
                            -alpha2.next(),
                            -alpha2,
                            depth2,
                            NodeType::Cut,
                        )?;
                        if -result2.score <= alpha2 {
                            result.depth =
                                result.depth.min(result2.depth.saturating_add(depth_diff));
                            result.repetition_ply =
                                result.repetition_ply.min(result2.repetition_ply);
                            self.history.pop();
                            continue;
                        }
                    };

                    // Proper search.
                    let node_type2 = match node_type {
                        NodeType::PV => NodeType::PV,
                        NodeType::Cut if cur_move_index == 0 => NodeType::All,
                        _ => NodeType::Cut,
                    };
                    let result2 = self.search_alpha_beta::<V::Truncated>(
                        &epos2, -beta, -alpha2, depth2, node_type2,
                    )?;
                    self.history.pop();
                    let score = -result2.score;
                    let depth_actual = result2.depth.saturating_add(depth_diff);
                    if score > result.score {
                        result.score = score;
                        if score > alpha {
                            result.pv = result2.pv.add_front(mov);
                        }
                        if score >= beta {
                            result.depth = depth_actual;
                            result.repetition_ply = result2.repetition_ply;
                            break;
                        }
                    }
                    result.depth = result.depth.min(depth_actual);
                    result.repetition_ply = result.repetition_ply.min(result2.repetition_ply);
                }
                MoveCandidate::Null => {
                    let do_null_move = match ScoreExpanded::from(beta) {
                        ScoreExpanded::Win(_) => false,
                        ScoreExpanded::Loss(_) => true,
                        ScoreExpanded::Eval(beta_eval) => {
                            if lazy_eval.is_none() {
                                lazy_eval = Some(eposition.evaluate());
                            }
                            lazy_eval.unwrap()
                                >= beta_eval
                                    + (self.hyperparameters.null_move_margin
                                        * self.evaluator.scale())
                                        as Eval
                        }
                    };
                    if !do_null_move {
                        continue;
                    }
                    let epos2 = eposition.make_null_move().unwrap();
                    self.history.push_irreversible(epos2.position().hash());
                    let depth_diff = ONE_PLY + self.hyperparameters.reduction_null_move;
                    let result2 = self.search_alpha_beta::<EmptyVariation>(
                        &epos2,
                        -beta,
                        -beta.prev(),
                        depth.saturating_sub(depth_diff),
                        NodeType::Cut,
                    )?;
                    self.history.pop();
                    if -result2.score >= beta {
                        return Ok(SearchResultInternal {
                            score: beta,
                            depth: result2.depth.saturating_add(depth_diff),
                            pv: V::empty_truncated(),
                            // Repetitions don't count accross null move.
                            repetition_ply: Ply::MAX,
                        });
                    }
                }
                MoveCandidate::Futility => {
                    if depth <= ONE_PLY {
                        let futile = match ScoreExpanded::from(alpha) {
                            ScoreExpanded::Win(_) => true,
                            ScoreExpanded::Loss(_) => false,
                            ScoreExpanded::Eval(alpha_eval) => {
                                if lazy_eval.is_none() {
                                    lazy_eval = Some(eposition.evaluate());
                                }
                                let margin = (self.hyperparameters.futility_margin
                                    * self.evaluator.scale())
                                    as Eval;
                                lazy_eval.unwrap() <= alpha_eval - margin
                            }
                        };
                        if futile {
                            return Ok(SearchResultInternal {
                                score: alpha,
                                depth,
                                pv: V::empty_truncated(),
                                repetition_ply: Ply::MAX,
                            });
                        }
                    } else {
                        enable_late_move_reduction = true;
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
    ) -> Result<SearchResultInternal<V>, Timeout> {
        self.new_node()?;

        // Assume we're not going to checkmate the opponent in quiescence.
        if alpha >= Score::WIN_MAX_PLY {
            return Ok(SearchResultInternal {
                score: Score::WIN_MAX_PLY,
                depth: 0,
                pv: V::empty_truncated(),
                repetition_ply: Ply::MAX,
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
                return Ok(SearchResultInternal {
                    score: Score::DRAW,
                    depth: 0,
                    pv: V::empty_truncated(),
                    repetition_ply: Ply::MAX,
                });
            }

            // If we are checkmated, we lose in 2 ply.
            result = SearchResultInternal {
                score: ScoreExpanded::Loss(position.ply() + 2).into(),
                depth: 0,
                pv: V::empty_truncated(),
                repetition_ply: Ply::MAX,
            };
            moves = Either::Left(movegen::check_evasions(position));
        } else {
            // Fastest win is at ply+3 (checkmate in 1).
            // Fastest loss is at ply+4 (we get checkmated next move).
            if ply + 3 > PLY_DRAW || ply + 4 > PLY_DRAW && beta <= Score::DRAW {
                return Ok(SearchResultInternal {
                    score: Score::DRAW,
                    depth: 0,
                    pv: V::empty_truncated(),
                    repetition_ply: Ply::MAX,
                });
            }

            // We can at least stand pat, no need to eval.
            if beta <= -Score::WIN_MAX_PLY {
                return Ok(SearchResultInternal {
                    score: -Score::WIN_MAX_PLY,
                    depth: 0,
                    pv: V::empty_truncated(),
                    repetition_ply: Ply::MAX,
                });
            }

            // Stand pat.
            let contempt = match position.to_move() {
                Color::Red => self.red_contempt,
                Color::Blue => -self.red_contempt,
            };
            let eval = eposition.evaluate() + contempt;
            result = SearchResultInternal {
                score: ScoreExpanded::Eval(eval).into(),
                depth: 0,
                pv: V::empty(),
                repetition_ply: Ply::MAX,
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
        if let Some(deadline) = self.hard_deadline {
            if self.nodes % CHECK_TIMEOUT_NODES == 0 && Instant::now() >= deadline {
                log::info!("hto"); // hard timeout
                return Err(Timeout);
            }
        }
        Ok(())
    }

    fn generate_move_candidates<'pos>(
        &self,
        position: &'pos Position,
        in_check: bool,
        use_null_move: bool,
        tt_move: Option<Move>,
        use_killers: bool,
    ) -> impl Iterator<Item = MoveCandidate> + 'pos {
        let tt_move = tt_move.into_iter().map(MoveCandidate::extra);
        if in_check {
            Either::Left(tt_move.chain(movegen::check_evasions(position).map(MoveCandidate::new)))
        } else {
            let null_move = if use_null_move {
                Some(MoveCandidate::Null)
            } else {
                None
            }
            .into_iter();

            let captures = movegen::captures_checks(position)
                .chain(movegen::captures_non_checks(position))
                .map(MoveCandidate::new);

            let futility = iter::once(MoveCandidate::Futility);

            let killers = if use_killers {
                Either::Left(
                    self.killer_moves[position.ply() as usize]
                        .into_iter()
                        .flatten()
                        .map(MoveCandidate::extra),
                )
            } else {
                Either::Right(iter::empty())
            };

            let checks = movegen::jumps_checks(position)
                .chain(movegen::drops_checks(position))
                .map(MoveCandidate::new);

            let quiet_moves = movegen::jumps_non_checks(position)
                .chain(movegen::drops_non_checks(position))
                .map(MoveCandidate::new);

            Either::Right(
                null_move
                    .chain(tt_move)
                    .chain(captures)
                    .chain(killers)
                    .chain(checks)
                    .chain(futility)
                    .chain(quiet_moves),
            )
        }
    }

    fn search_blue_setup(&mut self, possible_moves: &[SetupMove]) -> SearchResultBlueSetup {
        assert_eq!(self.root_position.stage(), Stage::Setup);
        assert_eq!(self.root_position.to_move(), Color::Blue);
        self.root_moves_setup = possible_moves.to_vec();
        self.ttable.new_epoch();
        self.pvtable.new_epoch();
        let eposition = EvaluatedPosition::new(self.evaluator, self.root_position.clone());
        _ = self.blue_setup_iterative_deepening(&eposition);
        SearchResultBlueSetup {
            score: self.blue_setup_score,
            mov: self.root_moves_setup[0],
            pv: self.pv.clone(),
            depth: self.depth,
            root_moves_considered: self.root_moves_considered,
            num_root_moves: self.root_moves_setup.len(),
            nodes: self.nodes,
        }
    }

    fn blue_setup_iterative_deepening(
        &mut self,
        eposition: &EvaluatedPosition<E>,
    ) -> Result<(), Timeout> {
        self.hard_deadline = self.deadlines.as_ref().map(|ds| ds.hard);
        while self.depth < self.max_depth {
            if let Some(ds) = self.deadlines.as_ref() {
                if Instant::now() >= ds.start_next_depth {
                    log::info!("ndto"); // next depth timeout
                    break;
                }
            }
            self.blue_setup_iterative_deepening_iteration(eposition)?;
        }
        Ok(())
    }

    fn blue_setup_iterative_deepening_iteration(
        &mut self,
        eposition: &EvaluatedPosition<E>,
    ) -> Result<(), Timeout> {
        // First move.
        let next_depth = self.depth + DEPTH_INCREMENT;
        let mov = self.root_moves_setup[0];
        let epos2 = eposition.make_setup_move(mov).unwrap();
        self.history.push_irreversible(epos2.position().hash());
        let result = self.search_alpha_beta::<LongVariation>(
            &epos2,
            -Score::INFINITE,
            Score::INFINITE,
            next_depth.saturating_sub(ONE_PLY),
            NodeType::PV,
        )?;
        self.history.pop();
        self.depth = next_depth;
        self.pv = result.pv;
        self.blue_setup_score = -result.score;
        self.root_moves_considered = 1;

        while self.root_moves_considered < self.root_moves_setup.len() {
            if let Some(ds) = self.deadlines.as_ref() {
                if Instant::now() >= ds.soft {
                    log::info!("sto"); // next depth timeout
                    break;
                }
            }
            let mov = self.root_moves_setup[self.root_moves_considered];
            let epos2 = eposition.make_setup_move(mov).unwrap();
            self.history.push_irreversible(epos2.position().hash());
            let alpha = self.blue_setup_score;
            // Null window.
            let result = self.search_alpha_beta::<EmptyVariation>(
                &epos2,
                -alpha.next(),
                -alpha,
                self.depth.saturating_sub(ONE_PLY),
                NodeType::Cut,
            )?;
            let score = -result.score;
            if score <= alpha {
                self.root_moves_considered += 1;
                self.history.pop();
                continue;
            }
            // Full window search.
            let result = self.search_alpha_beta::<LongVariation>(
                &epos2,
                -Score::INFINITE,
                -alpha,
                self.depth.saturating_sub(ONE_PLY),
                NodeType::PV,
            )?;
            self.history.pop();
            let score = -result.score;
            if score > alpha {
                self.root_moves_setup[0..=self.root_moves_considered].rotate_right(1);
                self.blue_setup_score = score;
                self.pv = result.pv;
            }
            self.root_moves_considered += 1;
        }
        // Other moves.
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

pub struct SearchResultBlueSetup {
    pub score: Score,
    pub mov: SetupMove,
    pub pv: LongVariation,
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
    futile: bool,
}

enum MoveCandidate {
    Move { mov: Move, extra: bool },
    Null,
    Futility,
}

impl MoveCandidate {
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

#[derive(Debug)]
struct Timeout;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum NodeType {
    PV,
    Cut,
    All,
}
