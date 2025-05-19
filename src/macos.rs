use core_graphics::event::*;
use core_graphics::event_source::*;
use core_graphics::geometry::CGPoint;

use crate::Key;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Windows error: {0}")]
    WindowsError(#[from] windows::core::Error),
}

pub(crate) struct PlatformImpl;

impl PlatformImpl {
    pub(crate) fn new() -> Result<Self, SimulationError> {
        Self
    }

    pub(crate) fn move_mouse_abs() -> Result<(), SimulationError> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
        let event = CGEvent::new_mouse_event(
            &source,
            CGEventType::MouseMoved,
            CGPoint::new(x, y),
            CGMouseButton::Left,
        ).unwrap();

        event.post(CGEventTapLocation::HID);
    }
}
