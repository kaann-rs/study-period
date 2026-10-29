use std::ptr;

// Exercises in variable bindings, raw pointers, and closure trait bounds.
// Const generic example: struct S<const N: usize = 0> { data: [u8; N] }
fn main() {
    // Related control-flow constructs: if let, while let, let ... else, and let chains.
    // Bindings are immutable by default; mut allows reassignment.
    let mut first_var: i32 = 10_i32;
    first_var = 101;

    println!("First Variables {}", first_var);

    // SAFETY: s1 points to a live, initialized, aligned i32. Reading copies the
    // value because i32 is Copy; s remains valid until the block ends.
    unsafe {
        let s: i32 = 10;

        let s1: *const i32 = &s as *const i32;

        ptr::read(s1);

        println!("{:?}", *s1);
    }

    test();
}

const N: usize = 0;

struct Color<F>
where
    F: Fn(i8) -> i8,
{
    x: F,
}

impl<F> Color<F>
where
    F: Fn(i8) -> i8,
{
    fn hex(f: F) -> i8 {
        f(1)
    }
}

fn test() {
    let x = |y: i8| -> i8 { y };

    let a: i8 = Color::hex(x);

    println!("{:?}", a);
}
