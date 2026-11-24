// Kitap: 2. bölüm, Yaygın Kavramlar

fn main() {
    show_variables();
    show_shadowing();
    show_data_types();
    show_control_flow();

    println!("add(2, 3) = {}", add(2, 3));
    println!("five() = {}", five());
}

fn show_variables() {
    let x = 5;
    println!("x = {x}");

    let mut y = 5;
    println!("ilk y = {y}");
    y = 6;
    println!("yeni y = {y}");

    const MAX_SCORE: u32 = 100_000;
    println!("MAX_SCORE = {MAX_SCORE}");
}

fn show_shadowing() {
    let spaces = "   ";
    let spaces = spaces.len();
    println!("boşluk sayısı = {spaces}");
}

fn show_data_types() {
    let whole: i32 = -42;
    let decimal: f64 = 3.15;
    let is_true: bool = true;
    let letter: char = 'ş';
    println!("{whole} {decimal} {is_true} {letter}");

    let tuple: (i32, f64, char) = (1, 2.0, 'x');
    let (a, b, c) = tuple;
    println!("{a} {b} {c}, .1 = {}", tuple.1);

    let array = [10, 20, 30];
    println!("ilk = {}", array[0]);
}

fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn five() -> i32 {
    let answer = {
        let base = 2;
        base + 3
    };
    answer
}

fn show_control_flow() {
    let number = 7;

    let label = if number % 2 == 0 { "çift" } else { "tek" };
    println!("{number} → {label}");

    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 5 {
            break counter * 2;
        }
    };
    println!("loop sonucu = {result}");

    let mut countdown = 3;
    while countdown > 0 {
        print!("{countdown} ");
        countdown -= 1;
    }
    println!("kalk!");

    for i in 1..4 {
        print!("{i} ");
    }
    println!();

    let array = [10, 20, 30];
    for item in array {
        print!("{item} ");
    }
    println!();
}
