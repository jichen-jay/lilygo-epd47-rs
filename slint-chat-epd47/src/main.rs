use esp_idf_svc::hal::prelude::Peripherals;
use log::info;
use std::rc::Rc;
use slint::Model;

// Import Slint
slint::include_modules!();

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting EPD47 Chat Demo");

    // Initialize peripherals (for future display integration)
    let _peripherals = Peripherals::take()?;

    // Create the Slint UI
    let ui = ChatWindow::new().map_err(|e| anyhow::anyhow!("Failed to create UI: {:?}", e))?;
    
    // Demo conversation data - simulating a real chat
    let demo_messages = vec![
        ("Welcome to ChatGPT on E-Paper!", false),
        ("Hello! I'd like to learn about renewable energy.", true),
        ("That's a great topic! Renewable energy includes solar, wind, hydro, and geothermal power. These sources are sustainable because they naturally replenish and don't run out like fossil fuels.", false),
        ("What's the most efficient type?", true),
        ("Solar panels typically achieve 15-22% efficiency, while modern wind turbines can reach 35-45%. However, 'efficiency' depends on location - coastal areas favor wind, sunny regions favor solar.", false),
        ("Can you explain how solar panels work?", true),
        ("Solar panels use photovoltaic cells made of silicon. When sunlight hits these cells, it knocks electrons loose, creating an electric current. This direct current (DC) is then converted to alternating current (AC) for home use.", false),
        ("That's fascinating! What about costs?", true),
        ("Solar costs have dropped dramatically - about 90% since 2010! Installation typically costs $15,000-25,000 for homes, but tax credits and savings over time make it increasingly affordable.", false),
        ("Thanks for the detailed explanation!", true),
        ("You're welcome! Feel free to ask about any other topics. I'm here to help with questions about science, technology, writing, or anything else you're curious about.", false),
    ];

    // Convert to Slint format
    let slint_messages: Vec<(slint::SharedString, bool)> = demo_messages
        .into_iter()
        .map(|(content, is_user)| (content.into(), is_user))
        .collect();
    
    let message_model = Rc::new(slint::VecModel::from(slint_messages));
    ui.set_messages(message_model.clone().into());

    // Simulate typing indicator and new messages
    let ui_weak = ui.as_weak();
    let message_model_clone = message_model.clone();
    ui.on_send_message(move |message| {
        let ui = ui_weak.upgrade().unwrap();
        info!("Demo: User sent message: {}", message);
        
        // Add user message to the conversation
        message_model_clone.push((message.clone(), true));
        
        // Show typing indicator
        ui.set_is_loading(true);
        
        // Generate response immediately (simulate AI processing)
        let response = generate_demo_response(&message.to_string());
        
        // Use a timer to simulate thinking delay
        let message_model_timer = message_model_clone.clone();
        let ui_weak_inner = ui.as_weak();
        let response_msg = response.clone();
        slint::Timer::single_shot(std::time::Duration::from_millis(1500), move || {
            if let Some(ui) = ui_weak_inner.upgrade() {
                message_model_timer.push((response_msg.into(), false));
                ui.set_is_loading(false);
            }
        });
        
        // Clear input
        ui.set_current_message("".into());
    });

    // Demo clear chat functionality
    let ui_weak = ui.as_weak();
    let message_model_clone = message_model.clone();
    ui.on_clear_chat(move || {
        info!("Demo: Clearing chat");
        
        // Reset to welcome message
        message_model_clone.set_vec(vec![
            ("Chat cleared! Welcome back to ChatGPT on E-Paper.".into(), false),
            ("This demo shows how a chat interface would work on an e-paper display.".into(), false),
            ("Try typing different topics like 'weather', 'cooking', 'space', or 'technology'!".into(), false),
        ]);
        
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_is_loading(false);
            ui.set_current_message("".into());
        }
    });

    info!("Starting chat demo UI");
    
    // Run the UI event loop
    ui.run().map_err(|e| anyhow::anyhow!("UI error: {:?}", e))?;

    Ok(())
}

// Generate demo responses based on simple keyword matching
fn generate_demo_response(message: &str) -> String {
    let message_lower = message.to_lowercase();
    
    if message_lower.contains("weather") {
        "I can't check real weather data in this demo, but a real chat app could integrate with weather APIs to provide current conditions and forecasts for any location you specify.".to_string()
    } else if message_lower.contains("cooking") || message_lower.contains("recipe") {
        "Here's a simple recipe demo: For pasta, boil water, add salt, cook pasta 8-12 minutes, drain, and add your favorite sauce. A real app could suggest recipes based on ingredients you have!".to_string()
    } else if message_lower.contains("space") || message_lower.contains("astronomy") {
        "Space is fascinating! Did you know the James Webb Space Telescope can see galaxies from when the universe was just 400 million years old? That's like looking 13+ billion years into the past!".to_string()
    } else if message_lower.contains("technology") || message_lower.contains("ai") {
        "This e-paper display is perfect for reading and text-based AI interactions! E-paper uses very little power and is easy on the eyes, making it ideal for extended conversations.".to_string()
    } else if message_lower.contains("hello") || message_lower.contains("hi") {
        "Hello! Welcome to this e-paper chat demo. This simulates how ChatGPT would work on an ESP32 with an e-paper display. What would you like to explore?".to_string()
    } else if message_lower.contains("help") {
        "This is a demo of a chat interface optimized for e-paper displays. Try asking about weather, cooking, space, or technology to see different response examples!".to_string()
    } else if message_lower.len() < 3 {
        "I see you sent a short message. In a real chat app, I'd be able to understand and respond to any length of message, from quick questions to detailed discussions.".to_string()
    } else {
        let preview = if message.len() > 50 { 
            format!("{}...", &message[..47])
        } else { 
            message.to_string()
        };
        format!("Thanks for your message about '{}'! In a real implementation, I'd use advanced AI to provide helpful, detailed responses about any topic you're interested in discussing.", preview)
    }
}