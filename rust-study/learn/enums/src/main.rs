// Kitap: 5. bölüm, Enum'lar ve Desen Eşleme

#[derive(Debug)]
enum IpAddress {
    V4(u8, u8, u8, u8),
    V6(String),
    Unknown,
}

#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

impl Message {
    fn run(&self) {
        match self {
            Message::Quit => println!("çık"),
            Message::Move { x, y } => println!("taşı {x},{y}"),
            Message::Write(text) => println!("yaz: {text}"),
            Message::ChangeColor(r, g, b) => println!("renk {r},{g},{b}"),
        }
    }
}

#[derive(Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

mod deep_dive;

fn main() {
    show_enums();
    show_option();
    show_match();
    show_all_cases();
    show_if_let();
    deep_dive::run();
}

fn show_enums() {
    let addresses = [
        IpAddress::V4(127, 0, 0, 1),
        IpAddress::V6(String::from("::1")),
        IpAddress::Unknown,
    ];
    for address in addresses {
        let description = match address {
            IpAddress::V4(a, b, c, d) => format!("IPv4 {a}.{b}.{c}.{d}"),
            IpAddress::V6(s) => format!("IPv6 {s}"),
            IpAddress::Unknown => String::from("bilinmeyen"),
        };
        println!("{description}");
    }

    for message in [
        Message::Quit,
        Message::Move { x: 1, y: 2 },
        Message::Write(String::from("selam")),
        Message::ChangeColor(255, 0, 0),
    ] {
        message.run();
    }
}

fn show_option() {
    let some_number: Option<i32> = Some(5);
    let nothing: Option<i32> = None;

    let result = match some_number {
        Some(n) => n + 1,
        None => 0,
    };
    println!("sonuç = {result}, nothing = {nothing:?}");
    println!("nothing.unwrap_or(-1) = {}", nothing.unwrap_or(-1));
}

fn show_match() {
    let dice = 9;
    match dice {
        3 => println!("üç!"),
        7 => println!("yedi!"),
        other => println!("diğer: {other}"),
    }
    let n = 4;
    let even_or_odd = match n % 2 {
        0 => "çift",
        _ => "tek",
    };
    println!("{n} → {even_or_odd}");
}

fn show_all_cases() {
    for direction in [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ] {
        let arrow = match direction {
            Direction::North => "↑",
            Direction::South => "↓",
            Direction::East => "→",
            Direction::West => "←",
        };
        print!("{arrow} ");
    }
    println!();
}

fn show_if_let() {
    let setting: Option<u8> = Some(3);

    if let Some(v) = setting {
        println!("ayar = {v}");
    } else {
        println!("ayar yok");
    }

    let Some(v) = setting else {
        println!("değer yok, çık");
        return;
    };
    println!("let-else ile v = {v}");
}
