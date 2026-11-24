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
