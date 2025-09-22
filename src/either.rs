#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}
