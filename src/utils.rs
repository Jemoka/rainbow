//! Utilities for Rainbow Table Generation

use md5;
use indicatif::ProgressBar;

/// Apply the `deriv`-th reduction function upon string slice pointer `hash`
///
/// # Logic
/// 1. Take each character of `hash`, add `deriv` to its UTF-8 representation
/// 2. Take mod 96 and add 30 to coerce result between chars 0 => ~
///
/// # Examples
///
/// ```
/// reduce("this is a test", 934291) 
/// ```

pub fn reduce<T: AsRef<str>>(hash: T, deriv: u32) -> String {
    let hashref = hash.as_ref();
    hashref.bytes()
           .map(|i| (((i as u32 + deriv) % (96)) + 30) as u8 as char)
           .collect::<String>()
}


/// Generate the hash chain of `n`-length from a `src` source string.
/// Optionally pass a pointer to a indicativ::Progressbar to update
///
/// # Examples
///
/// ```
/// generate_chain("hallo", 10000, None)
/// ```

pub fn generate_chain<T: AsRef<str>>(src: T, n:u32, bar:Option<&ProgressBar>) -> String {
    let mut hash:String = format!("{:x}", md5::compute(src.as_ref()));
    for i in 0..n {
        hash = format!("{:x}", md5::compute(reduce(hash, i)));
        if let Some(b) = bar {(*b).inc(1)}
    }

    return hash;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_test() {
        assert_eq!(reduce("this is a test", 934291), "E9:DQ:DQ2QE6DE");
    }


    #[test]
    fn generate_chain_test() {
        assert_eq!(generate_chain("hallo", 10000, None), "0b6fdc5e8b76c91911a6705d5b88c195");
    }
}

