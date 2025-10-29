use extra::moverand;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::rc::Rc;
use wazir_drop::{
    EvaluatedPosition, Evaluator, Features, LinearEvaluator, PieceSquareFeatures, Position, Stage,
};

#[test]
fn test_linear_piece_square_evaluator() {
    test_linear_evaluator(PieceSquareFeatures);
}

fn test_linear_evaluator<F: Features>(features: F) {
    let mut rng = StdRng::from_os_rng();
    let to_move_weight = rng.random();
    let feature_weights: Vec<i16> = (0..features.count()).map(|_| rng.random()).collect();
    let evaluator = Rc::new(LinearEvaluator::new(
        features,
        to_move_weight,
        &feature_weights,
    ));
    test_evaluator(&evaluator, &mut rng);
}

fn test_evaluator<E: Evaluator>(evaluator: &Rc<E>, rng: &mut StdRng) {
    for _ in 0..100 {
        let mut position = EvaluatedPosition::new(evaluator, Position::initial());
        while position.position().stage() != Stage::End {
            let mov = moverand::random_move(position.position(), rng);
            position = position.make_move(mov).unwrap();
            let value = position.evaluate();
            let fresh_value =
                EvaluatedPosition::new(evaluator, position.position().clone()).evaluate();
            assert_eq!(value, fresh_value);
        }
    }
}
