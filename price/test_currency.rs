use iso_currency::Currency;
use std::str::FromStr;

fn main() {
    println!("Testing Currency::from_str:");
    match Currency::from_str("USD") {
        Ok(c) => println!("USD via from_str: {:?}", c),
        Err(e) => println!("USD via from_str failed: {:?}", e),
    }
    match Currency::from_str("EUR") {
        Ok(c) => println!("EUR via from_str: {:?}", c),
        Err(e) => println!("EUR via from_str failed: {:?}", e),
    }
    
    println!("\nTesting Currency::from_code:");
    match Currency::from_code("USD") {
        Some(c) => println!("USD via from_code: {:?}", c),
        None => println!("USD via from_code failed"),
    }
    match Currency::from_code("EUR") {
        Some(c) => println!("EUR via from_code: {:?}", c),
        None => println!("EUR via from_code failed"),
    }
}
