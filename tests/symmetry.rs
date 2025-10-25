use wazir_drop::{enums::SimpleEnumExt, NormalizedSquare, Square, Symmetry};

#[test]
fn test_inverse() {
    assert_eq!(Symmetry::FlipX.inverse(), Symmetry::FlipX);
    assert_eq!(Symmetry::RotateLeft.inverse(), Symmetry::RotateRight);

    for symmetry in Symmetry::all() {
        assert_eq!(symmetry.inverse().inverse(), symmetry);
    }
}

#[test]
fn test_apply() {
    assert_eq!(Symmetry::Identity.apply(Square::A2), Square::A2);
    assert_eq!(Symmetry::FlipX.apply(Square::A2), Square::A7);
    assert_eq!(Symmetry::FlipY.apply(Square::A2), Square::H2);
    assert_eq!(Symmetry::Rotate180.apply(Square::A2), Square::H7);
    assert_eq!(Symmetry::SwapXY.apply(Square::A2), Square::B1);
    assert_eq!(Symmetry::RotateLeft.apply(Square::A2), Square::G1);
    assert_eq!(Symmetry::RotateRight.apply(Square::A2), Square::B8);
    assert_eq!(Symmetry::OtherDiagonal.apply(Square::A2), Square::G8);

    for symmetry in Symmetry::all() {
        for square in Square::all() {
            assert_eq!(symmetry.inverse().apply(symmetry.apply(square)), square);
        }
    }
}

#[test]
fn test_normalize() {
    assert_eq!(
        Symmetry::normalize(Square::A2),
        (Symmetry::Identity, NormalizedSquare::A2)
    );
    assert_eq!(
        Symmetry::normalize(Square::A7),
        (Symmetry::FlipX, NormalizedSquare::A2)
    );
    assert_eq!(
        Symmetry::normalize(Square::B1),
        (Symmetry::SwapXY, NormalizedSquare::A2)
    );
    assert_eq!(
        Symmetry::normalize(Square::B8),
        (Symmetry::RotateLeft, NormalizedSquare::A2)
    );
    assert_eq!(
        Symmetry::normalize(Square::G1),
        (Symmetry::RotateRight, NormalizedSquare::A2)
    );
    assert_eq!(
        Symmetry::normalize(Square::G8),
        (Symmetry::OtherDiagonal, NormalizedSquare::A2)
    );
    assert_eq!(
        Symmetry::normalize(Square::H2),
        (Symmetry::FlipY, NormalizedSquare::A2)
    );
    assert_eq!(
        Symmetry::normalize(Square::H7),
        (Symmetry::Rotate180, NormalizedSquare::A2)
    );

    for square in Square::all() {
        let (symmetry, normalized_square) = Symmetry::normalize(square);
        assert_eq!(symmetry.inverse().apply(normalized_square.into()), square);
    }
}
