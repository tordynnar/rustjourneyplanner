#[derive(Debug, Clone)]
pub struct NeverEq<T> {
    pub value : T
}

impl<T> PartialEq for NeverEq<T> {
    fn eq(&self, _: &NeverEq<T>) -> bool {
        false
    }
}
