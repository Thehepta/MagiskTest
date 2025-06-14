
pub trait Silent<T> {
    fn silent(self) -> Option<T>;
}

impl<T, E> Silent<T> for Result<T, E> {
    fn silent(self) -> Option<T> {
        self.ok()
    }
}