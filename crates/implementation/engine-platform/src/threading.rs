//! Platform threading utilities

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Thread pool for managing worker threads
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Job>,
}

/// Worker thread
struct Worker {
    handle: JoinHandle<()>,
}

/// Job type for thread pool
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new thread pool
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self { workers, sender }
    }

    /// Execute a job on the thread pool
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }

    /// Get the number of worker threads
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Close the channel by dropping the sender
        // This will cause worker threads to exit their loops

        // Wait for all workers to finish
        for worker in self.workers.drain(..) {
            if let Err(e) = worker.handle.join() {
                eprintln!("Worker thread panicked: {:?}", e);
            }
        }
    }
}

impl Worker {
    /// Create a new worker
    fn new(_id: usize, receiver: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    break;
                }
            }
        });

        Self { handle }
    }
}
