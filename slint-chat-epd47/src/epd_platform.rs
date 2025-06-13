use std::{rc::Rc, cell::RefCell, time::{Duration, Instant}};
use slint::platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};
use slint::{PhysicalPosition, PhysicalSize, PlatformError};
use log::{info, error};

use crate::MockEpdDisplay;

/// ESP-IDF Slint platform implementation for EPD47 e-paper display
pub struct EspIdfEpdPlatform {
    window: Rc<MinimalSoftwareWindow>,
    display: RefCell<MockEpdDisplay>,
    framebuffer: RefCell<Vec<u8>>,
    display_width: usize,
    display_height: usize,
    start_time: Instant,
}

impl EspIdfEpdPlatform {
    pub fn new(display: MockEpdDisplay) -> Self {
        let display_width = display.width() as usize;
        let display_height = display.height() as usize;
        
        info!("Creating ESP-IDF EPD platform with size {}x{}", display_width, display_height);
        
        // Create the Slint window
        let window = MinimalSoftwareWindow::new(Default::default());
        window.set_size(PhysicalSize::new(display_width as u32, display_height as u32));
        
        // Set scale factor for e-paper display
        window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
            scale_factor: 1.0,
        });
        window.dispatch_event(slint::platform::WindowEvent::Resized {
            size: window.size().to_logical(1.0),
        });

        // Initialize framebuffer (grayscale, 1 byte per pixel for simplicity)
        let framebuffer = vec![0xFF; display_width * display_height]; // Start with white

        Self {
            window,
            display: RefCell::new(display),
            framebuffer: RefCell::new(framebuffer),
            display_width,
            display_height,
            start_time: Instant::now(),
        }
    }

    /// Convert RGB565 to grayscale byte
    fn rgb565_to_grayscale(rgb565: u16) -> u8 {
        // Extract RGB components from RGB565
        let r = ((rgb565 >> 11) & 0x1F) as u8;
        let g = ((rgb565 >> 5) & 0x3F) as u8;
        let b = (rgb565 & 0x1F) as u8;
        
        // Convert to 8-bit values
        let r8 = (r * 255) / 31;
        let g8 = (g * 255) / 63;
        let b8 = (b * 255) / 31;
        
        // Calculate grayscale value using luminance formula
        (0.299 * r8 as f32 + 0.587 * g8 as f32 + 0.114 * b8 as f32) as u8
    }

    /// Update the e-paper display with rendered content
    fn update_display(&self, buffer: &[Rgb565Pixel]) -> Result<(), PlatformError> {
        let mut display = self.display.borrow_mut();
        let mut framebuffer = self.framebuffer.borrow_mut();
        
        // Convert RGB565 buffer to grayscale
        for (i, pixel) in buffer.iter().enumerate() {
            if i < framebuffer.len() {
                framebuffer[i] = Self::rgb565_to_grayscale(pixel.0);
            }
        }
        
        // Clear display
        display.clear().map_err(|e| PlatformError::Other(e.into()))?;
        
        // Update display (in chunks to avoid overwhelming the display)
        const CHUNK_HEIGHT: u32 = 50;
        let width = self.display_width as u32;
        let height = self.display_height as u32;
        
        for y in (0..height).step_by(CHUNK_HEIGHT as usize) {
            let chunk_height = std::cmp::min(CHUNK_HEIGHT, height - y);
            let start_idx = (y * width) as usize;
            let end_idx = std::cmp::min(start_idx + (chunk_height * width) as usize, framebuffer.len());
            
            if start_idx < framebuffer.len() && end_idx <= framebuffer.len() {
                let chunk_data = &framebuffer[start_idx..end_idx];
                display.update_region(0, y, width, chunk_height, chunk_data)
                    .map_err(|e| PlatformError::Other(e.into()))?;
            }
        }
        
        // Flush to display
        display.flush().map_err(|e| PlatformError::Other(e.into()))?;
        
        info!("EPD display updated successfully");
        Ok(())
    }
}

impl slint::platform::Platform for EspIdfEpdPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn run_event_loop(&self) -> Result<(), PlatformError> {
        info!("Starting ESP-IDF Slint event loop for EPD47...");

        // Create buffer for rendering
        let mut buffer = Vec::new();
        buffer.resize(self.display_width * self.display_height, Rgb565Pixel(0xFFFF)); // Start with white

        let mut frame_count = 0u32;
        let mut last_update = Instant::now();

        loop {
            // Update timers and animations
            slint::platform::update_timers_and_animations();

            // Render frame if needed
            let needs_update = self.window.draw_if_needed(|renderer| {
                frame_count += 1;
                info!("Rendering frame {} for EPD47...", frame_count);
                
                // Render to RGB565 buffer
                let regions = renderer.render(&mut buffer, self.display_width);
                
                info!("Rendered {} regions in frame {}", regions.len(), frame_count);
                
                // Only update display every few seconds to avoid excessive refreshing on e-paper
                let now = Instant::now();
                if now.duration_since(last_update) > Duration::from_secs(2) {
                    // Update the e-paper display
                    if let Err(e) = self.update_display(&buffer) {
                        error!("Failed to update EPD display: {:?}", e);
                    } else {
                        last_update = now;
                    }
                } else {
                    info!("Skipping display update (too frequent for e-paper)");
                }
            });

            if needs_update {
                info!("Frame {} rendered", frame_count);
            }

            // E-paper displays are slow, so we can use longer delays
            std::thread::sleep(Duration::from_millis(100));

            // Longer delay when idle to save power
            if !self.window.has_active_animations() {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }
}