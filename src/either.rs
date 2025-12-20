#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A: Iterator, B: Iterator<Item = A::Item>> Either<A, B> {
    pub fn try_for_each_result<E, F>(mut self, f: F) -> Result<(), E>
    where
        F: FnMut(A::Item) -> Result<(), E>,
    {
        match &mut self {
            Either::Left(a) => a.try_for_each(f),
            Either::Right(b) => b.try_for_each(f),
        }
    }
}

impl<A: Iterator, B: Iterator<Item = A::Item>> Iterator for Either<A, B> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(a) => a.next(),
            Either::Right(b) => b.next(),
        }
    }

    fn fold<Acc, F>(self, init: Acc, f: F) -> Acc
    where
        F: FnMut(Acc, Self::Item) -> Acc,
    {
        match self {
            Either::Left(a) => a.fold(init, f),
            Either::Right(b) => b.fold(init, f),
        }
    }
}
