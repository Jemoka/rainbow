mod table;
mod utils;


// use indicatif::ProgressBar;

fn main() {

    // "abc" = "900150983cd24fb0d6963f7d28e17f72"

    let res = table::Rainbow::create(100, 5000, None);
    if let Ok(table) = res {
        println!("result: {:?}", table.decode("900150983cd24fb0d6963f7d28e17f72"));
    }
    // println!("{:?}", table::Rainbow::new(10000, 5000, None));


    // let bar = ProgressBar::new(10000);
    // println!("{:?}", utils::generate_chain("ha", 10000, Some(&bar)));
    // println!("{:?}", utils::generate_chain("hallo", 10000, None));
}

// 4294967295
