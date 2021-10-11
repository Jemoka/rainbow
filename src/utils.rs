use md5;
use indicatif::ProgressBar;

use rand_core::{SeedableRng, RngCore};
use rand_chacha::ChaChaRng;

/// Apply the `deriv`-th reduction function upon string slice pointer `hash`
///
/// # Arguments
///
/// * `hash`: a slice-like u16 digest representing the source hash
/// * `deriv`: u32 representing the n-th reduce function being called
///
/// # Logic
///
/// 1. Take the deriv number to seed a cha cha random number generator
/// 2. Get each output conformed to be within 30 from the generator
/// 3. Return the nth value from the hash, added to another random number
///    mod 96 + 30 to conform charset
///
/// # Examples
///
/// ```
/// reduce([1,2,4,5,11,4,5,9,9,4,8,9,3,4,1,9], 934291) 
/// ```
pub fn reduce(hash: &[u8], deriv: u32) -> Vec<u8> {
    let mut generator = ChaChaRng::seed_from_u64(deriv as u64);
    (0..(generator.next_u32() % 30)).map(|i| {
        let nth:usize = (i as usize) % hash.len();
        ((hash[nth] as u32 + generator.next_u32())%96 + 30) as u8
    }).collect()
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

