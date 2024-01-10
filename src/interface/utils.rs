use std::fmt::Display;
use std::io::Write;
use std::str::FromStr;

pub fn ask_user<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    std::io::stdin()
        .read_line(&mut res)
        .expect("Error in string entered");
    res.trim().to_string()
}

pub fn ask_user_parse<T: Display, U: FromStr>(question: T) -> Option<U> {
    loop {
        let res = ask_user(&question);
        if res.is_empty() {
            return None;
        }
        if let Ok(res) = res.parse() {
            return Some(res);
        }
        println!("Unable to parse reply to {:?}", std::any::type_name::<U>());
    }
}

pub fn ask_user_nonempty<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    let mut init = true;
    while init || res.is_empty() {
        std::io::stdin()
            .read_line(&mut res)
            .expect("Error in string entered");
        if !init {
            println!("Answer is empty");
        }
        init = false;
    }
    res.trim().to_string()
}
