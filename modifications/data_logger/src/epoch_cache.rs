pub struct EpochCache {
    epoch_length: usize,
    current: Option<u64>,
    working: Vec<u64>,
}

impl Default for EpochCache {
    fn default() -> Self {
        EpochCache::new()
    }
}

impl EpochCache {
    pub fn new() -> Self {
        EpochCache {
            epoch_length: 10,
            current: None,
            working: Vec::new(),
        }
    }

    pub fn add(&mut self, value: u64) {
        self.working.push(value);

        if self.working.len() == self.epoch_length {
            self.current = self.moving_average();

            self.working.clear();
        }
    }

    pub fn current(&self) -> Option<u64> {
        self.current
    }

    fn moving_average(&self) -> Option<u64> {
        if self.working.len() != self.epoch_length {
            return None;
        }

        let alpha = 0.5;

        let avg = self
            .working
            .iter()
            .skip(1)
            .fold(self.working[0] as f64, |avg, &cur| {
                alpha * (cur as f64) + (1.0 - alpha) * avg
            });

        Some(avg as u64)
    }
}
