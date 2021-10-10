//! A Rainbow Table!

use md5;
use std::fs;
use rand::{distributions::Alphanumeric, Rng};

use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_json;

use threadpool::ThreadPool;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

use super::utils;

/// Represents a Fully-Hashable Rainbow Table
#[derive(Debug, Hash, Deserialize, Serialize)]
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
    /// * `threads`: an u8 containing an integer number of workers
    /// * `seeds`: an Option<Vec<String>> perhaps containing a vector of seed strings
    ///
    /// # Examples
    ///
    /// ```
    /// Rainbow::create(10000, 1000, None);
    /// ```
    pub fn create(samples: u32,
                  length: u32,
                  threads: u8,
                  seeds: Option<Vec<String>>) -> Result<Self, &'static str> {

        // Set up seed values
        let seed_vec:Vec<String> = if let Some(seeds_arr) = seeds {
            seeds_arr
        } else {
            (0..samples).map(|_| rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(rand::thread_rng()
                                  .gen_range(0..4))
                            .map(char::from)
                            .collect())
                .collect()
        };

        // Assert length of vector
        assert_eq!(seed_vec.len(),
                   (samples as usize),
                   "seed vector must be the same length as the lengh of table");

        // Set up indicativ
        let indicator = Arc::new(Mutex::new(ProgressBar::new(samples as u64)));

        // Create thread pool and channels
        let pool = ThreadPool::new(threads as usize);
        let (sender, receiver) = channel();

        // Go!
        for item in seed_vec {
            let payload = Arc::clone(&indicator);
            let sender = sender.clone();

            pool.execute(move || {
                let _ = sender.send((item.clone(), utils::generate_chain(item, length, None)));
                (*(payload.lock().unwrap())).inc(1);
            });
        }

        let rainbow_table:Vec<(String, String)> = receiver.iter()
                                                          .take(samples as usize)
                                                          .collect();

        // Finish progress bar
        (*(indicator.clone().lock().unwrap())).finish_and_clear();

        return Ok(Rainbow { number_samples: samples, chain_length: length,  rainbow_table: rainbow_table});
    }


    /// Decode a u8 digest of md5 hash from the table. Returns a `Result`
    /// of either the String password or a static &str message of error.
    ///
    /// # Arguments
    ///
    /// * `src_hash`: a u8 slice pointer representing the m5 digest
    /// * `threads`: a u8 containing the number of workers
    ///
    /// # Examples
    ///
    /// ```
    /// let table = Rainbow::create(10000, 1000, None);
    /// let digest:[u8;16] = md5::compute("q+O{3>&{7.P`My").into();
    /// table.decode(&digest);
    /// ```
    pub fn decode(&self, src_hash: &[u8], threads: u8) -> Result<String, &'static str> {
        // Setup hash and target containers, which must last the lifetime
        // of this function
        let hash_string:String = src_hash.iter().map(|i| *i as char).collect();

        // Create the mutex for the bar and smart pointer for rainbow table
        let indicator = Arc::new(Mutex::new(ProgressBar::new(self.chain_length as u64)));
        let table = Arc::new(self.rainbow_table.clone());

        // Create the thread pool
        let pool = ThreadPool::new(threads as usize);
        let (result_sender, result_receiver) = channel(); // used to send found chains
        let (status_sender, status_receiver) = channel(); // used to report finished status

        // Clone table, length, and hash for distribution into threads
        let chain_length: u32 = self.chain_length;

        for nth_reduction in (0..chain_length).rev() {
            // Clone the appropriate theading references to be processed
            let payload = Arc::clone(&indicator);
            let table = Arc::clone(&table);
            let base_hash: [u8;16] = md5::compute(
                utils::reduce(&src_hash, nth_reduction).as_bytes()
            ).into();
            let result_sender = result_sender.clone();
            let status_sender = status_sender.clone();
            let hash_string = hash_string.clone();

            pool.execute(move || {
                // Compute the first hash
                let mut hash: [u8;16] = base_hash;

                let mut current_reduction: String;

                // Calculate the nth reduction chain
                for i in (nth_reduction+1)..chain_length {
                    current_reduction = utils::reduce(&hash, i);
                    hash = md5::compute(current_reduction.as_bytes()).into();
                }

                // Check if reduced hash is in the table
                for elem in &*table {
                    if elem.1 == hash_string {
                        let _ = result_sender.send((elem.0.clone(), elem.1.clone()));
                        break;
                    }
                }

                (*(payload.lock().unwrap())).inc(1);
                let _ = status_sender.send(());
            });
        }

        let mut res_src:Result<(String,String), _> = result_receiver.try_recv();
        let mut count = 0;

        // Hold main thread until success or completion
        while res_src.is_err() && count < self.chain_length {
            let _ = status_receiver.recv(); count+=1; // hold until next finish
            res_src = result_receiver.try_recv();
        }

        // If completed, 
        if res_src.is_err() {return Err("Unfortunately, the hash is not in the table.");}
        (*(indicator.clone().lock().unwrap())).finish_and_clear();

        let res = res_src.unwrap();
        let mut current_reduction: String = res.0;
        let mut hash:[u8;16] = md5::compute(current_reduction.as_bytes()).into();

        for i in 0..self.chain_length {
            println!("{:?}", hash);
            if hash.iter().map(|i| *i as char).collect::<String>() == hash_string {
                    return Ok(current_reduction)
            } else {
                current_reduction = utils::reduce(&hash, i);
                hash = md5::compute(current_reduction.as_bytes()).into();
            }

        }


        return Err("Unfortunately, the hash is not in the backchain.");
    }

    /// Use serde to write the rainbow table to a file. Returns a `Result`
    /// of either a unit or `io::Error`.
    ///
    /// # Arguments
    ///
    /// * `file`: file path to write to
    ///
    /// # Examples
    ///
    /// ```
    /// let table = Rainbow::create(10000, 1000, None);
    /// table.write_json("./hewo.json");
    /// ```
    pub fn write_json(&self, file:&str) -> Result<(),std::io::Error> {
        fs::write(file, serde_json::to_string(self).unwrap())
    }
}
