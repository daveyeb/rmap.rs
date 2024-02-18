pub trait ModChar {
    fn is_punct(ch: char) -> bool;
}

impl ModChar for char {
    fn is_punct(ch: char) -> bool {
        ch.is_ascii_punctuation()
    }
}

pub fn capitalize(s1: String) -> String {
    let mut c = s1.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn to_slice(src: &Vec<u8>) -> [u8; 16] {
    let mut result = [0; 16];
    for n in 0..16 {
        result[n] = *src.get(n).unwrap();
    }

    result
}
