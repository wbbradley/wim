pub struct KeyGenerator {
    iter: std::ops::RangeFrom<i64>,
}

impl KeyGenerator {
    pub fn new() -> Self {
        Self {
            iter: (0 as i64..).into_iter(),
        }
    }
    pub fn next_key(&mut self) -> i64 {
        self.iter.next().unwrap()
    }
    pub fn next_key_string(&mut self) -> String {
        format!("{}", self.next_key())
    }
}
