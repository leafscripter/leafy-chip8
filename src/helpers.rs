pub fn as_index<T>(x: T) -> usize where usize : From<T> {
    usize::from(x)
}

