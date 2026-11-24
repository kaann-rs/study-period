// Kitap: 18.1. bölüm, Desenler: Derinleşme

pub fn run() {
    println!("\n== Derinleşme: Patterns — envanter komut ayrıştırıcısı ==");
    show_command_parser();
    show_binding_modes();
    show_nested_at_and_guard();
    show_slice_patterns();
    show_exhaustive_match();
}

#[derive(Debug, PartialEq)]
enum Command {
    Add { name: String, count: u32 },
    Remove { name: String },
    List,
    Note(String),
}

#[derive(Debug, PartialEq)]
enum ParseError {
    Empty,
    InvalidCount(String),
    WrongArguments { command: String, given: usize },
    Unknown(String),
}

fn parse(line: &str) -> Result<Command, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.as_slice() {
        [] => Err(ParseError::Empty),
        ["listele"] => Ok(Command::List),
        ["sil", name] => Ok(Command::Remove {
            name: name.to_string(),
        }),
        ["ekle", name] => Ok(Command::Add {
            name: name.to_string(),
            count: 1,
        }),
        ["ekle", name, count] => {
            let Ok(count @ 1..) = count.parse::<u32>() else {
                return Err(ParseError::InvalidCount(count.to_string()));
            };
            Ok(Command::Add {
                name: name.to_string(),
                count,
            })
        }
        ["not", first, rest @ ..] => {
            let mut text = first.to_string();
            for word in rest {
                text.push(' ');
                text.push_str(word);
            }
            Ok(Command::Note(text))
        }
        [
            command @ ("ekle" | "sil" | "listele" | "not"),
            arguments @ ..,
        ] => Err(ParseError::WrongArguments {
            command: command.to_string(),
            given: arguments.len(),
        }),
        [unknown, ..] => Err(ParseError::Unknown(unknown.to_string())),
    }
}

#[derive(Debug, PartialEq)]
struct Record {
    name: String,
    count: u32,
}

fn apply(stock: &mut Vec<Record>, command: Command) -> String {
    match command {
        Command::Add { name, count } => {
            if let Some(Record { count: current, .. }) =
                stock.iter_mut().find(|record| record.name == name)
            {
                *current = current.saturating_add(count);
                format!("{name}: {current} adet oldu")
            } else {
                let message = format!("{name} eklendi ({count} adet)");
                stock.push(Record { name, count });
                message
            }
        }
        Command::Remove { name } => match stock.iter().position(|record| record.name == name) {
            Some(index) => {
                let Record { name, count } = stock.remove(index);
                format!("{name} silindi ({count} adet vardı)")
            }
            None => format!("{name} bulunamadı"),
        },
        Command::List => match stock.as_slice() {
            [] => String::from("envanter boş"),
            [only] => format!("1 kalem: {}", only.name),
            [first, .., last] => format!("{} kalem: {} … {}", stock.len(), first.name, last.name),
        },
        Command::Note(text) => format!("not alındı: {text}"),
    }
}

fn show_command_parser() {
    println!("-- 1. Komut ayrıştırıcı");
    let mut stock = Vec::new();
    for line in [
        "listele",
        "ekle vida 40",
        "ekle somun",
        "ekle vida 2",
        "not raf  3 dolu",
        "listele",
        "sil somun",
        "listele",
        "ekle vida çok",
        "sil",
        "boya kırmızı",
        "   ",
    ] {
        match parse(line) {
            Ok(command) => println!("{line:?} → {}", apply(&mut stock, command)),
            Err(error) => println!("{line:?} → hata: {error:?}"),
        }
    }
}

fn show_binding_modes() {
    println!("-- 2. Bağlama modları");
    let mut record = Record {
        name: String::from("vida"),
        count: 10,
    };

    let Record { name, count } = &record;
    println!("ödünç: {name} × {count}");

    let Record { ref name, count } = record;
    println!("ref: {name} × {count}, kayıt yerinde: {}", record.name);

    let Record { count, .. } = &mut record;
    *count += 5;
    let Record {
        count: ref mut counter,
        ..
    } = record;
    *counter *= 2;
    println!("güncel: {} × {}", record.name, record.count);

    let selected: &Option<String> = &Some(String::from("somun"));
    match selected {
        Some(name) => println!("seçili: {name} ({} bayt)", name.len()),
        None => println!("seçim yok"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MouseButton {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq)]
enum Event {
    Key {
        character: char,
        ctrl: bool,
    },
    Click {
        position: Point,
        button: MouseButton,
    },
    Scroll(i32),
}

fn describe(event: &Event) -> String {
    match event {
        Event::Key {
            character: 'q',
            ctrl: true,
        } => String::from("çıkış"),
        Event::Key {
            character: letter @ ('a'..='z' | 'A'..='Z'),
            ctrl: false,
        } => format!("harf: {letter}"),
        Event::Key { character, ctrl } => format!("tuş: {character:?} ctrl={ctrl}"),
        Event::Click {
            position: Point { x: 0, y },
            button: MouseButton::Left,
        } => format!("sol kenar çubuğu, satır {y}"),
        Event::Click {
            position: point @ Point { x, y },
            ..
        } if x == y => format!("köşegende tık: {point:?}"),
        Event::Click {
            position: Point { x, y },
            button,
        } => format!("tık ({x},{y}) {button:?}"),
        Event::Scroll(amount @ ..=-1) => format!("yukarı {}", amount.unsigned_abs()),
        Event::Scroll(0) => String::from("kaydırma yok"),
        Event::Scroll(amount @ 1..) => format!("aşağı {amount}"),
    }
}

fn show_nested_at_and_guard() {
    println!("-- 3. İç içe yapı çözme, @ ve guard");
    let events = [
        Event::Key {
            character: 'q',
            ctrl: true,
        },
        Event::Key {
            character: 'q',
            ctrl: false,
        },
        Event::Key {
            character: '7',
            ctrl: false,
        },
        Event::Click {
            position: Point { x: 0, y: 12 },
            button: MouseButton::Left,
        },
        Event::Click {
            position: Point { x: 5, y: 5 },
            button: MouseButton::Right,
        },
        Event::Click {
            position: Point { x: 3, y: 8 },
            button: MouseButton::Right,
        },
        Event::Scroll(-3),
        Event::Scroll(2),
    ];
    for event in &events {
        println!("{}", describe(event));
    }

    let is_allowed = false;
    let result = match 1 {
        1 | 2 if is_allowed => "bir ya da iki, izinli",
        _ => "diğer",
    };
    println!("guard önceliği: {result}");
}

fn summarize(numbers: &[i32]) -> String {
    match numbers {
        [] => String::from("boş"),
        [only] => format!("tek eleman: {only}"),
        [first, last] => format!("iki eleman: {first} ve {last}"),
        [first, middle @ .., last] => format!("{first} … {last} (arada {} eleman)", middle.len()),
    }
}

fn sum(numbers: &[i32]) -> i32 {
    match numbers {
        [] => 0,
        [first, rest @ ..] => first + sum(rest),
    }
}

fn show_slice_patterns() {
    println!("-- 4. Dilim desenleri");
    for slice in [&[][..], &[7], &[1, 2], &[3, 1, 4, 1, 5]] {
        println!("{slice:?} → {} (toplam {})", summarize(slice), sum(slice));
    }

    let color: [u8; 3] = [255, 128, 0];
    let [r, g, b] = color;
    let parts = ["tasi", "vida", "raf-2"];
    let [command, arguments @ ..] = parts.as_slice() else {
        return;
    };
    println!(
        "rgb = {r},{g},{b} | komut = {command}, {} argüman",
        arguments.len()
    );
}

fn show_exhaustive_match() {
    println!("-- 5. Bütünlük ve matches!");
    let stock: Vec<Record> = [("vida", 42), ("pul", 0), ("somun", 3)]
        .into_iter()
        .map(|(name, count)| Record {
            name: name.to_string(),
            count,
        })
        .collect();

    let sold_out = stock
        .iter()
        .filter(|record| matches!(record, Record { count: 0, .. }))
        .count();
    let low_stock: Vec<&str> = stock
        .iter()
        .filter(|record| matches!(record.count, 1..=5))
        .map(|record| record.name.as_str())
        .collect();
    let short_names_in_stock = stock
        .iter()
        .any(|record| matches!(record, Record { name, count: 1.. } if name.len() <= 4));
    println!(
        "tükenen: {sold_out}, azalan: {low_stock:?}, kısa adlı dolu var mı: {short_names_in_stock}"
    );

    let Some(Record {
        name,
        count: count @ 1..,
    }) = stock.first()
    else {
        println!("gösterilecek dolu kayıt yok");
        return;
    };
    println!("ilk dolu kayıt: {name} ({count})");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_values_do_not_panic() {
        assert_eq!(describe(&Event::Scroll(i32::MIN)), "yukarı 2147483648");
        let mut stock = Vec::new();
        for _ in 0..2 {
            apply(&mut stock, parse("ekle vida 4000000000").unwrap());
        }
        assert_eq!(stock[0].count, u32::MAX);
    }

    #[test]
    fn valid_commands_are_parsed() {
        assert_eq!(parse("listele"), Ok(Command::List));
        assert_eq!(
            parse("  ekle   vida  40 "),
            Ok(Command::Add {
                name: String::from("vida"),
                count: 40
            })
        );
        assert_eq!(
            parse("ekle somun"),
            Ok(Command::Add {
                name: String::from("somun"),
                count: 1
            })
        );
        assert_eq!(
            parse("not raf 3 dolu"),
            Ok(Command::Note(String::from("raf 3 dolu")))
        );
    }

    #[test]
    fn bad_lines_give_the_right_error() {
        assert_eq!(parse("   "), Err(ParseError::Empty));
        assert_eq!(
            parse("ekle vida sıfır"),
            Err(ParseError::InvalidCount(String::from("sıfır")))
        );
        assert_eq!(
            parse("ekle vida 0"),
            Err(ParseError::InvalidCount(String::from("0")))
        );
        assert_eq!(
            parse("sil"),
            Err(ParseError::WrongArguments {
                command: String::from("sil"),
                given: 0
            })
        );
        assert_eq!(
            parse("boya kırmızı"),
            Err(ParseError::Unknown(String::from("boya")))
        );
    }

    #[test]
    fn adding_increases_existing_record() {
        let mut stock = Vec::new();
        for line in ["ekle vida 40", "ekle vida 2", "ekle somun"] {
            apply(&mut stock, parse(line).unwrap());
        }
        assert_eq!(stock.len(), 2);
        assert_eq!(stock[0].count, 42);
        assert_eq!(apply(&mut stock, Command::List), "2 kalem: vida … somun");
        assert_eq!(
            apply(&mut stock, parse("sil vida").unwrap()),
            "vida silindi (42 adet vardı)"
        );
        assert_eq!(apply(&mut stock, Command::List), "1 kalem: somun");
    }

    #[test]
    fn slice_patterns_cover_every_length() {
        assert_eq!(summarize(&[]), "boş");
        assert_eq!(summarize(&[7]), "tek eleman: 7");
        assert_eq!(summarize(&[1, 2]), "iki eleman: 1 ve 2");
        assert_eq!(summarize(&[3, 1, 4, 1, 5]), "3 … 5 (arada 3 eleman)");
        assert_eq!(sum(&[]), 0);
        assert_eq!(sum(&[3, 1, 4, 1, 5]), 14);
    }

    #[test]
    fn events_match_the_most_specific_arm() {
        let click = |x, y, button| Event::Click {
            position: Point { x, y },
            button,
        };
        assert_eq!(
            describe(&click(0, 4, MouseButton::Left)),
            "sol kenar çubuğu, satır 4"
        );
        assert_eq!(
            describe(&click(0, 0, MouseButton::Right)),
            "köşegende tık: Point { x: 0, y: 0 }"
        );
        assert_eq!(
            describe(&click(0, 4, MouseButton::Right)),
            "tık (0,4) Right"
        );
        assert_eq!(describe(&Event::Scroll(0)), "kaydırma yok");
        assert_eq!(describe(&Event::Scroll(-3)), "yukarı 3");
        let letter = Event::Key {
            character: 'Z',
            ctrl: false,
        };
        assert_eq!(describe(&letter), "harf: Z");
    }
}
