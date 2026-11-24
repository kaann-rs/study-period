// Kitap: 15.1. bölüm, Eşzamanlılık: Derinleşme

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, TryLockError};
use std::thread;

pub fn run() {
    println!("\n== Derinleşme: eşzamanlılık pratikte ==");
    show_scoped_threads();
    show_worker_pool();
    show_shared_state();
    show_atomic_counter();
    show_hidden_bugs();
}

pub fn parallel_sum(data: &[u64], part_count: usize) -> u64 {
    let part_size = data.len().div_ceil(part_count.max(1)).max(1);
    thread::scope(|scope| {
        let handles: Vec<_> = data
            .chunks(part_size)
            .map(|part| scope.spawn(move || part.iter().sum::<u64>()))
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .sum()
    })
}

pub fn parallel_square(data: &mut [u64], part_count: usize) {
    let part_size = data.len().div_ceil(part_count.max(1)).max(1);
    thread::scope(|scope| {
        for part in data.chunks_mut(part_size) {
            scope.spawn(move || {
                for x in part {
                    *x *= *x;
                }
            });
        }
    });
}

fn show_scoped_threads() {
    let mut numbers: Vec<u64> = (1..=1000).collect();
    println!("paralel toplam (4 parça) = {}", parallel_sum(&numbers, 4));
    parallel_square(&mut numbers, 4);
    println!(
        "kareler: ilk üç = {:?}, son = {}",
        &numbers[..3],
        numbers[999]
    );
}

pub struct Job {
    pub id: u32,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct JobResult {
    pub id: u32,
    pub word_count: usize,
}

pub fn run_in_pool(jobs: Vec<Job>, worker_count: usize) -> Vec<JobResult> {
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (result_tx, result_rx) = mpsc::channel::<JobResult>();
    let job_rx = Arc::new(Mutex::new(job_rx));

    let mut workers = Vec::new();
    for _ in 0..worker_count.max(1) {
        let job_rx = Arc::clone(&job_rx);
        let result_tx = result_tx.clone();
        workers.push(thread::spawn(move || {
            loop {
                let received = job_rx.lock().unwrap().recv();
                match received {
                    Ok(job) => {
                        let word_count = job.text.split_whitespace().count();
                        if result_tx
                            .send(JobResult {
                                id: job.id,
                                word_count,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }));
    }
    drop(result_tx);

    for job in jobs {
        job_tx.send(job).unwrap();
    }
    drop(job_tx);

    let mut results: Vec<JobResult> = result_rx.iter().collect();
    for worker in workers {
        worker.join().unwrap();
    }
    results.sort_by_key(|result| result.id);
    results
}

fn show_worker_pool() {
    let lines = [
        "bir iki üç",
        "tek",
        "dört kelimelik bir satır",
        "",
        "son satır",
    ];
    let jobs: Vec<Job> = (1..)
        .zip(lines)
        .map(|(id, text)| Job {
            id,
            text: text.to_string(),
        })
        .collect();
    let results = run_in_pool(jobs, 3);
    let summary: Vec<String> = results
        .iter()
        .map(|result| format!("#{}:{}", result.id, result.word_count))
        .collect();
    println!("işçi havuzu sonuçları = {}", summary.join(" "));
}

pub fn count_words(documents: &[&str]) -> BTreeMap<String, u32> {
    let shared: Mutex<BTreeMap<String, u32>> = Mutex::new(BTreeMap::new());
    thread::scope(|scope| {
        for document in documents {
            let shared = &shared;
            scope.spawn(move || {
                let mut local: BTreeMap<String, u32> = BTreeMap::new();
                for word in document.split_whitespace() {
                    *local.entry(word.to_lowercase()).or_insert(0) += 1;
                }
                let mut table = shared.lock().unwrap();
                for (word, count) in local {
                    *table.entry(word).or_insert(0) += count;
                }
            });
        }
    });
    shared.into_inner().unwrap()
}

fn show_shared_state() {
    let documents = ["rust hızlı ve güvenli", "Rust korkusuz", "güvenli ve hızlı"];
    let table = count_words(&documents);
    println!("kelime sıklığı = {table:?}");
}

pub fn parallel_count(thread_count: u64, per_thread: u64) -> u64 {
    let counter = AtomicU64::new(0);
    thread::scope(|scope| {
        for _ in 0..thread_count {
            scope.spawn(|| {
                for _ in 0..per_thread {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    counter.into_inner()
}

fn show_atomic_counter() {
    println!(
        "atomik sayaç (8 thread × 10_000) = {}",
        parallel_count(8, 10_000)
    );
}

fn show_hidden_bugs() {
    let account = Mutex::new(100);
    let first = account.lock().unwrap();
    let second_attempt = matches!(account.try_lock(), Err(TryLockError::WouldBlock));
    println!("kilit elimizdeyken ikinci kilit bloklanırdı: {second_attempt}");
    drop(first);
    println!(
        "bıraktıktan sonra alınabiliyor: {}",
        account.try_lock().is_ok()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_sum_equals_serial_sum() {
        let data: Vec<u64> = (1..=100).collect();
        for part_count in [1, 3, 7, 100, 500] {
            assert_eq!(parallel_sum(&data, part_count), 5050);
        }
        assert_eq!(parallel_sum(&[], 4), 0);
    }

    #[test]
    fn threads_write_to_separate_parts() {
        let mut data = vec![1, 2, 3, 4, 5];
        parallel_square(&mut data, 2);
        assert_eq!(data, [1, 4, 9, 16, 25]);
    }

    #[test]
    fn pool_runs_each_job_once() {
        let jobs: Vec<Job> = (0..50)
            .map(|id| Job {
                id,
                text: "a ".repeat(usize::try_from(id).unwrap()),
            })
            .collect();
        let results = run_in_pool(jobs, 4);
        assert_eq!(results.len(), 50);
        for (index, result) in results.iter().enumerate() {
            assert_eq!(usize::try_from(result.id).unwrap(), index);
            assert_eq!(result.word_count, index);
        }
    }

    #[test]
    fn word_counts_join_all_documents() {
        let table = count_words(&["a b", "B c", "c C c"]);
        let expected = BTreeMap::from([
            (String::from("a"), 1),
            (String::from("b"), 2),
            (String::from("c"), 4),
        ]);
        assert_eq!(table, expected);
    }

    #[test]
    fn atomic_counter_loses_nothing() {
        assert_eq!(parallel_count(4, 5_000), 20_000);
    }

    #[test]
    fn poisoned_lock_can_be_recovered() {
        let account = Arc::new(Mutex::new(vec![1]));
        let account_copy = Arc::clone(&account);
        let result = thread::spawn(move || {
            let mut guard = account_copy.lock().unwrap();
            guard.push(2);
            panic!("kasıtlı panik: kilit elimizdeyken");
        })
        .join();
        assert!(result.is_err());
        assert!(account.is_poisoned());
        assert!(account.lock().is_err());
        let recovered = account.lock().unwrap_or_else(|poison| poison.into_inner());
        assert_eq!(*recovered, [1, 2]);
    }
}

// Learn more
//
// You can learn scoped threads from these sources:
//   - Standard library docs, "std::thread::scope": https://doc.rust-lang.org/std/thread/fn.scope.html
//   - Rust Atomics and Locks, "Basics of Rust Concurrency": https://mara.nl/atomics/basics.html#scoped-threads
//   - Easy: Comprehensive Rust, "Scoped Threads": https://google.github.io/comprehensive-rust/concurrency/threads/scoped.html
//
// You can learn worker pools from these sources:
//   - The Rust Book, "From Single-Threaded to Multithreaded Server": https://doc.rust-lang.org/book/ch21-02-multithreaded.html
//   - docs.rs, "threadpool": https://docs.rs/threadpool/latest/threadpool/
//   - Crust of Rust (video), "Channels": https://www.youtube.com/watch?v=b4mS5UPHh20
//   - Easy: Let's Get Rusty (video), "Building a Web Server in Rust - Part 2": https://www.youtube.com/watch?v=1AamFJGAE8E
//
// You can learn sharing results with Arc and Mutex from these sources:
//   - The Rust Book, "Shared-State Concurrency": https://doc.rust-lang.org/book/ch16-03-shared-state.html
//   - Comprehensive Rust, "Example": https://google.github.io/comprehensive-rust/concurrency/shared-state/example.html
//   - Rust Atomics and Locks, "Basics of Rust Concurrency": https://mara.nl/atomics/basics.html#mutexes
//   - Easy: Let's Get Rusty (video), "Concurrency in Rust - Sharing State": https://www.youtube.com/watch?v=mupwF9jbVZ4
//
// You can learn atomic counters from these sources:
//   - Rust Atomics and Locks, "Atomics": https://mara.nl/atomics/atomics.html
//   - Standard library docs, "std::sync::atomic": https://doc.rust-lang.org/std/sync/atomic/index.html
//   - cfsamson, "Explaining Atomics in Rust": https://cfsamsonbooks.gitbook.io/explaining-atomics-in-rust
//   - Easy: DEV Community, "The Missing Guide to Rust's Atomic::Ordering": https://dev.to/aryan_getsrusty/the-missing-guide-to-rusts-atomicordering-1g9p
//
// You can learn deadlocks and poisoned locks from these sources:
//   - Effective Rust, "Be wary of shared-state parallelism": https://effective-rust.com/deadlock.html
//   - Standard library docs, "Poisoning": https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
//   - Rustonomicon, "Poisoning": https://doc.rust-lang.org/nomicon/poisoning.html
//   - Rust users forum, "Lock order reversals: how to prevent them?": https://users.rust-lang.org/t/lock-order-reversals-how-to-prevent-them/65016
