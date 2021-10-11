use md5;
use indicatif::ProgressBar;

use base64;
use std::cmp;

/// Apply the `deriv`-th reduction function upon string slice pointer `hash`
///
/// # Arguments
///
/// * `hash`: a slice-like u16 digest representing the source hash
/// * `deriv`: u32 representing the n-th reduce function being called
///
/// # Logic
///
/// 1. Take each value of the digest
/// 2. Take the first (mod 30 of deriv, add 1) of the size,
///    and max() that with 4 to prevent high collisions
/// 3. Take each char's utf-8, mod 96 and add 30 to conform charset
///
/// # Examples
///
/// ```
/// reduce([1,2,4,5,11,4,5,9,9,4,8,9,3,4,1,9], 934291) 
/// ```
pub fn reduce(hash: &[u8], deriv: u32) -> Vec<u8> {
    if deriv % 2 == 0 {
        base64::encode(hash).as_bytes().iter()
            .take(cmp::max(deriv % 30 + 1, 4) as usize)
            .map(|i| (((*i as u32 + deriv) % (74)) + 48) as u8)
            .collect::<Vec<u8>>()
    } else {
        hash.iter()
            .take(cmp::max(deriv % 30 + 1, 4) as usize)
            .map(|i| (((*i as u32 + deriv) % (74)) + 48) as u8)
            .collect::<Vec<u8>>()
    } 
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
pub fn generate_chain(src: &[u8], n:u32, bar:Option<&ProgressBar>) -> Vec<u8> {
    let mut hash:[u8;16];
    let mut src:Vec<u8> = src.to_owned();

    assert!(n>=1, "A zero-lengthed chain is not much of a chain.");

    for i in 0..n {
        hash = md5::compute(src).into();
        src = reduce(&hash, i);

        if let Some(b) = bar {(*b).inc(1)}
    }

    let final_hash:[u8;16] = md5::compute(src).into();

    return final_hash.to_vec();
}

