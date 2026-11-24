// Kitap: 11. bölüm, minigrep: Gerçek Bir Program

use std::env;
use std::process;

use minigrep::{Config, run};

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|error| {
        eprintln!("Argüman hatası: {error}");
        eprintln!("Kullanım: minigrep <sorgu> <dosya_yolu>   (büyük/küçük harf: IGNORE_CASE=1)");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Uygulama hatası: {e}");
        process::exit(1);
    }
}

// Learn more
//
// You can learn command line arguments from these sources:
//   - The Rust Book, "Accepting Command Line Arguments": https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html
//   - Command Line Book, "Parsing command line arguments": https://rust-cli.github.io/book/tutorial/cli-args.html
//   - Rust by Example, "Program arguments": https://doc.rust-lang.org/rust-by-example/std_misc/arg.html
//   - Easy: Easy Rust, "Taking user input": https://dhghomon.github.io/easy_rust/Chapter_63.html
//   - Easy: Let's Get Rusty (video), "Writing a CLI App in Rust! - Part 1": https://www.youtube.com/watch?v=XYkiwsplDTg
//
// You can learn stderr and exit codes from these sources:
//   - The Rust Book, "Redirecting Errors to Standard Error": https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html
//   - Command Line Book, "Exit codes": https://rust-cli.github.io/book/in-depth/exit-code.html
//   - Standard library docs, "std::eprintln": https://doc.rust-lang.org/std/macro.eprintln.html
//   - Easy: Let's Get Rusty (video), "Writing a CLI App in Rust! - Part 2": https://www.youtube.com/watch?v=AABHxixn6Cw
