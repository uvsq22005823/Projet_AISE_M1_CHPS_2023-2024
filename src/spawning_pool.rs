// Credits: https://doc.rust-lang.org/book/ch20-02-multithreaded.html
// and https://doc.rust-lang.org/book/ch20-03-graceful-shutdown-and-cleanup.html

use std::thread;
use std::sync::{Arc, Mutex, mpsc};

/// A structure with a thread and a Sender in it
///
/// More details can be found in the doc linked on top of this source file
///
/// WARNING: It is NOT finished yes (the program can't call drop function yet)
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize, saver: thread::JoinHandle<()>) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size + 1);
        workers.push(Worker{id: 0, thread: Some(saver)});

        for id in 1..size + 1 {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }


    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}



/// Not used unfortunately
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}



/// A thread with an ID that should stop when calling drop
struct Worker {
    id: usize,
    // thread is of type Option to bypass join() taking ownership
    thread: Option<thread::JoinHandle<()>>,
}


impl Worker {
    /// Create a worker (thread with id)
    ///
    /// WARNING: I have not tested the behaviour when a thread is None
    ///     You should probably kill the program if it happens idk
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // Try to handle thread not creating (thus Builder instead of thread::spawn)
        // I'm not really sure how to handle it tho
        let builder = thread::Builder::new().name(id.to_string());
        match builder.spawn(move || loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            })
        {
            Ok(t) => {
                return Worker { id, thread: Some(t) };
            }
            Err(_e) => {
                // Perhaps I should handle that differently but oh well.
                println!("Couldn't create thread number {}, aborting may be wise !", id);
                return Worker { id, thread: None};
            }
        };
    }
}
