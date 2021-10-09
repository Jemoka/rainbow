use indicatif::ProgressIterator;
use rand::{distributions::Alphanumeric, Rng};

use super::utils;

#[derive(Debug, Hash)]
pub struct Rainbow {
    number_samples: u32,
    chain_length: u32,
    rainbow_table: Vec<(String, String)>
}

impl Rainbow {
    pub fn create(samples: u32, length: u32, seeds: Option<Vec<String>>) -> Result<Self, &'static str> {
        let mut rainbow_table: Vec<(String, String)> = Vec::new();
        let seed_vec:Vec<String>;

        if let Some(seeds_arr) = seeds {
            seed_vec = seeds_arr;
        } else {
            seed_vec = (0..length).map(|_| rand::thread_rng()
                                       .sample_iter(&Alphanumeric)
                                       .take(rand::thread_rng()
                                                   .gen_range(0..25))
                                       .map(char::from)
                                       .collect()).collect();
        }

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
}


