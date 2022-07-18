use std::{thread, sync::{mpsc, Arc, Mutex}};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Arguments
    /// 
    /// * `size` - The number of threads in the pool.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use yarws::ThreadPool;
    /// 
    /// let pool = ThreadPool::new(4);
    /// 
    /// pool.execute(|| {
    ///    println!("Hello from a thread!");
    /// });
    /// ```
    /// 
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero.

    // TODO: Return Result<ThreadPool, PoolCreationError>
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Execute a job on the ThreadPool.
    /// 
    /// # Arguments
    /// 
    /// * `job` - The job to execute.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use yarws::ThreadPool;
    /// 
    /// let pool = ThreadPool::new(4);
    /// 
    /// pool.execute(|| {
    ///   println!("Hello from a thread!");
    /// });
    /// ```
    /// 
    /// # Panics
    /// 
    /// The `execute` function will panic if the ThreadPool has been shut down.
    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Shutdown the ThreadPool.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use yarws::ThreadPool;
    /// 
    /// let pool = ThreadPool::new(4);
    /// 
    /// pool.execute(|| {
    ///  println!("Hello from a thread!");
    /// });
    /// 
    /// drop(pool);
    /// ```
    /// 
    /// # Panics
    /// 
    /// The `drop` function will panic if the ThreadPool has been shut down.
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Gracefully shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);

                    job();
                },
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);

                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_executes() {
        let pool = ThreadPool::new(4);

        pool.execute(|| {
            println!("Hello from a thread!");
        });
    }

    #[test]
    fn thread_pool_executes_many_jobs() {
        let pool = ThreadPool::new(4);

        for _ in 0..8 {
            pool.execute(|| {
                println!("Hello from a thread!");
            });
        }
    }

    #[test]
    fn thread_pool_shutdown() {
        let pool = ThreadPool::new(4);

        for _ in 0..8 {
            pool.execute(|| {
                println!("Hello from a thread!");
            });
        }

        drop(pool);
    }

    #[test]
    #[should_panic]
    fn thread_pool_should_panic_if_size_is_zero() {
        let pool = ThreadPool::new(0);

        pool.execute(|| {
            println!("Hello from a thread!");
        });
    }

    #[test]
    fn drop_thread_pool_should_shutdown_all_workers() {
        let pool = ThreadPool::new(4);

        for _ in 0..8 {
            pool.execute(|| {
                println!("Hello from a thread!");
            });
        }

        drop(pool);
    }
}
