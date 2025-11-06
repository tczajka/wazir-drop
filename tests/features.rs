use extra::{moverand, PSFeatures};
use rand::{rngs::StdRng, SeedableRng};
use wazir_drop::{
    enums::{EnumMap, SimpleEnumExt},
    Color, Features, Position, Stage, WPSFeatures,
};

#[test]
fn test_ps_features() {
    assert_eq!(PSFeatures.count(), 80);
    test_features_random_games(&PSFeatures);
}

#[test]
fn test_wps_features() {
    assert_eq!(WPSFeatures.count(), 6360);
    test_features_random_games(&WPSFeatures);
}

fn gen_feature_vecs<F: Features>(features: &F, position: &Position) -> EnumMap<Color, Vec<i32>> {
    EnumMap::from_fn(|color| {
        let mut v = vec![0; features.count()];
        for feature in features.all(position, color) {
            v[feature] += 1;
        }
        v
    })
}

fn test_features_random_games<F: Features>(features: &F) {
    let mut rng = StdRng::from_os_rng();
    for _ in 0..100 {
        let mut position = Position::initial();
        let mut vs = gen_feature_vecs(features, &position);

        while !matches!(position.stage(), Stage::End(_)) {
            let mov = moverand::random_move(&position, &mut rng);
            position = position.make_move(mov).unwrap();
            let new_vs = gen_feature_vecs(features, &position);
            for color in Color::all() {
                if let Some((added, removed)) = features.diff(mov, &position, color) {
                    for feature in added {
                        vs[color][feature] += 1;
                    }
                    for feature in removed {
                        vs[color][feature] -= 1;
                    }
                    assert_eq!(vs[color], new_vs[color]);
                }
            }
            vs = new_vs;
        }
    }
}
