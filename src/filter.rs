pub trait FilterFn: Clone {
    type Input: Clone;
    fn filter(&self, input: Self::Input) -> bool;
}

#[derive(Default, Clone, Debug)]
pub struct Filter<F: FilterFn> {
    filter: Vec<F>,
}

impl<F: FilterFn> Filter<F> {
    pub fn new() -> Self {
        Self { filter: Vec::new() }
    }
    pub fn with_filter<T>(filter: T) -> Filter<F>
    where
        T: AsRef<[F]>,
    {
        Self {
            filter: filter.as_ref().to_vec(),
        }
    }
    pub fn add_filter(self, filter: F) -> Filter<F> {
        let mut filters = self.filter;
        filters.push(filter);
        Self { filter: filters }
    }
    pub fn replace_filter(&mut self, filter: Vec<F>) {
        self.filter = filter;
    }
    pub fn get_filter(&self) -> &Vec<F> {
        &self.filter
    }
}

impl<F: FilterFn> FilterFn for Filter<F> {
    type Input = F::Input;
    fn filter(&self, input: Self::Input) -> bool {
        self.filter.iter().any(|f| f.filter(input.clone()))
    }
}

impl<T> FilterFn for T
where
    T: AsRef<str> + Clone,
{
    type Input = T;
    fn filter(&self, input: Self::Input) -> bool {
        input.as_ref().contains(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let filter = Filter::<String>::with_filter(vec!["a".to_string(), "b".to_string()])
            .add_filter("c".to_string());
        dbg!(&filter);
        assert!(filter.filter("ajdkfjldsafhjka".to_string()));
        assert!(filter.filter("b".to_string()));
        assert!(filter.filter("c".to_string()));
        assert!(!filter.filter("d".to_string()));
    }
}
