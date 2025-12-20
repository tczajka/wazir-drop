#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Either<T0, T1> {
    Case0(T0),
    Case1(T1),
}

impl<T0: Iterator, T1: Iterator<Item = T0::Item>> Either<T0, T1> {
    pub fn try_for_each_result<E, F>(&mut self, f: F) -> Result<(), E>
    where
        F: FnMut(T0::Item) -> Result<(), E>,
    {
        match self {
            Either::Case0(a) => a.try_for_each(f),
            Either::Case1(b) => b.try_for_each(f),
        }
    }
}

impl<T0: Iterator, T1: Iterator<Item = T0::Item>> Iterator for Either<T0, T1> {
    type Item = T0::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Case0(a) => a.next(),
            Either::Case1(b) => b.next(),
        }
    }

    fn fold<Acc, F>(self, init: Acc, f: F) -> Acc
    where
        F: FnMut(Acc, Self::Item) -> Acc,
    {
        match self {
            Either::Case0(a) => a.fold(init, f),
            Either::Case1(b) => b.fold(init, f),
        }
    }
}
