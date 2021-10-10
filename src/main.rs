mod table;
mod utils;

pub use table::Rainbow;

// use indicatif::ProgressBar;

fn main() {
    // println!("{:?}", base64::encode([1,2,3,4,5,6,7]).as_bytes());

    // "abc" = "900150983cd24fb0d6963f7d28e17f72"

    // let a = "hewoo";
    // let digest:[u8;16] = md5::compute(a).into();
    // println!("{:x}, {:?}, {}", md5::compute(a), digest, utils::reduce(&digest, 1));

    let res = table::Rainbow::create(10000, 2000, 250, None);
    // println!("table: {:?}", res);
    if let Ok(table) = res {
        let r:[u8;16] = md5::compute("aoa").into();
        println!("\n \n result: {:?}", table.decode(&r, 200));
    }


    // let bar = ProgressBar::new(10000);
    // println!("{:?}", utils::generate_chain("ha", 10000, Some(&bar)));
    // println!("{:?}", utils::generate_chain("hallo", 10000, None));
}

// 4294967295
