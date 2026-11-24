// Kitap: 21. bölüm, Alt Tür İlişkisi ve Varyans

fn main() {
    show_subtyping();
    show_covariance();
    show_invariance();
}

fn show_subtyping() {
    fn need_short(x: &str) -> usize {
        x.len()
    }

    let static_text: &'static str = "program boyu yaşarım";
    println!(
        "subtyping: &'static → &'a kabul edildi, len = {}",
        need_short(static_text)
    );
}

fn show_covariance() {
    fn count_words(words: &Vec<&str>) -> usize {
        words.len()
    }

    let words: Vec<&'static str> = vec!["a", "b", "c"];
    println!(
        "covariance: paylaşımlı & esnek, eleman sayısı = {}",
        count_words(&words)
    );
}

struct MutStr<'a, 'b> {
    s: &'a mut &'b str,
}

fn show_invariance() {
    let mut s = "hello";
    *MutStr { s: &mut s }.s = "world";
    println!("invariance: MutStr<'a,'b> ile s = {s}");

    println!("invariance: tek lifetime'da aynı kod E0502 verir (yorumda açıklandı)");
}

// Learn more
//
// You can learn lifetime subtyping from these sources:
//   - Rust Reference, "Subtyping and variance": https://doc.rust-lang.org/reference/subtyping.html
//   - Rustonomicon, "Subtyping and Variance": https://doc.rust-lang.org/nomicon/subtyping.html
//   - okryvyts, "Lifetime subtyping": https://okryvyts.github.io/lifetimes-book/lifetime_subtyping.html
//   - Easy: DEV Community, "Variance - best perspective of understanding lifetime in Rust": https://dev.to/arichyx/variance-best-perspective-of-understanding-lifetime-in-rust-m84
//
// You can learn covariance from these sources:
//   - Rustonomicon, "Variance": https://doc.rust-lang.org/nomicon/subtyping.html#variance
//   - sunshowers, "Formalizing variance": https://lifetime-variance.sunshowers.io/ch01-02-formalizing-variance.html
//   - Crust of Rust (video), "Subtyping and Variance": https://www.youtube.com/watch?v=iVYWDIW71jk
//   - Easy: sunshowers, "Building an intuition": https://lifetime-variance.sunshowers.io/ch01-01-building-an-intuition.html
//   - Easy: DEV Community, "Variance - best perspective of understanding lifetime in Rust": https://dev.to/arichyx/variance-best-perspective-of-understanding-lifetime-in-rust-m84
//
// You can learn invariance from these sources:
//   - Learning Rust by quinedot, "Nested borrows and invariance": https://quinedot.github.io/rust-learning/st-invariance.html
//   - Rust Reference, "Variance": https://doc.rust-lang.org/reference/subtyping.html#variance
//   - sunshowers, "Formalizing variance": https://lifetime-variance.sunshowers.io/ch01-02-formalizing-variance.html
//   - Easy: DEV Community, "Variance - best perspective of understanding lifetime in Rust": https://dev.to/arichyx/variance-best-perspective-of-understanding-lifetime-in-rust-m84
