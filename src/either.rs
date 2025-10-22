#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A: Iterator, B: Iterator<Item = A::Item>> Iterator for Either<A, B> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(a) => a.next(),
            Either::Right(b) => b.next(),
        }
    }
}
