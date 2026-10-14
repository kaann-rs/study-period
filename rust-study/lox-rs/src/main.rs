use lox_rs::Lox;

fn main() -> core::result::Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();

    match args {
        y if y.len() > 2 => {
            println!("Usage: lox-rs [script]");
            std::process::exit( 64);
        }
        c if c.len() == 2 => {
            Lox::run_file(&c[1])?;
        }
        _ => {
            Lox::run_prompt();
        }
    }

    Ok(())
}
