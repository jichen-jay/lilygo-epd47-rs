use alloc::{string::String, vec::Vec, format};

/// Represents a single chat message
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub content: String,
    pub is_user: bool,
    pub timestamp: String,
}

impl ChatMessage {
    pub fn new(content: String, is_user: bool) -> Self {
        Self {
            content,
            is_user,
            timestamp: Self::get_current_time(),
        }
    }

    /// Simple timestamp generation (in a real app, you'd use proper time)
    fn get_current_time() -> String {
        // For demo purposes, return a fixed time format
        // In a real implementation, you'd get actual time from RTC or network
        "12:34".to_string()
    }
}

/// Handles chat message storage and processing
pub struct MessageHandler {
    messages: Vec<ChatMessage>,
    max_messages: usize,
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 50, // Limit to prevent memory issues
        }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        
        // Keep only the most recent messages to prevent memory overflow
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn add_user_message(&mut self, content: String) -> anyhow::Result<()> {
        let message = ChatMessage::new(content.clone(), true);
        self.add_message(message);
        
        // Generate AI response
        let response = self.generate_ai_response(&content);
        let ai_message = ChatMessage::new(response, false);
        self.add_message(ai_message);
        
        Ok(())
    }

    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.clone()
    }

    pub fn get_message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Generate a simple AI response based on user input
    /// In a real implementation, this would call the ChatGPT API
    pub fn generate_ai_response(&self, user_input: &str) -> String {
        let input_lower = user_input.to_lowercase();
        
        // Simple pattern matching for demo responses
        if input_lower.contains("hello") || input_lower.contains("hi") {
            "Hello! I'm your AI assistant running on an e-paper display. How can I help you today?".to_string()
        } else if input_lower.contains("weather") {
            "I don't have access to current weather data since I'm running offline on this e-paper display. But I hope it's beautiful where you are!".to_string()
        } else if input_lower.contains("time") {
            "I don't have access to real-time clock data in this demo, but I hope you're having a wonderful day!".to_string()
        } else if input_lower.contains("joke") {
            "Here's a tech joke for you: Why do programmers prefer dark mode? Because light attracts bugs! 😄".to_string()
        } else if input_lower.contains("how are you") {
            "I'm doing great! It's quite unique being a ChatGPT clone running on an e-paper display. The refresh rate is slow, but the battery life is amazing!".to_string()
        } else if input_lower.contains("help") {
            "I'm a simple ChatGPT clone designed for e-paper displays. I can chat with you, tell jokes, and discuss various topics. What would you like to talk about?".to_string()
        } else if input_lower.contains("display") || input_lower.contains("screen") {
            "This e-paper display is quite special! It only uses power when updating the screen, which makes it perfect for reading and long-term display applications.".to_string()
        } else if input_lower.contains("battery") {
            "E-paper displays are incredibly power-efficient! They only consume power when refreshing the image, so the battery can last for weeks or even months.".to_string()
        } else if input_lower.contains("technology") || input_lower.contains("tech") {
            "I love discussing technology! E-paper displays use electrophoretic technology to move charged particles, creating a paper-like reading experience without backlighting.".to_string()
        } else if input_lower.contains("book") || input_lower.contains("read") {
            "E-paper displays are perfect for reading! They provide a paper-like experience that's easy on the eyes and works great even in direct sunlight.".to_string()
        } else if input_lower.contains("color") || input_lower.contains("colours") {
            "This display uses grayscale instead of colors to optimize for e-paper technology. The monochrome appearance gives it a classic, timeless feel!".to_string()
        } else if input_lower.contains("