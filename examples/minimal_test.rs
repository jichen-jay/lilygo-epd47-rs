#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;

#[entry]
fn main() -> ! {
    println!("=== ESP32-S3 Ultra Minimal Test ===");
    
    let _peripherals = esp_hal::init(esp_hal::Config::default());
    println!("ESP-HAL initialized successfully!");
    
    let delay = Delay::new();
    println!("Delay created successfully!");
    
    let mut counter = 0;
    loop {
        println!("Ultra minimal counter: {}", counter);
        counter += 1;
        delay.delay_millis(1000);
        
        if counter >= 5 {
            println!("Test completed! Everything is working.");
            // Continue counting but slower
            delay.delay_millis(2000);
        }
    }
}