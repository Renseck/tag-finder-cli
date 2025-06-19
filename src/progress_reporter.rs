pub struct ProgressReporter {
    total: usize,
    current: usize,
    step_size: usize,
    message: String,
}

impl ProgressReporter {
    pub fn new(total: usize, message: String) -> Self {
        Self {
            total,
            current: 0,
            step_size: std::cmp::max(1, total / 20),
            message,
        }
    }

    /* ========================================================================================== */
    pub fn with_step_size(mut self, step_size: usize) -> Self {
        self.step_size = step_size;
        self
    }

    /* ========================================================================================== */
    pub fn tick(&mut self) {
        self.current += 1;
        if self.current % self.step_size == 0 || self.current == self.total {
            println!("   {} {}/{}", self.message, self.current, self.total);
        }
    }

    /* ========================================================================================== */
    pub fn finish(&self, completion_message: &str) {
        println!("✅ {}", completion_message);
    }
}