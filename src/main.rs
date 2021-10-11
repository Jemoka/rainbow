mod table;
mod utils;

pub use table::Rainbow;
use clap;
use clap::{Arg, App, SubCommand};

fn main() {
    let menu = clap_app!(



    // println!("{:?}", base64::encode([1,2,3,4,5,6,7]).as_bytes());

    // "abc" = "900150983cd24fb0d6963f7d28e17f72"

    // let res:[u8;16] = md5::compute("hewoo").into();
    // println!("{}", utils::reduce(&res, 23).iter().map(|i| *i as char).collect::<String>());

    let re:Vec<&str>;
    if let Ok(file) = fs::read_to_string("./10-million-password-list-top-1000000.txt") {
        re = file.lines().collect();
        let res = table::Rainbow::create(999998, 10000, 5, Some(&re));
        if let Ok(table) = res {
            if let Ok(_) = table.write_json("./pwl.json") {} else {};
            // let mut v = vec![];
            // let mut counter = 0;
            // for row in &table.rainbow_table {
            //     if v.contains(&row.1) {
            //         counter +=1;
            //     }
            //     v.push(row.1.clone());
            // }
            // println!("Duplicated rows: {}", counter);

            // let r:[u8;16] = md5::compute("hne3to").into();
            // println!("\n \n result: {:?}", table.decode(&r, 200));
        }

    }

    // println!("table: {:?}", res);


    // let bar = ProgressBar::new(10000);
    // println!("{:?}", utils::generate_chain("ha", 10000, Some(&bar)));
    // println!("{:?}", utils::generate_chain("hallo", 10000, None));
}

// 4294967295
