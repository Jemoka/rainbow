//! A Rainbow Table!

use indicatif::ProgressIterator;
use rand::{distributions::Alphanumeric, Rng};

use super::utils;

use std::thread;
use md5;

/// Represents a Fully-Hashable Rainbow Table
#[derive(Debug, Hash)]
pub struct Rainbow {
    number_samples: u32,
    chain_length: u32,
    rainbow_table: Vec<(String, String)>
}

impl Rainbow {
    /// Returns a Result containing, when seed size is correct, a Rainbow Table
    /// If `seeds` argument is not passed, random strings with lengths between
    /// 0 and 25 chars will be generated.
    ///
    /// # Arguments
    ///
    /// * `samples`: a u32 representing the number of seed samples in the table
    /// * `length`: a u32 representing the length of each generated chain
    /// * `seeds`: an Option<Vec<String>> perhaps containing a vector of seed strings
    /// * `threads`: an Option<u8> perhaps containing an integer number of workers
    ///
    /// # Examples
    ///
    /// ```
    /// Rainbow::create(10000, 1000, None);
    /// ```
    pub fn create(samples: u32,
                  length: u32,
                  seeds: Option<Vec<String>>,
                  threads: Option<u8>) -> Result<Self, &'static str> {

        // let mut rainbow_table: Vec<(String, String)> = Vec::new();
        let workers:u8 = if let Some(r) = threads {r} else {10};
        let mut threads: Vec<thread::Thread> = Vec::new();

        let seed_vec:Vec<String> = if let Some(seeds_arr) = seeds {
            seeds_arr
        } else {
            (0..length).map(|_| rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(rand::thread_rng()
                                  .gen_range(0..25))
                            .map(char::from)
                            .collect())
                .collect()
        };

        for i in (0..samples).progress() {
            if let Some(res) = seed_vec.get(i as usize) {
                rainbow_table.push((res.clone(),
                                    utils::generate_chain(res, length, None)));
            } else {
                return Err("Seed vector length and desired sample count mismatches!");
            };
        }

        return Ok(Rainbow { number_samples: samples, chain_length: length,  rainbow_table: rainbow_table});
    }

    pub fn decode(&self, hash: &str) -> Result<String, &'static str> {
        let mut target: Option<String> = None;
        let mut reduction: u32 = 0;

        let mut curr_hash: String = String::from(hash);
        while target == None && reduction < self.number_samples {
            for elem in &self.rainbow_table {
                if elem.1 == curr_hash {
                    target = Some(elem.0.clone());
                    break;
                }
            }

            reduction += 1;
            curr_hash = format!("{:x}", md5::compute(utils::reduce(curr_hash, reduction)));
        }

        if let Some(res_src) = target {
            let mut current_res: String = res_src.clone();

            for i in 0..self.chain_length {

                let reduction: String = utils::reduce(&current_res, i);
                curr_hash = format!("{:x}", md5::compute(&reduction));

                if curr_hash == hash { return Ok(reduction) }
                else { current_res = format!("{:x}", md5::compute(&reduction)) }
            }

            return Err("Unfortunately, the hash is not in the table.");

        } else {
            return Err("Unfortunately, the hash is not in the table.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rainbow_new_test() {
        let _table = Rainbow::create(1000, 50, None);
    }


    #[test]
    fn rainbow_reduce_test() {
        let _table = Rainbow::create(1000, 50, None);
        if let Ok(_t) = _table {
            let _res = _t.decode("hewo");
        }
    }
}


