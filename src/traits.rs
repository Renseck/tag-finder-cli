pub trait ThreadCountConfigurable {
    fn with_thread_count(self, count: usize) -> Self;
}

pub trait ConfigConfigurable {
    fn with_config(self, config: crate::config::Config) -> Self;
}

pub trait ProgressConfigurable {
    fn with_progress(self, show_progress: bool) -> Self;
}

pub trait ProcessorBuilder: ThreadCountConfigurable + Sized {
    fn configure_threads(self, thread_count: Option<usize>) -> Self {
        match thread_count {
            Some(count) => self.with_thread_count(count),
            None => self,
        }
    }
}

// Auto-implement for any type that has ThreadCountConfigurable
impl<T: ThreadCountConfigurable> ProcessorBuilder for T {}