use extra::{LinearEvaluator, PSFeatures, moverand};
use rand::{SeedableRng, rngs::StdRng};
use wazir_drop::{EvaluatedPosition, Evaluator, Nnue, Position, Stage, WPSFeatures};

#[test]
fn test_evaluators() {
    test_evaluator(&LinearEvaluator::<WPSFeatures>::default());
    test_evaluator(&LinearEvaluator::<PSFeatures>::default());
    test_evaluator(&Nnue::default());
}

fn test_evaluator<E: Evaluator>(evaluator: &E) {
    let mut rng = StdRng::from_os_rng();
    for _ in 0..100 {
        let mut position = EvaluatedPosition::new(evaluator, Position::initial());
        while !matches!(position.position().stage(), Stage::End(_)) {
            let mov = moverand::random_move(position.position(), &mut rng);
            position = position.make_any_move(mov).unwrap();
            let value = position.evaluate();
            let fresh_value =
                EvaluatedPosition::new(evaluator, position.position().clone()).evaluate();
            assert_eq!(value, fresh_value);
        }
    }
}
