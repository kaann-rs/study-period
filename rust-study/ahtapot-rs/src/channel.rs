    use std::sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    };

    pub struct ThreadChannel<T> {
        pub sender: Sender<T>,
        pub receiver: Arc<Mutex<Receiver<T>>>,
    }

    impl<T> ThreadChannel<T> {
        pub fn new() -> Self {
            let (tx, rx) = mpsc::channel::<T>();

            let rx = Arc::new(Mutex::new(rx));

            Self {
                sender: tx,
                receiver: rx,
            }
        }
    }
