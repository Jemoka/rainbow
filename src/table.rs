//! A Rainbow Table!

use md5;
use std::fs;
use std::convert::TryInto;
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
    pub rainbow_table: Vec<(Vec<u8>, Vec<u8>)>
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
                  seeds: Option<&[&str]>) -> Result<Self, &'static str> {

        // Set up seed values
        let seed_vec:Vec<String> = if let Some(seeds_arr) = seeds {
            seeds_arr.iter().map(|i| String::from(*i)).collect()
        } else {
            (0..samples).map(|_| rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(rand::thread_rng()
                                  .gen_range(0..25))
                            .map(char::from)
                            .collect())
                .collect()
        };

        // Assert length of vector
        assert_eq!(seed_vec.len(),
                   (samples as usize),
                   "seed vector must be the same length as the sample length of table");

        // Set up mutex for progress bar
        let indicator = Arc::new(Mutex::new(ProgressBar::new(samples as u64)));

        // Create thread pool and channels
        let pool = ThreadPool::new(threads as usize);
        let (sender, receiver) = channel();

        // Take each seed, generate the corresponding chain, store it
        for item in seed_vec {
            // Create thread-specific pointers to indicator and result sender
            let payload = Arc::clone(&indicator);
            let sender = sender.clone();

            // Go!
            pool.execute(move || {
                // Generate a chain of length `length` via reduction fns [0,length-1]
                let _ = sender.send((item.as_bytes().to_owned(), utils::generate_chain(item.as_bytes(), length, None)));

                // Increment the shared counter
                (*(payload.lock().unwrap())).inc(1);
            });
        }

        // Collect results together
        let rainbow_table:Vec<(Vec<u8>, Vec<u8>)> = receiver.iter()
                                                            .take(samples as usize)
                                                            .collect();

        // Finish progress bar
        (*(indicator.clone().lock().unwrap())).finish_and_clear();

        return Ok(Rainbow { number_samples: samples, chain_length: length, rainbow_table: rainbow_table});
    }

    /// Private method to discover the correct chain to decode for a hash
    fn discover_chain(&self, src_hash: &[u8], threads: u8) -> Result<(Vec<u8>,Vec<u8>), &'static str> {
        // Copy the length of theh table into the current scope
        let chain_length: u32 = self.chain_length;

        // Copy the source hash into the curren scope and coerce it into array
        let src_hash:[u8;16] = if let Ok(res) = src_hash.try_into() { res }
                                else { return Err("Unexpected input hash pointer format! Should be array of 16 u8s") };

        // Create a thread-safe mutex for the bar and a thread-safe smart pointer to the rainbow table
        let table = Arc::new(self.rainbow_table.clone());
        let indicator = Arc::new(Mutex::new(ProgressBar::new(self.chain_length as u64)));
        
        // Create the thread pool and necessary message channels
        let pool = ThreadPool::new(threads as usize);
        let (result_sender, result_receiver) = channel(); // used to send found chains
        let (status_sender, status_receiver) = channel(); // used to report finished status

        // For loop through the last n-th reduction, perform them, and check
        // This would be worth (1/2)O(n^2) --- {last}, {second last, last}, ...
        // It's in chain_length+1 because we want to capture one set of reduction
        // with range chain_length..chain_length --- no further reduction
        for start in (0..(chain_length+1)).rev() {
            // Make thread-specific copies of each safe smart pointer
            let table = table.clone();
            let payload = indicator.clone();
            let result_sender = result_sender.clone();
            let status_sender = status_sender.clone();

            // Go!
            pool.execute(move || {
                // Copy the source hash to a mutable var
                let mut hash:[u8;16] = src_hash;

                // Calculate the nth reduction starting from the start-th reduction
                for i in start..chain_length {
                    // Reduce the hash, then compute again
                    hash = md5::compute(utils::reduce(&hash, i)).into();
                }

                // Vecorize the computed array into an owned object
                let final_hash:Vec<u8> = hash.to_vec();

                // Check if final reduced hash is in the table
                for elem in &*table {
                    // If the hash vas found
                    if elem.1 == final_hash {
                        // Tell the sender about it, and ignore errors by consuming
                        // the value.
                        // TODO, there should not be any but still
                        let _ = result_sender.send((elem.0.clone(), elem.1.clone()));
                        break;
                    }
                }

                // Incriment the counter
                (*(payload.lock().unwrap())).inc(1);

                // Tell'em we are done, ignoring any thread msging errors
                // TODO as with above
                let _ = status_sender.send(());
            });
        }

        // Wait for an answer from any of the threads, or wait until all threads
        // declare the end of search
        let mut res_src = result_receiver.try_recv();
        let mut count = 0;

        // Use a while loop to block current thread until we either recieved the
        // end message from all (chain_length number of) threads or we recieved
        // a response message that isn't an error

        while res_src.is_err() && count < self.chain_length {
            let _ = status_receiver.recv(); count+=1; // hold main thread until next finish message to check
            res_src = result_receiver.try_recv(); // some thread finished, let's recieve potential result
        }

        // Clear the progress bar
        (*(indicator.clone().lock().unwrap())).finish_and_clear();

        // If we completed, and still didn't find anything, return an Error.
        // Otherwise, return the result.
        return if res_src.is_err() { Err("Cannot find hash inside table.") }
                else { Ok(res_src.unwrap()) };
    }

    // Private method to regenerate a chain and recover the password
    fn recover_password(&self, chain_src:&[u8], target_hash: &[u8]) -> Result<String, &'static str> {
        let mut hash:[u8;16];
        let mut src:Vec<u8> = chain_src.to_owned();

        for i in 0..self.chain_length {
            hash = md5::compute(&src).into();
            if hash == *target_hash {
                return Ok(src.iter().map(|i| *i as char).collect());
            }

            src = utils::reduce(&hash, i);
        }

        return Err("Cannot find hash inside backchain.");

    }

    // fn discover_chain(&self, src_hash: &[u8], threads: u8) -> Result<(Vec<u8>,Vec<u8>), &'static str> {

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
        // Attempt to discover the chain in which the hash is included
        // or propergate error forwards if needed
        let target_chain = self.discover_chain(src_hash, threads)?.0;
        self.recover_password(&target_chain, src_hash)
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

    /// Use serde to read a file into a rainbow table. Returns a `Result`
    /// of either a table or `io::Error`.
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
    pub fn read_json(file:&str) -> Result<Self,std::io::Error> {
        let deserialized: Self = serde_json::from_str(&fs::read_to_string(file)?).unwrap();
        Ok(deserialized)
    }
}
