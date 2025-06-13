use std::{thread, time::Duration};

mod epd_platform;

use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::sys as esp_idf_sys;
use log::{info, error};

use crate::epd_platform::EspIdfEpdPlatform;

// Include Slint modules (generated from .slint files)
slint::include_modules!();

/// Simple Slint demo application for ESP-IDF
pub struct SlintDemo {
    welcome_ui: WelcomeScreen,
    chat_ui: ChatWindow,
}

impl SlintDemo {
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating Slint demo for ESP-IDF...");
        
        // Create Slint UI components
        let welcome_ui = WelcomeScreen::new().map_err(|e| anyhow::anyhow!(e))?;
        let chat_ui = ChatWindow::new().map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(Self {
            welcome_ui,
            chat_ui,
        })
    }

    pub fn run_welcome_demo(&self) -> anyhow::Result<()> {
        info!("Running welcome screen demo");
        
        // Set up welcome screen callback
        let chat_ui_weak = self.chat_ui.as_weak();
        self.welcome_ui.on_start_chat(move || {
            info!("Start chat button clicked!");
            // In a full implementation, you'd switch to chat screen here
        });
        
        // Show welcome screen for 5 seconds
        info!("Showing welcome screen...");
        
        // Run welcome UI briefly
        let welcome_weak = self.welcome_ui.as_weak();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(5));
            if let Some(welcome) = welcome_weak.upgrade() {
                welcome.hide().unwrap_or_else(|e| error!("Failed to hide welcome: {:?}", e));
            }
        });
        
        self.welcome_ui.run().map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(())
    }

    pub fn run_chat_demo(&self) -> anyhow::Result<()> {
        info!("Running chat UI demo");
        
        // Set up some demo messages
        let demo_messages = vec![
            crate::ChatMessage {
                content: "Hello! This is a Slint UI demo running on ESP-IDF.".into(),
                is_user: false,
                timestamp: "12:00".into(),
            },
            crate::ChatMessage {
                content: "How does it look on the e-paper display?".into(),
                is_user: true,
                timestamp: "12:01".into(),
            },
            crate::ChatMessage {
                content: "The e-paper display provides excellent readability and very low power consumption!".into(),
                is_user: false,
                timestamp: "12:01".into(),
            },
        ];
        
        self.chat_ui.set_messages(demo_messages.into());
        self.chat_ui.set_is_loading(false);
        
        // Set up callbacks for demo
        self.setup_demo_callbacks();
        
        // Run the chat UI
        info!("Starting chat UI event loop...");
        self.chat_ui.run().map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(())
    }

    fn setup_demo_callbacks(&self) {
        // Demo send message callback (just adds echo)
        let chat_ui_weak = self.chat_ui.as_weak();
        self.chat_ui.on_send_message(move |message| {
            info!("Demo: User sent message: {}", message);
            
            if let Some(chat_ui) = chat_ui_weak.upgrade() {
                // Get current messages
                let mut messages: Vec<crate::ChatMessage> = chat_ui.get_messages().iter().collect();
                
                // Add user message
                messages.push(crate::ChatMessage {
                    content: message.clone(),
                    is_user: true,
                    timestamp: "12:34".into(),
                });
                
                // Add demo response
                let response = format!("Demo response to: {}", message);
                messages.push(crate::ChatMessage {
                    content: response.into(),
                    is_user: false,
                    timestamp: "12:34".into(),
                });
                
                // Update UI
                chat_ui.set_messages(messages.into());
            }
        });
        
        // Demo clear callback
        let chat_ui_weak = self.chat_ui.as_weak();
        self.chat_ui.on_clear_chat(move || {
            info!("Demo: Clearing chat");
            if let Some(chat_ui) = chat_ui_weak.upgrade() {
                chat_ui.set_messages(Default::default());
            }
        });
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting Slint UI Demo on ESP-IDF for EPD47");

    // Initialize peripherals
    let peripherals = Peripherals::take()?;
    
    info!("Initializing EPD47 display...");
    
    // Create mock display for now (you can integrate real EPD47 later)
    let mock_display = MockEpdDisplay::new();
    
    // Set up Slint platform for EPD47
    let platform = EspIdfEpdPlatform::new(mock_display);
    slint::platform::set_platform(Box::new(platform))?;

    info!("Slint platform initialized for ESP-IDF");

    // Create and run demo
    let demo = SlintDemo::new()?;
    
    info!("Running welcome screen demo...");
    if let Err(e) = demo.run_welcome_demo() {
        error!("Welcome demo error: {:?}", e);
    }
    
    info!("Running chat UI demo...");  
    if let Err(e) = demo.run_chat_demo() {
        error!("Chat demo error: {:?}", e);
    }

    info!("Demo completed");
    Ok(())
}

/// Mock EPD display for testing Slint UI
struct MockEpdDisplay {
    width: u32,
    height: u32,
}

impl MockEpdDisplay {
    fn new() -> Self {
        Self {
            width: 960,
            height: 540,
        }
    }
    
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    
    fn clear(&mut self) -> Result<(), &'static str> {
        info!("Mock EPD: clear()");
        Ok(())
    }
    
    fn update_region(&mut self, _x: u32, _y: u32, _w: u32, _h: u32, _data: &[u8]) -> Result<(), &'static str> {
        info!("Mock EPD: update_region({}, {}, {}, {})", _x, _y, _w, _h);
        Ok(())
    }
    
    fn flush(&mut self) -> Result<(), &'static str> {
        info!("Mock EPD: flush()");
        Ok(())
    }
}