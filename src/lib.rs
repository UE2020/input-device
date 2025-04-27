//! # input-device
//!
//! **`input-device`** is a cross-platform, low-level input simulation crate.
//!
//! It implements input emulation for various input devices
//! such as the mouse, keyboard, touchscreens, and pen devices
//! to simulate user input reliably across operating systems.
//!
//! ## Main Components
//!
//! - [`InputSimulator`]: The core struct for simulating input events.
//! - [`Key`]: An enum representing physical keyboard keys.
//! - [`KeyIter`]: An iterator over all possible [`Key`] variants.
//! - [`SimulationError`]: Errors returned when simulation operations fail.
//! These error variants differ across platforms.
//!
//! ## Current Features
//!
//! - Move the mouse to an absolute position or relative to the current position.
//! - Simulate left, middle, and right mouse button presses and releases.
//! - Scroll horizontally and vertically using the mouse wheel.
//! - Press and release keyboard keys individually.
//! - Query the full virtual screen size for multi-monitor setups.
//!
//! ## Example
//!
//! ```rust
//! use input_device::{InputDeviceSimulator, Key};
//!
//! fn main() -> Result<(), input_device::SimulationError> {
//!     let mut simulator = InputDeviceSimulator::new()?;
//!     std::thread::sleep(std::Duration::from_secs(1));
//!
//!     simulator.move_mouse_abs(500, 500)?;
//!     simulator.left_mouse_down()?;
//!     std::thread::sleep(std::Duration::from_secs(1));
//!     simulator.left_mouse_up()?;
//!     simulator.key_down(Key::A)?;
//!     std::thread::sleep(std::Duration::from_secs(1));
//!     simulator.key_up(Key::A)?;
//!
//!     Ok(())
//! }
//! ```

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

mod keys;
pub use keys::*;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
pub use windows::*;

/// An input simulator.
///
/// This struct contains all the resources needed to
/// simulate input on the target platform.
///
/// Semantics may differ between platforms. Known
/// differences are documented.
pub struct InputSimulator(PlatformImpl);

impl InputSimulator {
    /// Create a new input simulator.
    pub fn new() -> Result<Self, SimulationError> {
        Ok(Self(PlatformImpl::new()?))
    }

    pub fn move_mouse_abs(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        self.0.move_mouse_abs(x, y)
    }

    pub fn move_mouse_rel(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        self.0.move_mouse_rel(x, y)
    }

    pub fn left_mouse_down(&mut self) -> Result<(), SimulationError> {
        self.0.left_mouse_down()
    }

    pub fn middle_mouse_down(&mut self) -> Result<(), SimulationError> {
        self.0.middle_mouse_down()
    }

    pub fn right_mouse_down(&mut self) -> Result<(), SimulationError> {
        self.0.right_mouse_down()
    }

    pub fn left_mouse_up(&mut self) -> Result<(), SimulationError> {
        self.0.left_mouse_up()
    }

    pub fn middle_mouse_up(&mut self) -> Result<(), SimulationError> {
        self.0.middle_mouse_up()
    }

    pub fn right_mouse_up(&mut self) -> Result<(), SimulationError> {
        self.0.right_mouse_up()
    }

    pub fn wheel(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        self.0.wheel(x, y)
    }

    pub fn key_down(&mut self, key: Key) -> Result<(), SimulationError> {
        self.0.key_down(key)
    }

    pub fn key_up(&mut self, key: Key) -> Result<(), SimulationError> {
        self.0.key_up(key)
    }

    pub fn touch_down(&mut self, slot: i32, x: i32, y: i32) -> Result<(), SimulationError> {
        self.0.touch_down(slot, x, y)
    }

    pub fn touch_up(&mut self, slot: i32) -> Result<(), SimulationError> {
        self.0.touch_up(slot)
    }

    pub fn touch_move(&mut self, slot: i32, x: i32, y: i32) -> Result<(), SimulationError> {
        self.0.touch_move(slot, x, y)
    }

    /// This function gets the combined size of the virtual "screen space", NOT
    /// the size of the main monitor.
    ///
    /// For example, if someone has two 1366x768 monitors side-by-side, this
    /// function would return 1366*2x768 = 2732x768
    ///
    /// This is useful for many calculations involving input simulation.
    pub fn get_screen_size(&self) -> Result<(i32, i32), SimulationError> {
        self.0.get_screen_size()
    }
}
