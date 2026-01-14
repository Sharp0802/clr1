use serde::Serialize;

#[derive(Serialize)]
pub struct Boxed<T>(Box<T>);

impl<T> Boxed<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}
