use md5;
use base64;
use indicatif::ProgressBar;

/// Apply the `deriv`-th reduction function upon string slice pointer `hash`
///
/// # Arguments
///
/// * `hash`: a slice-like u16 digest representing the source hash
/// * `deriv`: u32 representing the n-th reduce function being called
///
/// # Logic
///
/// 1. Base64 encode the hash digest
/// 2. Take each value of the digest, add `deriv` to its UTF-8 representation
/// 3. Take the first deriv % 30 +1 -th char
///
/// # Examples
///
/// ```
/// reduce([1,2,4,5,11,4,5,9,9,4,8,9,3,4,1,9], 934291) 
/// ```
pub fn reduce(hash: &[u8], deriv: u32) -> String {
    base64::encode(hash)
        .as_bytes()
        .iter()
        .map (|&val| val as char)
        .take(((deriv % 30) + 1) as usize)
        .collect::<String>()
        // .map(|&val| (((val as u32 + deriv) % (96)) + 30) as u8 as char)
}


/// Generate the hash chain of `n`-length from a `src` source string.
/// Optionally pass a pointer to a indicatif::Progressbar to update
///
/// # Arguments
///
/// * `src`: a str-like reference representing the source string of chain
/// * `n`: u32 representing the length of desired chain
/// * `bar`: Option perhaps containing pointer to a indicatif `ProgressBar` to update
///
/// # Examples
///
/// ```
/// generate_chain("hallo", 10000, None)
/// ```
pub fn generate_chain<T: AsRef<str>>(src: T, n:u32, bar:Option<&ProgressBar>) -> String {
    let mut vec:[u8;16] = md5::compute(src.as_ref()).into();
    for i in 0..n {
        vec = md5::compute(reduce(&vec, i)).into();
        // hash = format!("{:x}", md5::compute(reduce(hash, i)));
        if let Some(b) = bar {(*b).inc(1)}
    }

    return vec.iter().map(|i| *i as char).collect();
}

