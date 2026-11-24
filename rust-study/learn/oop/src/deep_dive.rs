// Kitap: 17.1. bölüm, Nesne Yönelimli Özellikler: Derinleşme

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

pub fn run() {
    println!("\n== Derinleşme: Trait Object'ler ==");
    show_open_system();
    show_dyn_generic_enum();
    show_pointer_choice();
    show_hidden_static();
    show_upcasting_and_any();
    show_composition_and_clone();
}

pub trait Channel: Debug + Any + CloneChannel {
    fn name(&self) -> &'static str;
    fn format(&self, message: &str) -> String;

    fn max_length(&self) -> Option<usize> {
        None
    }

    fn send(&self, message: &str) -> String {
        let body: String = match self.max_length() {
            Some(limit) => message.chars().take(limit).collect(),
            None => message.to_string(),
        };
        format!("[{}] {}", self.name(), self.format(&body))
    }

    fn with_label(self, label: &'static str) -> Labeled<Self>
    where
        Self: Sized,
    {
        Labeled { inner: self, label }
    }
}

#[derive(Debug, Clone)]
struct Email {
    address: String,
}
impl Channel for Email {
    fn name(&self) -> &'static str {
        "eposta"
    }
    fn format(&self, message: &str) -> String {
        format!("{} adresine: {message}", self.address)
    }
}

#[derive(Debug, Clone)]
struct Sms {
    number: String,
}
impl Channel for Sms {
    fn name(&self) -> &'static str {
        "sms"
    }
    fn format(&self, message: &str) -> String {
        format!("{} numarasına: {message}", self.number)
    }
    fn max_length(&self) -> Option<usize> {
        Some(12)
    }
}

#[derive(Default, Clone)]
struct Notifier {
    channels: HashMap<String, Box<dyn Channel>>,
}
impl Notifier {
    fn add(&mut self, channel: impl Channel) {
        self.add_boxed(Box::new(channel));
    }
    fn add_boxed(&mut self, channel: Box<dyn Channel>) {
        self.channels.insert(channel.name().to_string(), channel);
    }
    fn send(&self, channel_name: &str, message: &str) -> Option<String> {
        self.channels
            .get(channel_name)
            .map(|channel| channel.send(message))
    }
    fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.channels.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}

#[derive(Debug, Clone)]
struct Webhook {
    url: String,
}
impl Channel for Webhook {
    fn name(&self) -> &'static str {
        "webhook"
    }
    fn format(&self, message: &str) -> String {
        format!("POST {} gövde={message:?}", self.url)
    }
}

fn sample_notifier() -> Notifier {
    let mut notifier = Notifier::default();
    notifier.add(Email {
        address: String::from("ece@ornek.com"),
    });
    notifier.add(Sms {
        number: String::from("555-0101"),
    });
    notifier.add(Webhook {
        url: String::from("/kanca"),
    });
    notifier
}

fn show_open_system() {
    println!("-- 1. genişletilebilir sistem --");
    let notifier = sample_notifier();
    println!("kayıtlı: {:?}", notifier.names());
    for name in ["eposta", "sms", "webhook", "faks"] {
        match notifier.send(name, "Kargonuz yola çıktı") {
            Some(output) => println!("  {output}"),
            None => println!("  ({name}: böyle bir kanal yok)"),
        }
    }
}

fn send_all_dyn(channels: &[Box<dyn Channel>], message: &str) -> Vec<String> {
    channels
        .iter()
        .map(|channel| channel.send(message))
        .collect()
}

fn send_all_generic<C: Channel>(channels: &[C], message: &str) -> Vec<String> {
    channels
        .iter()
        .map(|channel| channel.send(message))
        .collect()
}

enum FixedChannel {
    Email(Email),
    Sms(Sms),
}
impl FixedChannel {
    fn send(&self, message: &str) -> String {
        match self {
            FixedChannel::Email(e) => e.send(message),
            FixedChannel::Sms(s) => s.send(message),
        }
    }
}

fn show_dyn_generic_enum() {
    println!("-- 2. dyn / generic / enum --");
    let email = || Email {
        address: String::from("a@b.c"),
    };
    let sms = || Sms {
        number: String::from("111"),
    };
    let boxed: Vec<Box<dyn Channel>> = vec![Box::new(email()), Box::new(sms())];
    let fixed = [FixedChannel::Email(email()), FixedChannel::Sms(sms())];
    let enum_output: Vec<String> = fixed
        .iter()
        .map(|channel| channel.send("Merhaba"))
        .collect();
    println!("dyn     : {:?}", send_all_dyn(&boxed, "Merhaba"));
    println!(
        "enum    : aynı çıktı mı? {}",
        enum_output == send_all_dyn(&boxed, "Merhaba")
    );
    println!(
        "generic : {:?}",
        send_all_generic(&[sms(), sms()], "Merhaba")
    );
}

fn notify_once(channel: &dyn Channel, message: &str) -> String {
    channel.send(message)
}

fn show_pointer_choice() {
    println!("-- 3. işaretçi seçimi --");
    let sms = Sms {
        number: String::from("222"),
    };
    println!("&dyn : {}", notify_once(&sms, "kısa"));

    let shared: Rc<dyn Channel> = Rc::new(sms);
    let urgent_list = [Rc::clone(&shared)];
    let daily_list = [Rc::clone(&shared)];
    let names = (urgent_list[0].name(), daily_list[0].name());
    println!(
        "Rc   : adlar = {names:?}, sahip sayısı = {}",
        Rc::strong_count(&shared)
    );

    let shared_channel: Arc<dyn Channel + Send + Sync> = Arc::new(Webhook {
        url: String::from("/olay"),
    });
    let jobs: Vec<_> = (1..=2)
        .map(|job_number| {
            let channel = Arc::clone(&shared_channel);
            thread::spawn(move || channel.send(&format!("iş {job_number}")))
        })
        .collect();
    for job in jobs {
        println!("Arc  : {}", job.join().expect("iş parçacığı panik etmez"));
    }
}

trait Template {
    fn apply(&self, message: &str) -> String;
}
struct Prefix<'a>(&'a str);
impl Template for Prefix<'_> {
    fn apply(&self, message: &str) -> String {
        format!("{}{message}", self.0)
    }
}

fn box_template(prefix: &str) -> Box<dyn Template + '_> {
    Box::new(Prefix(prefix))
}

fn show_hidden_static() {
    println!("-- 4. gizli 'static sınırı --");
    let prefix = String::from(">> ");
    let template = box_template(&prefix);
    println!("{}", template.apply("ödünç tutan trait object"));
}

fn can_cut_by_type(channel: &dyn Channel) -> bool {
    let any_value: &dyn Any = channel;
    any_value.is::<Sms>()
}
fn can_cut(channel: &dyn Channel) -> bool {
    channel.max_length().is_some()
}

fn show_upcasting_and_any() {
    println!("-- 5. upcasting ve Any --");
    let sms = Sms {
        number: String::from("333"),
    };
    let channel: &dyn Channel = &sms;
    let debug_text: &dyn Debug = channel;
    println!("Debug yüzü: {debug_text:?}");
    let any_value: &dyn Any = channel;
    println!(
        "downcast: {:?}",
        any_value.downcast_ref::<Sms>().map(|s| s.number.as_str())
    );
    println!(
        "Email'e downcast: {}",
        any_value.downcast_ref::<Email>().is_some()
    );

    let wrapped = sms.clone().with_label("[ACİL]");
    println!(
        "kokulu: sms={} sarılı={}",
        can_cut_by_type(&sms),
        can_cut_by_type(&wrapped)
    );
    println!("doğru : sms={} sarılı={}", can_cut(&sms), can_cut(&wrapped));
}

#[derive(Debug, Clone)]
pub struct Labeled<C> {
    inner: C,
    label: &'static str,
}
impl<C: Channel + Clone> Channel for Labeled<C> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }
    fn format(&self, message: &str) -> String {
        format!("{} {}", self.label, self.inner.format(message))
    }
    fn max_length(&self) -> Option<usize> {
        self.inner.max_length()
    }
}

pub trait CloneChannel {
    fn clone_box(&self) -> Box<dyn Channel>;
}
impl<T: Channel + Clone> CloneChannel for T {
    fn clone_box(&self) -> Box<dyn Channel> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Channel> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

fn show_composition_and_clone() {
    println!("-- 6. bileşim ve klonlama --");
    let urgent = Sms {
        number: String::from("444"),
    }
    .with_label("[ACİL]")
    .with_label("[TEST]");
    println!("{}", urgent.send("Sunucu yanıt vermiyor"));

    let original: Box<dyn Channel> = Box::new(urgent);
    let copy = original.clone();
    println!("kopya: {copy:?}");

    let notifier = sample_notifier();
    let mut backup = notifier.clone();
    backup.add_boxed(copy);
    println!("yedek: {:?}", backup.send("sms", "Deneme"));
    println!("asıl : {:?}", notifier.send("sms", "Deneme"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_channel_needs_no_core_change() {
        let mut notifier = Notifier::default();
        assert_eq!(notifier.send("webhook", "x"), None);
        notifier.add(Webhook {
            url: String::from("/t"),
        });
        assert_eq!(
            notifier.send("webhook", "x").as_deref(),
            Some("[webhook] POST /t gövde=\"x\"")
        );
    }

    #[test]
    fn template_method_cuts_by_limit() {
        let sms = Sms {
            number: String::from("1"),
        };
        let email = Email {
            address: String::from("a@b"),
        };
        assert_eq!(
            sms.send("on iki karakterden uzun"),
            "[sms] 1 numarasına: on iki karak"
        );
        assert!(email.send("on iki karakterden uzun").ends_with("uzun"));
    }

    #[test]
    fn dyn_generic_and_enum_give_same_result() {
        let sms = Sms {
            number: String::from("9"),
        };
        let dyn_result = send_all_dyn(&[Box::new(sms.clone())], "m");
        let generic_result = send_all_generic(std::slice::from_ref(&sms), "m");
        assert_eq!(dyn_result, generic_result);
        assert_eq!(dyn_result[0], FixedChannel::Sms(sms).send("m"));
    }

    #[test]
    fn wrapper_passes_work_to_inner_channel() {
        let wrapped = Sms {
            number: String::from("7"),
        }
        .with_label("!");
        assert_eq!(wrapped.name(), "sms");
        assert_eq!(wrapped.max_length(), Some(12));
        assert_eq!(wrapped.send("selam"), "[sms] ! 7 numarasına: selam");
    }

    #[test]
    fn downcast_misses_wrapper_but_trait_method_does_not() {
        let wrapped = Sms {
            number: String::from("7"),
        }
        .with_label("!");
        assert!(!can_cut_by_type(&wrapped));
        assert!(can_cut(&wrapped));
        let any_value: &dyn Any = &wrapped as &dyn Channel;
        assert!(any_value.is::<Labeled<Sms>>());
    }

    #[test]
    fn boxed_channel_and_notifier_clone_separately() {
        let original = sample_notifier();
        let mut copy = original.clone();
        copy.channels.remove("sms");
        assert_eq!(original.names(), vec!["eposta", "sms", "webhook"]);
        assert_eq!(copy.names(), vec!["eposta", "webhook"]);
    }

    #[test]
    fn borrowing_trait_object_lives_as_long_as_its_data() {
        let prefix = String::from("# ");
        assert_eq!(box_template(&prefix).apply("başlık"), "# başlık");
    }
}
