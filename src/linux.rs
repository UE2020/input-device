use evdev::{
    AbsInfo, AbsoluteAxisCode, AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode,
    UinputAbsSetup, uinput::VirtualDevice,
};
use log::info;
use thiserror::Error;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::{connection::Connection, rust_connection::RustConnection};

/// An error returned by the `InputDeviceSimulator`.
#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("X11 reply error: {0}")]
    X11ReplyError(#[from] x11rb::errors::ReplyError),
    #[error("X11 connection error: {0}")]
    X11ConnectionError(#[from] x11rb::errors::ConnectionError),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("X11 connect error: {0}")]
    X11ConnectError(#[from] x11rb::errors::ConnectError),
}

/// An input simulator.
///
/// This struct contains the resources necessary to simulate all
/// input events.
pub struct InputDeviceSimulator {
    conn: RustConnection,
    abs_mouse_device: VirtualDevice,
    rel_mouse_device: VirtualDevice,
    //touch_device: VirtualDevice,
    //pen_device: VirtualDevice,
}

impl InputDeviceSimulator {
    /// Create a new input simulator.
    pub fn new() -> Result<InputDeviceSimulator, SimulationError> {
        let mut rel_mouse_device = VirtualDevice::builder()?
            .name("Simulated input-device Relative Mouse")
            .with_keys(&AttributeSet::from_iter([KeyCode::BTN_LEFT]))?
            .with_relative_axes(&AttributeSet::from_iter([
                RelativeAxisCode::REL_X,
                RelativeAxisCode::REL_Y,
                RelativeAxisCode::REL_WHEEL,
                RelativeAxisCode::REL_HWHEEL,
                RelativeAxisCode::REL_WHEEL_HI_RES,
                RelativeAxisCode::REL_HWHEEL_HI_RES,
            ]))?
            .build()?;

        for path in rel_mouse_device.enumerate_dev_nodes_blocking()? {
            let path = path?;
            info!("Available as {}", path.display());
        }

        let mut abs_mouse_device = VirtualDevice::builder()?
            .name("Simulated input-device Absolute Mouse")
            .with_keys(&AttributeSet::from_iter([KeyCode::BTN_LEFT]))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                AbsInfo::new(0, 0, 100_000, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                AbsInfo::new(0, 0, 100_000, 0, 0, 0),
            ))?
            .build()?;

        for path in abs_mouse_device.enumerate_dev_nodes_blocking()? {
            let path = path?;
            info!("Available as {}", path.display());
        }

        let (conn, _screen_num) = x11rb::connect(None)?;

        Ok(Self {
            rel_mouse_device,
            abs_mouse_device,
            conn,
        })
    }

    pub fn move_mouse_abs(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        let (width, height) = self.get_screen_size()?;
        let (x, y) = (
            (x as f64 / width as f64 * 100_000.0).round() as i32,
            (y as f64 / height as f64 * 100_000.0).round() as i32,
        );
        self.abs_mouse_device.emit(&[
            InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, x),
            InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_Y.0, y),
        ])?;
        Ok(())
    }

    pub fn move_mouse_rel(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        self.rel_mouse_device.emit(&[
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, x),
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, y),
        ])?;
        Ok(())
    }

    /// This function gets the combined size of the virtual "screen space", NOT
    /// the size of the main monitor.
    ///
    /// For example, if someone has two 1366x768 monitors side-by-side, this
    /// function would return 1366*2x768 = 2732x768
    ///
    /// This is useful for many calculations involving input simulation.
    pub fn get_screen_size(&self) -> Result<(i32, i32), SimulationError> {
        let root_window = self.conn.setup().roots[0].root;
        let geometry = self.conn.get_geometry(root_window)?.reply()?;
        Ok((geometry.width as _, geometry.height as _))
    }
}
