use extra::moverand;
use rand::{rngs::StdRng, SeedableRng};
use wazir_drop::{
    enums::{EnumMap, SimpleEnumExt},
    Color, Features, PieceSquareFeatures, Position, Stage,
};

#[test]
fn test_piece_square_features() {
    test_features_random_games::<PieceSquareFeatures>();
}

fn gen_feature_vecs<F: Features>(position: &Position) -> EnumMap<Color, Vec<i32>> {
    EnumMap::from_fn(|color| {
        let mut f = vec![0; F::COUNT];
        for feature in F::all(position, color) {
            f[feature] += 1;
        }
        f
    })
}

fn test_features_random_games<F: Features>() {
    let mut rng = StdRng::from_os_rng();
    for _ in 0..100 {
        let mut position = Position::initial();
        let mut features = gen_feature_vecs::<F>(&position);

        while position.stage() != Stage::End {
            let mov = moverand::random_move(&position, &mut rng);
            position = position.make_move(mov).unwrap();
            let new_features = gen_feature_vecs::<F>(&position);
            for color in Color::all() {
                if let Some((added, removed)) = F::diff(mov, &position, color) {
                    for feature in added {
                        features[color][feature] += 1;
                    }
                    for feature in removed {
                        features[color][feature] -= 1;
                    }
                    assert_eq!(features[color], new_features[color]);
                }
            }
            features = new_features;
        }
    }
}
