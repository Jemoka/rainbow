mod utils;
mod table;

pub use table::Rainbow;

use std::num::ParseIntError;
use std::fs;

#[macro_use]
extern crate clap;

/*
 * Hello, gentle Reader!
 * This file contains the driver code to the CLI of `rainbow`, and is
 * likely not helpful to the usage of this package as a library. Probing
 * `table.rs` is likely more helpful to that end.
 *
 */

// Decode a hexdecimal string
// https://stackoverflow.com/questions/52987181/how-can-i-convert-a-hex-string-to-a-u8-slice
fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

fn main() {
    let menu = clap_app!(jrainbow =>
                         (version: "0.0.1")
                         (author: "Houjun Liu <houliu@nuevaschool.org>")
                         (about: "Badly implemented MD5 Rainbow table")
                         (@subcommand attack =>
                          (about: "Use a rainbow table to crack a password")
                          (@arg TABLE: +required "Input JSON file path for a created rainbow table")
                          (@arg DIGEST: +required "Lower-hex digest of a md5 hash to crack")
                          (@arg THREADS: -t --threads +takes_value "Number of thread workers")
                         )
                         (@subcommand create =>
                          (about: "Create a rainbow table")
                          (@arg OUTPUT: +required "Output JSON file path for the created rainbow table")
                          (@arg PASSWORDFILE: -p --pfile +takes_value "Newline-seperated password file path")
                          (@arg THREADS: -t --threads +takes_value "Number of thread workers")
                          (@arg LENGTH: -l --length +takes_value "The length of rainbow chains")
                          (@arg SAMPLES: -s --samples +takes_value "The number of samples to seed")
                         )
    ).get_matches();

    if let Some(menu) = menu.subcommand_matches("attack") {
        let table:&str = menu.value_of("TABLE").unwrap();
        let digest:Vec<u8> = decode_hex(menu.value_of("DIGEST").unwrap()).unwrap();
        let threads:u8 = menu.value_of("threads").unwrap_or("10").parse::<u8>().unwrap();

        let table:Rainbow = Rainbow::read_json(table).unwrap();

        if let Ok(answer) = table.decode(&digest, threads) {
            println!("{}", answer);
            std::process::exit(0);
        } else if let Err(err) = table.decode(&digest, threads) {
            println!("Error! {}", err);
            std::process::exit(1);
        }
    } else if let Some(menu) = menu.subcommand_matches("create") {
        let output:&str = menu.value_of("OUTPUT").unwrap();
        let threads:u8 = menu.value_of("THREADS").unwrap_or("10").parse::<u8>().unwrap();
        let samples:u32 = menu.value_of("SAMPLES").unwrap_or("50000").parse::<u32>().unwrap();
        let length:u32 = menu.value_of("LENGTH").unwrap_or("4000").parse::<u32>().unwrap();
        if let Some(passwordfile) = menu.value_of("PASSWORDFILE") {
            let re:Vec<&str>;
            if let Ok(file) = fs::read_to_string(passwordfile) {
                re = file.lines().collect();
                let table = table::Rainbow::create(re.len() as u32, length, threads, Some(&re)).unwrap();
                if let Ok(_) = table.write_json(output) {} else {};
            }
        } else {
            let table = table::Rainbow::create(samples, length, threads, None).unwrap();
            if let Ok(_) = table.write_json(output) {} else {};
        }
    } else {
        println!("USAGE:\n    jrainbow [SUBCOMMAND]\n\nFor more information try --help");
    }
}
