use std::{fmt::Display, io::Write};

pub fn ask_user<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    std::io::stdin()
        .read_line(&mut res)
        .expect("Error in string entered");
    res
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
