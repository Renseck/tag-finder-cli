use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use  crate::utils::{create_thread_pool, update_progress, calculate_progress_step_size};
use crate::traits::{ThreadCountConfigurable, ProgressConfigurable};

pub struct ParallelProcessor {
    thread_count: Option<usize>,
    show_progress: bool,
}

impl ParallelProcessor {
    pub fn new() -> Self {
        Self { 
            thread_count: None,
            show_progress: true,
        }
    }

    /* ========================================================================================== */
    pub fn process<T, R, F>(
        &self,
        items: Vec<T>,
        processor: F,
        message: &str,
    ) -> Result<Vec<R>, Box<dyn std::error::Error>> 
    where
        T: Send + Sync,
        R: Send,
        F: Fn(&T) -> Result<R, Box<dyn std::error::Error + Send + Sync>> + Send + Sync,
    {
        let pool = create_thread_pool(self.thread_count)?;
        let total = items.len();

        if self.show_progress {
            println!("{} {} items using {} threads...", message, total, pool.current_num_threads());
        }

        let results: Result<Vec<_>, Box<dyn std::error::Error + Send + Sync>> = if self.show_progress {
            let progress_counter = Arc::new(Mutex::new(0usize));
            let step_size = calculate_progress_step_size(total, 20);

            pool.install(|| {
                items
                    .par_iter()
                    .map(|item| {
                        update_progress(&progress_counter, total, step_size);
                        processor(item)
                    })
                    .collect()
            })
        } else {
            pool.install(|| {
                items
                    .par_iter()
                    .map(|item| processor(item))
                    .collect()
            })
        };

        results.map_err(|e| -> Box<dyn std::error::Error> {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })
    }

    /* ========================================================================================== */
    pub fn process_flat_map<T, R, F>(
         &self,
        items: Vec<T>,
        mapper: F,
        message: &str,
    ) -> Result<Vec<R>, Box<dyn std::error::Error>>
    where
        T: Send + Sync,
        R: Send,
        F: Fn(&T) -> Vec<R> + Send + Sync,
    {
        let pool = create_thread_pool(self.thread_count)?;
        let total = items.len();

        if self.show_progress {
            println!("{} {} items using {} threads...", message, total, pool.current_num_threads());
        }

        let results: Vec<R> = if self.show_progress {
            let progress_counter = Arc::new(Mutex::new(0usize));
            let step_size = calculate_progress_step_size(total, 20);

            pool.install(|| {
                items
                    .par_iter()
                    .flat_map(|item| {
                        update_progress(&progress_counter, total, step_size);
                        mapper(item)
                    })
                    .collect()
            })
        } else {
            pool.install(|| {
                items
                    .par_iter()
                    .flat_map(|item| mapper(item))
                    .collect()
            })
        };

        Ok(results)
    }
}

impl ThreadCountConfigurable for ParallelProcessor {
    fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self
    }
}

impl ProgressConfigurable for ParallelProcessor {
    fn with_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }
}