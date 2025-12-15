use crate::{
    constants::Eval, enums::EnumMap, AnyMove, Color, Features, InvalidMove, Move, Position,
    SetupMove,
};

pub trait Evaluator: Send + Sync + 'static {
    type Accumulator: Clone;
    type Features: Features;

    fn features(&self) -> Self::Features;
    fn new_accumulator(&self) -> Self::Accumulator;
    fn add_feature(&self, accumulator: &mut Self::Accumulator, feature: usize);
    fn remove_feature(&self, accumulator: &mut Self::Accumulator, feature: usize);
    fn evaluate(&self, accumulators: &EnumMap<Color, Self::Accumulator>, to_move: Color) -> Eval;
    fn scale(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct EvaluatedPosition<'a, E: Evaluator> {
    evaluator: &'a E,
    position: Position,
    accumulators: EnumMap<Color, E::Accumulator>,
}

impl<'a, E: Evaluator> EvaluatedPosition<'a, E> {
    pub fn new(evaluator: &'a E, position: Position) -> Self {
        let accumulators = EnumMap::from_fn(|color| refresh(evaluator, &position, color));
        Self {
            evaluator,
            position,
            accumulators,
        }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn make_any_move(&self, mov: AnyMove) -> Result<Self, InvalidMove> {
        match mov {
            AnyMove::Setup(mov) => self.make_setup_move(mov),
            AnyMove::Regular(mov) => self.make_move(mov),
        }
    }

    pub fn make_setup_move(&self, mov: SetupMove) -> Result<Self, InvalidMove> {
        let position = self.position.make_setup_move(mov)?;
        let accumulators = EnumMap::from_fn(|color| {
            update(
                self.evaluator,
                &self.accumulators[color],
                &position,
                color,
                self.evaluator.features().diff_setup(mov, &position, color),
            )
        });
        Ok(Self {
            evaluator: self.evaluator,
            position,
            accumulators,
        })
    }

    pub fn make_move(&self, mov: Move) -> Result<Self, InvalidMove> {
        let position = self.position.make_move(mov)?;
        let accumulators = EnumMap::from_fn(|color| {
            update(
                self.evaluator,
                &self.accumulators[color],
                &position,
                color,
                self.evaluator.features().diff(mov, &position, color),
            )
        });
        Ok(Self {
            evaluator: self.evaluator,
            position,
            accumulators,
        })
    }

    pub fn make_null_move(&self) -> Result<Self, InvalidMove> {
        let position = self.position.make_null_move()?;
        Ok(Self {
            evaluator: self.evaluator,
            position,
            accumulators: self.accumulators.clone(),
        })
    }

    pub fn evaluate(&self) -> Eval {
        self.evaluator
            .evaluate(&self.accumulators, self.position.to_move())
    }
}

fn refresh<E: Evaluator>(evaluator: &E, position: &Position, color: Color) -> E::Accumulator {
    let mut acc = evaluator.new_accumulator();
    for feature in evaluator.features().all(position, color) {
        evaluator.add_feature(&mut acc, feature);
    }
    acc
}

fn update<E: Evaluator, Added, Removed>(
    evaluator: &E,
    accumulator: &E::Accumulator,
    new_position: &Position,
    color: Color,
    diff: Option<(Added, Removed)>,
) -> E::Accumulator
where
    Added: Iterator<Item = usize>,
    Removed: Iterator<Item = usize>,
{
    match diff {
        Some((added, removed)) => {
            let mut accumulator = accumulator.clone();
            for feature in removed {
                evaluator.remove_feature(&mut accumulator, feature);
            }
            for feature in added {
                evaluator.add_feature(&mut accumulator, feature);
            }
            accumulator
        }
        None => refresh(evaluator, new_position, color),
    }
}
