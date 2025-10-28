use std::rc::Rc;

use crate::{enums::EnumMap, Color, Features, InvalidMove, Move, Position, RegularMove, SetupMove};

pub trait Evaluator {
    type Accumulator: Clone;
    type Features: Features;

    fn features(&self) -> &Self::Features;
    fn new_accumulator(&self) -> Self::Accumulator;
    fn add_feature(&self, accumulator: &mut Self::Accumulator, feature: usize);
    fn remove_feature(&self, accumulator: &mut Self::Accumulator, feature: usize);
    fn evaluate(&self, accumulators: &EnumMap<Color, Self::Accumulator>, to_move: Color) -> i32;
}

#[derive(Debug, Clone)]
pub struct EvaluatedPosition<E: Evaluator> {
    evaluator: Rc<E>,
    position: Position,
    accumulators: EnumMap<Color, E::Accumulator>,
}

impl<E: Evaluator> EvaluatedPosition<E> {
    pub fn new(evaluator: &Rc<E>, position: Position) -> Self {
        let accumulators = EnumMap::from_fn(|color| Self::refresh(evaluator, &position, color));
        Self {
            evaluator: Rc::clone(evaluator),
            position,
            accumulators,
        }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    fn refresh(evaluator: &E, position: &Position, color: Color) -> E::Accumulator {
        let mut acc = evaluator.new_accumulator();
        for feature in evaluator.features().all(position, color) {
            evaluator.add_feature(&mut acc, feature);
        }
        acc
    }

    fn update<Added, Removed>(
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
                for feature in added {
                    evaluator.add_feature(&mut accumulator, feature);
                }
                for feature in removed {
                    evaluator.remove_feature(&mut accumulator, feature);
                }
                accumulator
            }
            None => Self::refresh(evaluator, new_position, color),
        }
    }

    pub fn make_move(&self, mov: Move) -> Result<Self, InvalidMove> {
        match mov {
            Move::Setup(mov) => self.make_setup_move(mov),
            Move::Regular(mov) => self.make_regular_move(mov),
        }
    }

    pub fn make_setup_move(&self, mov: SetupMove) -> Result<Self, InvalidMove> {
        let position = self.position.make_setup_move(mov)?;
        let accumulators = EnumMap::from_fn(|color| {
            Self::update(
                &self.evaluator,
                &self.accumulators[color],
                &position,
                color,
                self.evaluator.features().diff_setup(mov, &position, color),
            )
        });
        Ok(Self {
            evaluator: self.evaluator.clone(),
            position,
            accumulators,
        })
    }

    pub fn make_regular_move(&self, mov: RegularMove) -> Result<Self, InvalidMove> {
        let position = self.position.make_regular_move(mov)?;
        let accumulators = EnumMap::from_fn(|color| {
            Self::update(
                &self.evaluator,
                &self.accumulators[color],
                &position,
                color,
                self.evaluator
                    .features()
                    .diff_regular(mov, &position, color),
            )
        });
        Ok(Self {
            evaluator: self.evaluator.clone(),
            position,
            accumulators,
        })
    }

    pub fn evaluate(&self) -> i32 {
        self.evaluator
            .evaluate(&self.accumulators, self.position.to_move())
    }
}
