use rules::{OpeningMove, Piece};

#[test]
fn test_opening_move_size_matches_piece_initial_count() {
    let num_pieces = (0..Piece::COUNT)
        .map(|i| Piece::from_index(i).initial_count())
        .sum();

    assert_eq!(OpeningMove::SIZE, num_pieces);
}
