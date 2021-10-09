mod utils;
mod table;

// use indicatif::ProgressBar;

fn main() {
    println!("{:?}", table::Rainbow::create(10000, 5000, None));

    // let bar = ProgressBar::new(10000);
    // println!("{:?}", utils::generate_chain("ha", 10000, Some(&bar)));
    // println!("{:?}", utils::generate_chain("hallo", 10000, None));
}

// 4294967295
