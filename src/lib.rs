#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

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
pub struct InputDeviceSimulator(PlatformImpl);

impl InputDeviceSimulator {
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

