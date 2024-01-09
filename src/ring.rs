pub struct Ring<T> {
    capacity: usize,
    items: Vec<T>,
}

impl<'a, T> IntoIterator for &'a Ring<T> {
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        // TODO
        self.items.iter()
    }
}

impl<T> Ring<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn size(&self) -> usize {
        // TODO?
        self.items.len()
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn add(&mut self, item: T) {
        // TODO check capacity
        self.items.push(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_should_have_capacity() {
        let r = Ring::<()>::new(5);
        assert_eq!(5, r.capacity());
    }

    #[test]
    fn empty_should_have_zero_size() {
        let r = Ring::<()>::new(5);
        assert_eq!(0, r.size());
    }

    #[test]
    fn should_impl_iter() {
        let r = Ring::<()>::new(5);
        for _ in &r {}
    }

    #[test]
    fn should_add_items() {
        let mut r = Ring::<u8>::new(5);
        r.add(42);
        r.add(43);
        r.add(7);
        r.add(0);
        r.add(3);

        let items: Vec<u8> = r.iter().copied().collect();
        assert_eq!(&[42, 43, 7, 0, 3], items.as_slice());
    }
}
