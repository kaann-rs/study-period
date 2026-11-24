// Kitap: 18. bölüm, Desenler (Patterns)

mod deep_dive;

fn main() {
    show_pattern_places();
    show_refutable_patterns();
    show_pattern_syntax();
    deep_dive::run();
}

fn describe_point(&(x, y): &(i32, i32)) -> String {
    format!("({x}, {y})")
}

fn show_pattern_places() {
    let (a, b, c) = (1, 2, 3);
    println!("let destructuring: {a} {b} {c}");

    let number = 7;
    let even_or_odd = match number % 2 {
        0 => "çift",
        _ => "tek",
    };
    println!("match: {number} -> {even_or_odd}");

    let color: Option<&str> = Some("mavi");
    if let Some(color_name) = color {
        println!("if let: renk {color_name}");
    } else {
        println!("if let: renk yok");
    }

    let mut stack = vec![1, 2, 3];
    print!("while let (pop): ");
    while let Some(top) = stack.pop() {
        print!("{top} ");
    }
    println!();

    print!("for destructuring: ");
    for (i, letter) in ['a', 'b', 'c'].iter().enumerate() {
        print!("{i}:{letter} ");
    }
    println!();

    println!("fn param destructuring: {}", describe_point(&(3, 4)));
}

fn show_refutable_patterns() {
    let data: Option<i32> = Some(99);
    let Some(x) = data else {
        println!("veri yok, çık");
        return;
    };
    println!("let-else ile bağlandı: x = {x}");
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Color(u8, u8, u8),
    Write(String),
}

struct Point {
    x: i32,
    y: i32,
}

fn show_pattern_syntax() {
    for n in [1, 5, 7, 42] {
        let label = match n {
            1 | 2 => "bir ya da iki",
            3..=9 => "üç ile dokuz arası",
            _ => "başka",
        };
        println!("match {n}: {label}");
    }

    let p = Point { x: 0, y: 7 };
    let Point { x, y } = p;
    println!("struct destructuring: x={x} y={y}");
    let p2 = Point { x: 10, y: 20 };
    match p2 {
        Point { x: 0, y } => println!("y ekseninde, y={y}"),
        Point { x, .. } => println!("x={x}, geri kalanı umursama"),
    }

    for message in [
        Message::Quit,
        Message::Move { x: 3, y: 4 },
        Message::Color(255, 0, 0),
        Message::Write(String::from("selam")),
    ] {
        match message {
            Message::Quit => println!("enum: çık"),
            Message::Move { x, y } => println!("enum: taşı ({x},{y})"),
            Message::Color(r, _, _) => {
                println!("enum: renk, kırmızı={r} (yeşil/maviyi _ ile yok say)")
            }
            Message::Write(s) => println!("enum: yaz '{s}'"),
        }
    }

    if let Message::Color(r, g, b) = Message::Color(10, 20, 30) {
        println!("tam renk destructuring: r={r} g={g} b={b}");
    }

    let pair = (2, -2);
    match pair {
        (x, y) if x + y == 0 => println!("guard: toplamı sıfır"),
        (x, _) if x % 2 == 0 => println!("guard: ilk çift"),
        _ => println!("guard: diğer"),
    }

    let id = 5;
    match id {
        n @ 1..=5 => println!("@ bağlama: {n} küçük aralıkta"),
        n @ 6..=10 => println!("@ bağlama: {n} büyük aralıkta"),
        _ => println!("@ bağlama: aralık dışı"),
    }

    let x = Some(10);
    let y = 20;
    match x {
        Some(y) => println!("shadowing: içteki y={y} (dıştaki 20 DEĞİL)"),
        None => {}
    }
    println!("shadowing: dıştaki y hâlâ {y}");
}
