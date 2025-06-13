// src/main.rs - Integrated Slint + EPD47 with esp-idf-svc
use esp_idf_svc::hal::prelude::Peripherals;
use log::info;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use embedded_graphics::{
    prelude::*,
    pixelcolor::Gray4,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    geometry::Point,
};

// Import Slint
slint::include_modules!();
use slint::Model; // Add this import for row_count method

// Import our EPD47 driver
mod epd47_idf;
use epd47_idf::{Epd47Display, DrawMode, init_epd47_pins};

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting EPD47 Slint Chat Demo");

    // Initialize peripherals
    let mut peripherals = Peripherals::take()?;

    // Initialize EPD47 display
    let pins = init_epd47_pins(&mut peripherals)?;
    let mut epd_display = Epd47Display::new(pins)?;
    
    // Power on the display
    epd_display.power_on()?;
    epd_display.clear()?;
    
    info!("EPD47 display initialized successfully");

    // Test the display with a simple circle
    test_epd47_display(&mut epd_display)?;

    // Create the Slint UI using default platform
    let ui = ChatWindow::new().map_err(|e| anyhow::anyhow!("Failed to create UI: {:?}", e))?;
    
    // Demo conversation data
    let demo_messages = vec![
        ("Welcome to ChatGPT on E-Paper!", false),
        ("Hello! How does this e-paper display work?", true),
        ("This EPD47 display uses electrophoretic technology with 960x540 resolution and 4-bit grayscale. It only consumes power during updates!", false),
        ("That's amazing! Can you tell me more about Slint?", true),
        ("Slint is a modern GUI toolkit that works great on embedded devices. It uses a software renderer that we can bridge to the e-paper display.", false),
        ("How is the performance?", true),
        ("E-paper displays refresh slowly (~2 seconds) but hold images without power. Perfect for low-power applications!", false),
    ];

    let slint_messages: Vec<(slint::SharedString, bool)> = demo_messages
        .into_iter()
        .map(|(content, is_user)| (content.into(), is_user))
        .collect();
    
    let message_model = Rc::new(slint::VecModel::from(slint_messages));
    ui.set_messages(message_model.clone().into());

    // Handle new messages
    let ui_weak = ui.as_weak();
    let message_model_clone = message_model.clone();
    ui.on_send_message(move |message| {
        let ui = ui_weak.upgrade().unwrap();
        info!("User sent: {}", message);
        
        // Add user message
        message_model_clone.push((message.clone(), true));
        
        // Generate AI response based on input
        let response = generate_ai_response(&message);
        message_model_clone.push((response.into(), false));
        
        // Clear input - Note: input_text property may not exist in your UI
        // ui.set_input_text("".into()); // Remove this line if input_text doesn't exist
    });

    info!("Starting Slint UI...");

    // Start a background thread to periodically update the EPD47 display
    let ui_weak_bg = ui.as_weak();
    thread::spawn(move || {
        let mut last_message_count = 0;
        
        loop {
            thread::sleep(Duration::from_secs(3)); // Update every 3 seconds
            
            if let Some(ui) = ui_weak_bg.upgrade() {
                let current_count = ui.get_messages().row_count();
                
                if current_count != last_message_count {
                    info!("Messages changed, updating EPD47 display...");
                    // Here you would capture Slint's rendered output and send to EPD47
                    // For now, just log the update
                    last_message_count = current_count;
                }
            } else {
                break; // UI has been dropped
            }
        }
    });

    // Run the Slint UI (this will block)
    ui.run().map_err(|e| anyhow::anyhow!("UI run failed: {:?}", e))?;

    // Cleanup when UI exits
    epd_display.power_off()?;
    info!("Application finished");

    Ok(())
}

fn test_epd47_display(display: &mut Epd47Display) -> anyhow::Result<()> {
    info!("Testing EPD47 display with graphics...");
    
    // Draw a test pattern
    Circle::new(Point::new(100, 100), 50)
        .into_styled(PrimitiveStyle::with_stroke(Gray4::BLACK, 3))
        .draw(display)?;
    
    Rectangle::new(Point::new(200, 200), embedded_graphics::geometry::Size::new(100, 50))
        .into_styled(PrimitiveStyle::with_fill(Gray4::new(8))) // Mid-gray
        .draw(display)?;
    
    // Flush to display
    display.flush(DrawMode::BlackOnWhite)?;
    
    info!("EPD47 test pattern displayed");
    Ok(())
}

fn generate_ai_response(input: &str) -> String {
    let input_lower = input.to_lowercase();
    
    if input_lower.contains("hello") || input_lower.contains("hi") {
        "Hello! I'm running on an EPD47 e-paper display. The 960x540 resolution gives us plenty of space for conversations!".to_string()
    } else if input_lower.contains("epd") || input_lower.contains("e-paper") || input_lower.contains("display") {
        "This EPD47 display is fascinating! It uses 4-bit grayscale (16 levels) and only needs power during updates. Perfect for battery-powered devices.".to_string()
    } else if input_lower.contains("slint") {
        "Slint is an excellent choice for embedded GUIs! Its software renderer works perfectly with e-paper displays, and the declarative UI language makes development smooth.".to_string()
    } else if input_lower.contains("power") || input_lower.contains("battery") {
        "One of the best features of e-paper is the ultra-low power consumption. Once an image is displayed, it stays visible with zero power until the next update!".to_string()
    } else if input_lower.contains("performance") || input_lower.contains("speed") {
        "E-paper displays prioritize power efficiency over speed. Updates take 1-3 seconds, but the crisp, paper-like appearance and zero power retention make it worth it!".to_string()
    } else {
        format!("Thanks for your message about '{}'! This chat is running on an ESP32-S3 with an EPD47 e-paper display using Rust and Slint UI.", input)
    }
}