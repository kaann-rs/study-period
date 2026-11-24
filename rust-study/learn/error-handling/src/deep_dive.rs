// Kitap: 8.1. bölüm, Hata Yönetimi: Derinleşme

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

pub fn run() {
    println!("\n== Derinleşme: Error Handling ==");
    show_custom_error();
    show_option_and_result();
    show_first_or_all_errors();
    show_file_loading();
    show_box_dyn_error();
    show_when_unwrap_is_ok();
}

#[derive(Debug)]
pub enum SettingsError {
    Io(io::Error),
    Format {
        line: usize,
        content: String,
    },
    Number {
        key: &'static str,
        value: String,
        source: ParseIntError,
    },
    Missing(&'static str),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SettingsError::Io(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
            SettingsError::Format { line, content } => {
                write!(
                    f,
                    "satır {line}: `anahtar=değer` bekleniyordu, bulunan {content:?}"
                )
            }
            SettingsError::Number { key, value, .. } => {
                write!(f, "`{key}` sayı olmalı, bulunan {value:?}")
            }
            SettingsError::Missing(key) => write!(f, "zorunlu anahtar eksik: {key}"),
        }
    }
}

impl Error for SettingsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SettingsError::Io(e) => Some(e),
            SettingsError::Number { source, .. } => Some(source),
            SettingsError::Format { .. } | SettingsError::Missing(_) => None,
        }
    }
}

impl From<io::Error> for SettingsError {
    fn from(e: io::Error) -> Self {
        SettingsError::Io(e)
    }
}

fn format_chain(error: &(dyn Error + 'static)) -> String {
    std::iter::successors(Some(error), |&cause| cause.source())
        .map(|cause| cause.to_string())
        .collect::<Vec<_>>()
        .join(" <- ")
}

#[derive(Debug, PartialEq)]
pub struct Settings {
    name: String,
    port: u16,
    max_connections: u32,
}

fn parse_line(line_number: usize, line: &str) -> Result<Option<(String, String)>, SettingsError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let (key, value) = line.split_once('=').ok_or_else(|| SettingsError::Format {
        line: line_number,
        content: line.to_string(),
    })?;
    Ok(Some((key.trim().to_string(), value.trim().to_string())))
}

fn parse_table(text: &str) -> Result<BTreeMap<String, String>, SettingsError> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| parse_line(i + 1, line).transpose())
        .collect()
}

fn read_number<T>(table: &BTreeMap<String, String>, key: &'static str) -> Result<T, SettingsError>
where
    T: FromStr<Err = ParseIntError>,
{
    let raw = table.get(key).ok_or(SettingsError::Missing(key))?;
    raw.parse().map_err(|source| SettingsError::Number {
        key,
        value: raw.clone(),
        source,
    })
}

pub fn parse_settings(text: &str) -> Result<Settings, SettingsError> {
    let table = parse_table(text)?;
    Ok(Settings {
        name: table
            .get("ad")
            .cloned()
            .unwrap_or_else(|| String::from("adsız")),
        port: read_number(&table, "port")?,
        max_connections: read_number(&table, "azami_baglanti")?,
    })
}

pub fn load_from_file(path: &Path) -> Result<Settings, SettingsError> {
    let text = fs::read_to_string(path)?;
    parse_settings(&text)
}

fn show_custom_error() {
    println!("-- 1. özel hata türü ve source zinciri --");
    let valid_text = "# sunucu\nad = deneme\nport = 8080\nazami_baglanti = 64\n";
    println!("geçerli: {:?}", parse_settings(valid_text));
    for bad_text in [
        "port = 8080\nazami baglanti 64",
        "port = seksen\nazami_baglanti = 64",
        "port = 8080",
        "port = 99999\nazami_baglanti = 1",
    ] {
        if let Err(e) = parse_settings(bad_text) {
            println!("hata: {}", format_chain(&e));
        }
    }
}

fn first_key(text: &str) -> Option<&str> {
    let line = text.lines().find(|line| !line.trim().is_empty())?;
    let (key, _) = line.split_once('=')?;
    Some(key.trim())
}

fn show_option_and_result() {
    println!("-- 2. Option <-> Result --");
    println!(
        "first_key = {:?} / {:?}",
        first_key("\nport = 1"),
        first_key("bozuk")
    );
    let missing: Option<u16> = None;
    println!(
        "ok_or      = {:?}",
        missing
            .ok_or(SettingsError::Missing("port"))
            .map_err(|e| e.to_string())
    );
    println!("ok()       = {:?}", "x1".parse::<u16>().ok());
}

fn collect_errors(text: &str) -> (usize, Vec<SettingsError>) {
    let mut valid_count = 0;
    let mut errors = Vec::new();
    for (i, line) in text.lines().enumerate() {
        match parse_line(i + 1, line) {
            Ok(Some(_)) => valid_count += 1,
            Ok(None) => {}
            Err(e) => errors.push(e),
        }
    }
    (valid_count, errors)
}

fn show_first_or_all_errors() {
    println!("-- 3. ilk hatada dur / hepsini topla --");
    let text = "port = 1\nbozuk satır\nad = x\nbir bozuk daha";
    let first = parse_table(text).map(|table| table.len());
    println!("collect (ilk hata): {:?}", first.map_err(|e| e.to_string()));
    let (valid_count, errors) = collect_errors(text);
    println!("toplama: {valid_count} geçerli, {} hata", errors.len());
    for e in &errors {
        println!("  - {e}");
    }
}

fn with_temp_file<T>(label: &str, content: &str, action: impl FnOnce(&Path) -> T) -> io::Result<T> {
    let name = format!("rust_study_ayar_{}_{label}.conf", std::process::id());
    let path = std::env::temp_dir().join(name);
    fs::write(&path, content)?;
    let result = action(&path);
    fs::remove_file(&path)?;
    Ok(result)
}

fn show_file_loading() {
    println!("-- 4. dosyadan yükleme --");
    let content = "ad = üretim\nport = 443\nazami_baglanti = 1000\n";
    match with_temp_file("run", content, load_from_file) {
        Ok(Ok(settings)) => println!(
            "yüklendi: {} :{} (azami {})",
            settings.name, settings.port, settings.max_connections
        ),
        Ok(Err(e)) => println!("ayar hatası: {}", format_chain(&e)),
        Err(e) => println!("geçici dosya kullanılamadı: {:?}", e.kind()),
    }
    let missing_file = std::env::temp_dir().join("rust_study_boyle_bir_dosya_yok.conf");
    match load_from_file(&missing_file) {
        Err(SettingsError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
            println!("dosya yok → varsayılan ayarlarla devam")
        }
        other => println!("beklenmeyen: {other:?}"),
    }
}

fn quick_sum(text: &str) -> Result<u32, Box<dyn Error>> {
    let mut total = 0;
    for part in text.split(',') {
        total += part.trim().parse::<u32>()?;
    }
    if total == 0 {
        return Err("toplam sıfır olamaz".into());
    }
    Ok(total)
}

fn show_box_dyn_error() {
    println!("-- 5. Box<dyn Error> --");
    println!("quick_sum(\"1, 2, 3\") = {:?}", quick_sum("1, 2, 3").ok());
    for input in ["1, iki", "0, 0"] {
        if let Err(e) = quick_sum(input) {
            let is_number_error = e.downcast_ref::<ParseIntError>().is_some();
            println!("{input:?} → {e} (ParseIntError mı? {is_number_error})");
        }
    }
}

fn show_when_unwrap_is_ok() {
    println!("-- 6. meşru expect --");
    let localhost: std::net::Ipv4Addr =
        "127.0.0.1".parse().expect("sabit literal geçerli bir IPv4");
    let numbers = [3, 1, 2];
    let largest = numbers.iter().max().expect("dizi boş değil");
    println!("yerel = {localhost}, en büyük = {largest}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_text_becomes_settings() {
        let settings = parse_settings("port=80\nazami_baglanti=5").unwrap();
        let expected = Settings {
            name: String::from("adsız"),
            port: 80,
            max_connections: 5,
        };
        assert_eq!(settings, expected);
    }

    #[test]
    fn format_error_has_line_number() {
        let error = parse_settings("port=80\n\nesittir yok").unwrap_err();
        assert!(matches!(error, SettingsError::Format { line: 3, .. }));
    }

    #[test]
    fn number_error_gives_cause_with_source() {
        let error = parse_settings("port=abc\nazami_baglanti=5").unwrap_err();
        assert!(matches!(error, SettingsError::Number { key: "port", .. }));
        let root_cause = error.source().expect("Number varyantı kaynağını taşır");
        assert!(root_cause.downcast_ref::<ParseIntError>().is_some());
    }

    #[test]
    fn missing_key_is_its_own_variant() {
        let error = parse_settings("port=80").unwrap_err();
        assert!(matches!(error, SettingsError::Missing("azami_baglanti")));
        assert!(error.source().is_none());
    }

    #[test]
    fn collecting_errors_does_not_stop_at_first() {
        let (valid_count, errors) = collect_errors("a=1\nbozuk\n# yorum\nb=2\nbozuk2");
        assert_eq!(valid_count, 2);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn loading_temp_file_works_and_removes_it() {
        let mut used_path = None;
        let result = with_temp_file("test", "port=1\nazami_baglanti=2", |path| {
            used_path = Some(path.to_path_buf());
            load_from_file(path)
        });
        let settings = result
            .expect("geçici dizin yazılabilir")
            .expect("içerik geçerli");
        assert_eq!((settings.port, settings.max_connections), (1, 2));
        assert!(!used_path.expect("closure çağrıldı").exists());
    }

    #[test]
    fn missing_file_becomes_io_variant() {
        let path = std::env::temp_dir().join("rust_study_kesinlikle_yok.conf");
        assert!(matches!(load_from_file(&path), Err(SettingsError::Io(_))));
    }
}

// Learn more
//
// You can learn custom error types from these sources:
//   - Rust by Example, "Defining an error type": https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html
//   - Effective Rust, "Prefer idiomatic Error types": https://effective-rust.com/errors.html
//   - BurntSushi, "Error Handling in Rust": https://burntsushi.net/rust-error-handling/
//   - Comprehensive Rust, "thiserror": https://google.github.io/comprehensive-rust/error-handling/thiserror.html
//   - Easy: KodeKloud, "Creating Custom Error Types": https://notes.kodekloud.com/docs/Rust-Programming/Collections-Error-Handling/Creating-Custom-Error-Types
//
// You can learn Option and Result together from these sources:
//   - Rust by Example, "Pulling Results out of Options": https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/option_result.html
//   - Standard library docs, "std::option::Option::ok_or_else": https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or_else
//   - Effective Rust, "Prefer Option and Result transforms over explicit match expressions": https://effective-rust.com/transform.html
//   - Easy: LogRocket, "Understanding Rust Option and Result enums": https://blog.logrocket.com/understanding-rust-option-results-enums/
//
// You can learn how to stop at the first error or collect all errors from these sources:
//   - Rust by Example, "Iterating over Results": https://doc.rust-lang.org/rust-by-example/error/iter_result.html
//   - Standard library docs, "std::result": https://doc.rust-lang.org/std/result/index.html
//   - Standard library docs, "std::iter::Iterator::partition": https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.partition
//
// You can learn how to load a file from these sources:
//   - The Rust Book, "Reading a File": https://doc.rust-lang.org/book/ch12-02-reading-a-file.html
//   - Rust by Example, "open": https://doc.rust-lang.org/rust-by-example/std_misc/file/open.html
//   - Standard library docs, "std::fs::read_to_string": https://doc.rust-lang.org/std/fs/fn.read_to_string.html
//   - Easy: Programiz, "Rust File Handling": https://www.programiz.com/rust/file-handling
//   - Easy: Easy Rust, "Using files": https://dhghomon.github.io/easy_rust/Chapter_64.html
//
// You can learn Box<dyn Error> from these sources:
//   - Rust by Example, "Boxing errors": https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html
//   - Comprehensive Rust, "Dynamic Error Types": https://google.github.io/comprehensive-rust/error-handling/error.html
//   - Effective Rust, "Prefer idiomatic Error types": https://effective-rust.com/errors.html
//   - The Rust Book, "Where to Use the ? Operator": https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#where-to-use-the--operator
//   - Easy: Easy Rust, "Box around traits": https://dhghomon.github.io/easy_rust/Chapter_54.html
//
// You can learn when unwrap is OK from these sources:
//   - BurntSushi, "Using unwrap() in Rust is Okay": https://burntsushi.net/unwrap/
//   - The Rust Book, "To panic! or Not to panic!": https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html
//   - Effective Rust, "Don't panic": https://effective-rust.com/panic.html
//   - Easy: Programiz, "Rust unwrap() and expect()": https://www.programiz.com/rust/unwrap-and-expect
