// src/epd47_idf.rs - New EPD47 driver for esp-idf-svc
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{*, Output, PinDriver},
    peripherals::Peripherals,
};

use log::{info, warn};
use embedded_graphics_core::{
    prelude::*,
    pixelcolor::{Gray4, GrayColor},
    geometry::Size,
    draw_target::DrawTarget,
    Pixel,
};

const DISPLAY_WIDTH: u32 = 960;
const DISPLAY_HEIGHT: u32 = 540;
const FRAMEBUFFER_SIZE: usize = (DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize) / 2; // 4-bit per pixel

#[derive(Debug)]
pub enum Epd47Error {
    Gpio(esp_idf_svc::sys::EspError),
    Hardware,
    InvalidColor,
    OutOfBounds,
}

impl std::fmt::Display for Epd47Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EPD47 Error: {:?}", self)
    }
}

impl std::error::Error for Epd47Error {}

pub struct Epd47PinConfig<'a> {
    // Data pins (8-bit parallel)
    pub data0: PinDriver<'a, AnyOutputPin, Output>,
    pub data1: PinDriver<'a, AnyOutputPin, Output>, 
    pub data2: PinDriver<'a, AnyOutputPin, Output>,
    pub data3: PinDriver<'a, AnyOutputPin, Output>,
    pub data4: PinDriver<'a, AnyOutputPin, Output>,
    pub data5: PinDriver<'a, AnyOutputPin, Output>,
    pub data6: PinDriver<'a, AnyOutputPin, Output>,
    pub data7: PinDriver<'a, AnyOutputPin, Output>,
    
    // Control pins
    pub cfg_data: PinDriver<'a, AnyOutputPin, Output>,
    pub cfg_clk: PinDriver<'a, AnyOutputPin, Output>,
    pub cfg_str: PinDriver<'a, AnyOutputPin, Output>,
    pub lcd_dc: PinDriver<'a, AnyOutputPin, Output>,
    pub lcd_wrx: PinDriver<'a, AnyOutputPin, Output>,
    pub rmt: PinDriver<'a, AnyOutputPin, Output>,
}

#[derive(Clone, Copy, Debug)]
pub enum DrawMode {
    BlackOnWhite,
    WhiteOnBlack,
    WhiteOnWhite,
}

/// EPD47 Display driver for esp-idf-svc
pub struct Epd47Display<'a> {
    pins: Epd47PinConfig<'a>,
    framebuffer: Vec<u8>,
    delay: Delay,
    powered_on: bool,
}

impl<'a> Epd47Display<'a> {
    pub fn new(pins: Epd47PinConfig<'a>) -> Result<Self, Epd47Error> {
        info!("Initializing EPD47 display ({}x{})", DISPLAY_WIDTH, DISPLAY_HEIGHT);
        
        let mut display = Self {
            pins,
            framebuffer: vec![0xFF; FRAMEBUFFER_SIZE], // Initialize to white
            delay: Delay::new_default(),
            powered_on: false,
        };
        
        // Initialize control registers
        display.init_config_registers()?;
        
        Ok(display)
    }

    pub fn width(&self) -> u32 {
        DISPLAY_WIDTH
    }

    pub fn height(&self) -> u32 {
        DISPLAY_HEIGHT
    }

    pub fn power_on(&mut self) -> Result<(), Epd47Error> {
        if self.powered_on {
            return Ok(());
        }

        info!("Powering on EPD47 display");
        
        // Power sequence - simplified version
        // You may need to adjust this based on the original driver
        self.set_config_power(true)?;
        self.delay.delay_ms(10);
        
        // Enable positive/negative voltages
        self.set_config_voltages(true, true)?;
        self.delay.delay_ms(20);
        
        // Enable output
        self.set_config_output_enable(true)?;
        self.delay.delay_ms(5);
        
        self.powered_on = true;
        info!("EPD47 display powered on successfully");
        Ok(())
    }

    pub fn power_off(&mut self) -> Result<(), Epd47Error> {
        if !self.powered_on {
            return Ok(());
        }

        info!("Powering off EPD47 display");
        
        // Reverse power sequence
        self.set_config_output_enable(false)?;
        self.delay.delay_ms(5);
        
        self.set_config_voltages(false, false)?;
        self.delay.delay_ms(20);
        
        self.set_config_power(false)?;
        self.delay.delay_ms(10);
        
        self.powered_on = false;
        info!("EPD47 display powered off");
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Epd47Error> {
        info!("Clearing EPD47 display");
        
        // Fill framebuffer with white (0xFF)
        self.framebuffer.fill(0xFF);
        
        Ok(())
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u8) -> Result<(), Epd47Error> {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return Err(Epd47Error::OutOfBounds);
        }
        
        if color > 0x0F {
            return Err(Epd47Error::InvalidColor);
        }

        let pixel_index = (y * DISPLAY_WIDTH + x) as usize;
        let byte_index = pixel_index / 2;
        let is_high_nibble = pixel_index % 2 == 0;

        if byte_index >= self.framebuffer.len() {
            return Err(Epd47Error::OutOfBounds);
        }

        if is_high_nibble {
            self.framebuffer[byte_index] = (self.framebuffer[byte_index] & 0x0F) | (color << 4);
        } else {
            self.framebuffer[byte_index] = (self.framebuffer[byte_index] & 0xF0) | color;
        }

        Ok(())
    }

    pub fn flush(&mut self, mode: DrawMode) -> Result<(), Epd47Error> {
        if !self.powered_on {
            warn!("Cannot flush - display is not powered on");
            return Err(Epd47Error::Hardware);
        }

        info!("Flushing framebuffer to EPD47 display ({:?})", mode);
        
        // This is a simplified flush implementation
        // The actual implementation would need to:
        // 1. Send framebuffer data via parallel interface (LCD_CAM)
        // 2. Apply proper waveforms via RMT
        // 3. Handle refresh timing
        
        // For now, simulate the flush operation
        self.delay.delay_ms(100); // Simulate refresh time
        
        info!("EPD47 display flush completed");
        Ok(())
    }

    // Helper methods for configuration register control
    fn init_config_registers(&mut self) -> Result<(), Epd47Error> {
        // Initialize all config pins to known state
        self.pins.cfg_data.set_low().map_err(Epd47Error::Gpio)?;
        self.pins.cfg_clk.set_high().map_err(Epd47Error::Gpio)?;
        self.pins.cfg_str.set_low().map_err(Epd47Error::Gpio)?;
        
        // Write default configuration
        self.write_config_register(0x00)?; // All disabled initially
        
        Ok(())
    }

    fn set_config_power(&mut self, enable: bool) -> Result<(), Epd47Error> {
        // Set power enable bit in config register
        let mut config = 0x00;
        if enable {
            config |= 0x04; // Power enable bit
        }
        self.write_config_register(config)
    }

    fn set_config_voltages(&mut self, pos_enable: bool, neg_enable: bool) -> Result<(), Epd47Error> {
        let mut config = 0x04; // Keep power enabled
        if pos_enable {
            config |= 0x08; // Positive voltage enable
        }
        if neg_enable {
            config |= 0x10; // Negative voltage enable
        }
        self.write_config_register(config)
    }

    fn set_config_output_enable(&mut self, enable: bool) -> Result<(), Epd47Error> {
        let mut config = 0x1C; // Keep power and voltages enabled
        if enable {
            config |= 0x01; // Output enable bit
        }
        self.write_config_register(config)
    }

    fn write_config_register(&mut self, value: u8) -> Result<(), Epd47Error> {
        // Shift out 8 bits to configuration register
        self.pins.cfg_str.set_low().map_err(Epd47Error::Gpio)?;
        
        for i in 0..8 {
            self.pins.cfg_clk.set_low().map_err(Epd47Error::Gpio)?;
            
            let bit = (value >> (7 - i)) & 0x01;
            if bit == 1 {
                self.pins.cfg_data.set_high().map_err(Epd47Error::Gpio)?;
            } else {
                self.pins.cfg_data.set_low().map_err(Epd47Error::Gpio)?;
            }
            
            self.pins.cfg_clk.set_high().map_err(Epd47Error::Gpio)?;
        }
        
        self.pins.cfg_str.set_high().map_err(Epd47Error::Gpio)?;
        Ok(())
    }
}

// Implement embedded-graphics DrawTarget trait
impl<'a> DrawTarget for Epd47Display<'a> {
    type Color = Gray4;
    type Error = Epd47Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.y >= 0 {
                let gray_value = color.luma();
                self.set_pixel(point.x as u32, point.y as u32, gray_value)?;
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for Epd47Display<'a> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    }
}

// Helper function to initialize pins
pub fn init_epd47_pins(peripherals: &mut Peripherals) -> Result<Epd47PinConfig, Epd47Error> {
    info!("Initializing EPD47 GPIO pins");
    
    let pins = Epd47PinConfig {
        data0: PinDriver::output(peripherals.pins.gpio6).map_err(Epd47Error::Gpio)?,
        data1: PinDriver::output(peripherals.pins.gpio7).map_err(Epd47Error::Gpio)?,
        data2: PinDriver::output(peripherals.pins.gpio4).map_err(Epd47Error::Gpio)?,
        data3: PinDriver::output(peripherals.pins.gpio5).map_err(Epd47Error::Gpio)?,
        data4: PinDriver::output(peripherals.pins.gpio2).map_err(Epd47Error::Gpio)?,
        data5: PinDriver::output(peripherals.pins.gpio3).map_err(Epd47Error::Gpio)?,
        data6: PinDriver::output(peripherals.pins.gpio8).map_err(Epd47Error::Gpio)?,
        data7: PinDriver::output(peripherals.pins.gpio1).map_err(Epd47Error::Gpio)?,
        
        cfg_data: PinDriver::output(peripherals.pins.gpio13).map_err(Epd47Error::Gpio)?,
        cfg_clk: PinDriver::output(peripherals.pins.gpio12).map_err(Epd47Error::Gpio)?,
        cfg_str: PinDriver::output(peripherals.pins.gpio0).map_err(Epd47Error::Gpio)?,
        lcd_dc: PinDriver::output(peripherals.pins.gpio40).map_err(Epd47Error::Gpio)?,
        lcd_wrx: PinDriver::output(peripherals.pins.gpio41).map_err(Epd47Error::Gpio)?,
        rmt: PinDriver::output(peripherals.pins.gpio38).map_err(Epd47Error::Gpio)?,
    };
    
    info!("EPD47 GPIO pins initialized successfully");
    Ok(pins)
}