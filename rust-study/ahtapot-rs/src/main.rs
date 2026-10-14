#[cfg(test)]
mod tests;

mod aht;
mod channel;
mod cli;
mod img;

use crate::channel::ThreadChannel;
use crate::{aht::AhtError, cli::Args, img::Image};
use std::thread::{self, available_parallelism};
use std::{path::PathBuf, sync::Arc};

fn main() {
    if let Err(e) = inner_main() {
        eprintln!("{:?}", e);
        std::process::exit(0);
    }
}

fn inner_main() -> Result<(), AhtError> {
    let data = Args::new()?;

    let args: Arc<Args> = Arc::new(data);
    let dir = args.get_dir()?;

    let num_thread = available_parallelism()?.get();

    let ThreadChannel {
        sender: tx,
        receiver: rx,
    } = ThreadChannel::<PathBuf>::new();

    let mut handles = vec![];

    for i in 0..num_thread {
        let args = Arc::clone(&args);
        let rx = Arc::clone(&rx);

        let handle = thread::spawn(move || -> Result<(), AhtError> {
            loop {
                let path = rx.lock().unwrap().recv();
                match path {
                    Ok(np) => {
                        let name = format!("{}{}", args.name, i);
                        Image::new(&np)?
                            .resize(args.width, args.height)
                            .save(&args.output, &name)?;
                    }
                    Err(_) => break,
                }
            }
            Ok(())
        });

        handles.push(handle);
    }

    for path in dir {
        if path.is_file() {
            tx.send(path).ok();
        }
    }

    drop(tx);

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}
