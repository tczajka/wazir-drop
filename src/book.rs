use std::time::Instant;

use crate::{
    base128::{Base128Decoder, Base128Encoder},
    book_data,
    constants::{Depth, DEPTH_INCREMENT, MAX_SEARCH_DEPTH, RED_SETUP_INDEX},
    log, Color, Evaluator, Piece, Position, Score, Search, SetupMove, Symmetry, Timeout,
};

pub fn encode_setup_move(encoder: &mut Base128Encoder, setup_move: SetupMove) {
    for &piece in &setup_move.pieces {
        encode_piece(encoder, piece);
    }
}

pub fn decode_setup_move(decoder: &mut Base128Decoder, color: Color) -> SetupMove {
    let mut pieces = [Piece::Alfil; SetupMove::SIZE];
    for piece in &mut pieces {
        *piece = decode_piece(decoder);
    }
    SetupMove { color, pieces }
}

pub fn encode_piece(encoder: &mut Base128Encoder, piece: Piece) {
    match piece {
        Piece::Alfil => encoder.encode_bits(1, 0b0),
        Piece::Dabbaba => encoder.encode_bits(2, 0b01),
        Piece::Ferz => encoder.encode_bits(3, 0b011),
        Piece::Knight => encoder.encode_bits(4, 0b0111),
        Piece::Wazir => encoder.encode_bits(4, 0b1111),
    }
}

pub fn decode_piece(decoder: &mut Base128Decoder) -> Piece {
    if decoder.decode_bits(1) == 0 {
        Piece::Alfil
    } else if decoder.decode_bits(1) == 0 {
        Piece::Dabbaba
    } else if decoder.decode_bits(1) == 0 {
        Piece::Ferz
    } else if decoder.decode_bits(1) == 0 {
        Piece::Knight
    } else {
        Piece::Wazir
    }
}

pub fn red_setup() -> SetupMove {
    for book_opening in BookIterator::new() {
        if book_opening.index == RED_SETUP_INDEX {
            log::info!("red setup #{}", RED_SETUP_INDEX);
            return book_opening.red;
        }
    }
    panic!("RED_SETUP_INDEX not found")
}

pub fn blue_setup<E: Evaluator>(
    red: SetupMove,
    search: &mut Search<E>,
    deadline: Instant,
) -> SetupMove {
    let (symmetry, red) = Symmetry::normalize_red_setup(red);
    for book_opening in BookIterator::new() {
        if book_opening.red == red {
            log::info!("blue setup #{index}", index = book_opening.index);
            return symmetry.inverse().apply_to_setup(book_opening.blue);
        }
    }
    log::info!("opening not found");
    let mut instance = SearchBlueSetup::new(red, search, deadline);
    instance.search()
}

struct SearchBlueSetup<'a, E: Evaluator> {
    position: Position,
    search: &'a mut Search<E>,
    deadline: Instant,
    moves: Vec<SetupMove>,
    depth: Depth,
    root_moves_considered: usize,
    score: Score,
}

impl<'a, E: Evaluator> SearchBlueSetup<'a, E> {
    fn new(red: SetupMove, search: &'a mut Search<E>, deadline: Instant) -> Self {
        let position = Position::initial().make_setup_move(red).unwrap();
        let moves = BookIterator::new()
            .flat_map(|book_opening| {
                let mov = book_opening.blue.with_color(Color::Blue);
                [Symmetry::Identity, Symmetry::FlipX]
                    .iter()
                    .map(move |symmetry| symmetry.apply_to_setup(mov))
            })
            .collect();

        Self {
            position,
            search,
            deadline,
            moves,
            depth: 0,
            root_moves_considered: 0,
            score: Score::DRAW,
        }
    }

    fn search(&mut self) -> SetupMove {
        _ = self.iterative_deepening();
        log::info!(
            "blue setup depth={depth} {root_moves_considered}/{num_moves} score={score}",
            depth = self.depth,
            root_moves_considered = self.root_moves_considered,
            num_moves = self.moves.len(),
            score = self.score
        );
        self.moves[0]
    }

    fn iterative_deepening(&mut self) -> Result<(), Timeout> {
        loop {
            self.iterative_deepening_iteration()?;
            if self.depth >= MAX_SEARCH_DEPTH {
                break;
            }
            self.depth += DEPTH_INCREMENT;
        }
        Ok(())
    }

    fn iterative_deepening_iteration(&mut self) -> Result<(), Timeout> {
        self.root_moves_considered = 0;
        let mut best_score = -Score::INFINITE;
        while self.root_moves_considered < self.moves.len() {
            let position = self
                .position
                .make_setup_move(self.moves[self.root_moves_considered])
                .unwrap();
            let score = -self.search.try_search_position(
                &position,
                self.depth,
                -Score::INFINITE,
                -best_score,
                Some(self.deadline),
            )?;
            if score > best_score {
                best_score = score;
                self.score = score;
                self.moves[0..=self.root_moves_considered].rotate_right(1);
            }
            self.root_moves_considered += 1;
        }
        Ok(())
    }
}

struct BookOpening {
    index: usize,
    red: SetupMove,
    blue: SetupMove,
}

struct BookIterator {
    next_index: usize,
    decoder: Option<Base128Decoder<'static>>,
}

impl BookIterator {
    fn new() -> Self {
        Self {
            next_index: 0,
            decoder: Some(Base128Decoder::new(book_data::OPENINGS)),
        }
    }
}

impl Iterator for BookIterator {
    type Item = BookOpening;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= book_data::NUM_OPENINGS {
            if let Some(decoder) = self.decoder.take() {
                decoder.finish();
            }
            return None;
        }
        let decoder = self.decoder.as_mut().unwrap();
        let red = decode_setup_move(decoder, Color::Red);
        let blue = decode_setup_move(decoder, Color::Blue);
        let opening = BookOpening {
            index: self.next_index,
            red,
            blue,
        };
        self.next_index += 1;
        Some(opening)
    }
}
