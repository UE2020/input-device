use core_graphics::display::CGDisplay;
use core_graphics::event::*;
use core_graphics::event_source::*;
use core_graphics::geometry::CGPoint;

use std::time::{Instant, Duration};

use crate::Key;
use thiserror::Error;

extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

fn has_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Core graphics error")]
    CoreGraphicsError,
    #[error("The application does not have the requisite accessibility permissions to perform input simulation")]
    PermissionError,
}

pub(crate) struct PlatformImpl {
    source: CGEventSource,
    display: CGDisplay,

    left_mouse_down: bool,
    right_mouse_down: bool,

    last_left_click: Instant,
    last_right_click: Instant,
}

impl PlatformImpl {
    pub(crate) fn new() -> Result<Self, SimulationError> {
        if !has_permission() {
            return Err(SimulationError::PermissionError);
        }
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| SimulationError::CoreGraphicsError)?;
        let display = CGDisplay::main();
        Ok(Self {
            source,
            display,
            left_mouse_down: false,
            right_mouse_down: false,
            last_left_click: Instant::now(),
            last_right_click: Instant::now(),
        })
    }

    fn show_cursor(&self) -> Result<(), SimulationError> {
        self.display.show_cursor().map_err(|_| SimulationError::CoreGraphicsError)?;
        Ok(())
    }

    pub(crate) fn move_mouse_abs(&self, x: i32, y: i32) -> Result<(), SimulationError> {
        let event_type = if self.left_mouse_down {
            CGEventType::LeftMouseDragged
        } else if self.right_mouse_down {
            CGEventType::RightMouseDragged
        } else {
            CGEventType::MouseMoved
        };

        let event = CGEvent::new_mouse_event(
            self.source.clone(),
            event_type,
            CGPoint::new(x as _, y as _),
            CGMouseButton::Left,
        )
        .map_err(|_| SimulationError::CoreGraphicsError)?;

        event.post(CGEventTapLocation::Session);

        Ok(())
    }

    pub(crate) fn move_mouse_rel(&self, x: i32, y: i32) -> Result<(), SimulationError> {
        let event_type = if self.left_mouse_down {
            CGEventType::LeftMouseDragged
        } else if self.right_mouse_down {
            CGEventType::RightMouseDragged
        } else {
            CGEventType::MouseMoved
        };

        // Get mouse position
        let event =
            CGEvent::new(self.source.clone()).map_err(|_| SimulationError::CoreGraphicsError)?;
        let loc = event.location();
        let event = CGEvent::new_mouse_event(
            self.source.clone(),
            event_type,
            CGPoint::new(x as f64 + loc.x, y as f64 + loc.y),
            CGMouseButton::Left,
        )
        .map_err(|_| SimulationError::CoreGraphicsError)?;

        event.set_integer_value_field(EventField::MOUSE_EVENT_DELTA_X, x as i64);
        event.set_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y, y as i64);

        event.post(CGEventTapLocation::Session);

        Ok(())
    }

    pub(crate) fn left_mouse_down(&mut self) -> Result<(), SimulationError> {
        let now = Instant::now();
        let is_double_click = (now - self.last_left_click) < Duration::from_millis(500);
        self.left_mouse_down = true;
        // Get mouse position
        let event =
            CGEvent::new(self.source.clone()).map_err(|_| SimulationError::CoreGraphicsError)?;
        let loc = event.location();
        let event = CGEvent::new_mouse_event(
            self.source.clone(),
            CGEventType::LeftMouseDown,
            loc,
            CGMouseButton::Left,
        )
        .map_err(|_| SimulationError::CoreGraphicsError)?;
        event.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, if is_double_click { 2 } else { 1 });
        event.post(CGEventTapLocation::Session);

        Ok(())
    }

    pub(crate) fn middle_mouse_down(&self) -> Result<(), SimulationError> {
        // TODO: no-op
        Ok(())
    }

    pub(crate) fn right_mouse_down(&mut self) -> Result<(), SimulationError> {
        let now = Instant::now();
        let is_double_click = (now - self.last_right_click) < Duration::from_millis(500);
        self.right_mouse_down = true;
        // Get mouse position
        let event =
            CGEvent::new(self.source.clone()).map_err(|_| SimulationError::CoreGraphicsError)?;
        let loc = event.location();
        let event = CGEvent::new_mouse_event(
            self.source.clone(),
            CGEventType::RightMouseDown,
            loc,
            CGMouseButton::Right,
        )
        .map_err(|_| SimulationError::CoreGraphicsError)?;
        event.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, if is_double_click { 2 } else { 1 });
        event.post(CGEventTapLocation::Session);

        Ok(())
    }

    pub(crate) fn left_mouse_up(&mut self) -> Result<(), SimulationError> {
        let now = Instant::now();
        let is_double_click = (now - self.last_left_click) < Duration::from_millis(500);
        self.last_left_click = now;
        self.left_mouse_down = false;
        // Get mouse position
        let event =
            CGEvent::new(self.source.clone()).map_err(|_| SimulationError::CoreGraphicsError)?;
        let loc = event.location();
        let event = CGEvent::new_mouse_event(
            self.source.clone(),
            CGEventType::LeftMouseUp,
            loc,
            CGMouseButton::Left,
        )
        .map_err(|_| SimulationError::CoreGraphicsError)?;
        event.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, if is_double_click { 2 } else { 1 });
        event.post(CGEventTapLocation::Session);

        Ok(())
    }

    pub(crate) fn middle_mouse_up(&self) -> Result<(), SimulationError> {
        // TODO: no-op
        Ok(())
    }

    pub(crate) fn right_mouse_up(&mut self) -> Result<(), SimulationError> {
        let now = Instant::now();
        let is_double_click = (now - self.last_right_click) < Duration::from_millis(500);
        self.last_right_click = now;
        self.right_mouse_down = false;
        // Get mouse position
        let event =
            CGEvent::new(self.source.clone()).map_err(|_| SimulationError::CoreGraphicsError)?;
        let loc = event.location();
        let event = CGEvent::new_mouse_event(
            self.source.clone(),
            CGEventType::RightMouseUp,
            loc,
            CGMouseButton::Right,
        )
        .map_err(|_| SimulationError::CoreGraphicsError)?;
        event.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, if is_double_click { 2 } else { 1 });
        event.post(CGEventTapLocation::Session);

        Ok(())
    }

    pub(crate) fn wheel(&self, x: i32, y: i32) -> Result<(), SimulationError> {
        let event =
            CGEvent::new_scroll_event(self.source.clone(), ScrollEventUnit::PIXEL, 2, y, x, 0)
                .map_err(|_| SimulationError::CoreGraphicsError)?;
        event.post(CGEventTapLocation::Session);
        Ok(())
    }

    pub(crate) fn get_screen_size(&self) -> Result<(i32, i32), SimulationError> {
        Ok((
            self.display.pixels_wide() as _,
            self.display.pixels_high() as _,
        ))
    }

    pub(crate) fn key_down(&self, key: Key) -> Result<(), SimulationError> {
        if let Some(keycode) = key_to_cgkeycode(key) {
            let event = CGEvent::new_keyboard_event(self.source.clone(), keycode, true)
                .map_err(|_| SimulationError::CoreGraphicsError)?;
            let flags = event.get_flags();
            event.set_flags(flags & !CGEventFlagSecondaryFn);
            event.post(CGEventTapLocation::HID);
        }
        Ok(())
    }

    pub(crate) fn key_up(&mut self, key: Key) -> Result<(), SimulationError> {
        if let Some(keycode) = key_to_cgkeycode(key) {
            let event = CGEvent::new_keyboard_event(self.source.clone(), keycode, false)
                .map_err(|_| SimulationError::CoreGraphicsError)?;
            event.set_flags(CGEventFlags::CGEventFlagNull);
            event.post(CGEventTapLocation::HID);
            self.show_cursor()?;
        }
        Ok(())
    }

    pub(crate) fn touch_down(
        &mut self,
        _slot: i32,
        _x: i32,
        _y: i32,
    ) -> Result<(), SimulationError> {
        // TODO: no-op
        Ok(())
    }

    pub(crate) fn touch_up(&mut self, _slot: i32) -> Result<(), SimulationError> {
        // TODO: no-op
        Ok(())
    }

    pub(crate) fn touch_move(
        &mut self,
        _slot: i32,
        _x: i32,
        _y: i32,
    ) -> Result<(), SimulationError> {
        // TODO: no-op
        Ok(())
    }

    pub(crate) fn pen(
        &mut self,
        _x: i32,
        _y: i32,
        _pressure: f64,
        _tilt_x: i32,
        _tilt_y: i32,
    ) -> Result<(), SimulationError> {
        // TODO: no-op
        Ok(())
    }
}

// Source: https://github.com/servo/core-foundation-rs/blob/61b90e72da0f37f63509b1f43d752caea56b7a9e/core-graphics/src/event.rs#L65
#[allow(unused)]
mod keycodes {
    use core_graphics::event::CGKeyCode;

    pub const ANSI_A: CGKeyCode = 0x00;
    pub const ANSI_S: CGKeyCode = 0x01;
    pub const ANSI_D: CGKeyCode = 0x02;
    pub const ANSI_F: CGKeyCode = 0x03;
    pub const ANSI_H: CGKeyCode = 0x04;
    pub const ANSI_G: CGKeyCode = 0x05;
    pub const ANSI_Z: CGKeyCode = 0x06;
    pub const ANSI_X: CGKeyCode = 0x07;
    pub const ANSI_C: CGKeyCode = 0x08;
    pub const ANSI_V: CGKeyCode = 0x09;
    pub const ANSI_B: CGKeyCode = 0x0B;
    pub const ANSI_Q: CGKeyCode = 0x0C;
    pub const ANSI_W: CGKeyCode = 0x0D;
    pub const ANSI_E: CGKeyCode = 0x0E;
    pub const ANSI_R: CGKeyCode = 0x0F;
    pub const ANSI_Y: CGKeyCode = 0x10;
    pub const ANSI_T: CGKeyCode = 0x11;
    pub const ANSI_1: CGKeyCode = 0x12;
    pub const ANSI_2: CGKeyCode = 0x13;
    pub const ANSI_3: CGKeyCode = 0x14;
    pub const ANSI_4: CGKeyCode = 0x15;
    pub const ANSI_6: CGKeyCode = 0x16;
    pub const ANSI_5: CGKeyCode = 0x17;
    pub const ANSI_EQUAL: CGKeyCode = 0x18;
    pub const ANSI_9: CGKeyCode = 0x19;
    pub const ANSI_7: CGKeyCode = 0x1A;
    pub const ANSI_MINUS: CGKeyCode = 0x1B;
    pub const ANSI_8: CGKeyCode = 0x1C;
    pub const ANSI_0: CGKeyCode = 0x1D;
    pub const ANSI_RIGHT_BRACKET: CGKeyCode = 0x1E;
    pub const ANSI_O: CGKeyCode = 0x1F;
    pub const ANSI_U: CGKeyCode = 0x20;
    pub const ANSI_LEFT_BRACKET: CGKeyCode = 0x21;
    pub const ANSI_I: CGKeyCode = 0x22;
    pub const ANSI_P: CGKeyCode = 0x23;
    pub const ANSI_L: CGKeyCode = 0x25;
    pub const ANSI_J: CGKeyCode = 0x26;
    pub const ANSI_QUOTE: CGKeyCode = 0x27;
    pub const ANSI_K: CGKeyCode = 0x28;
    pub const ANSI_SEMICOLON: CGKeyCode = 0x29;
    pub const ANSI_BACKSLASH: CGKeyCode = 0x2A;
    pub const ANSI_COMMA: CGKeyCode = 0x2B;
    pub const ANSI_SLASH: CGKeyCode = 0x2C;
    pub const ANSI_N: CGKeyCode = 0x2D;
    pub const ANSI_M: CGKeyCode = 0x2E;
    pub const ANSI_PERIOD: CGKeyCode = 0x2F;
    pub const ANSI_GRAVE: CGKeyCode = 0x32;
    pub const ANSI_KEYPAD_DECIMAL: CGKeyCode = 0x41;
    pub const ANSI_KEYPAD_MULTIPLY: CGKeyCode = 0x43;
    pub const ANSI_KEYPAD_PLUS: CGKeyCode = 0x45;
    pub const ANSI_KEYPAD_CLEAR: CGKeyCode = 0x47;
    pub const ANSI_KEYPAD_DIVIDE: CGKeyCode = 0x4B;
    pub const ANSI_KEYPAD_ENTER: CGKeyCode = 0x4C;
    pub const ANSI_KEYPAD_MINUS: CGKeyCode = 0x4E;
    pub const ANSI_KEYPAD_EQUAL: CGKeyCode = 0x51;
    pub const ANSI_KEYPAD_0: CGKeyCode = 0x52;
    pub const ANSI_KEYPAD_1: CGKeyCode = 0x53;
    pub const ANSI_KEYPAD_2: CGKeyCode = 0x54;
    pub const ANSI_KEYPAD_3: CGKeyCode = 0x55;
    pub const ANSI_KEYPAD_4: CGKeyCode = 0x56;
    pub const ANSI_KEYPAD_5: CGKeyCode = 0x57;
    pub const ANSI_KEYPAD_6: CGKeyCode = 0x58;
    pub const ANSI_KEYPAD_7: CGKeyCode = 0x59;
    pub const ANSI_KEYPAD_8: CGKeyCode = 0x5B;
    pub const ANSI_KEYPAD_9: CGKeyCode = 0x5C;
    pub const RETURN: CGKeyCode = 0x24;
    pub const TAB: CGKeyCode = 0x30;
    pub const SPACE: CGKeyCode = 0x31;
    pub const DELETE: CGKeyCode = 0x33;
    pub const ESCAPE: CGKeyCode = 0x35;
    pub const COMMAND: CGKeyCode = 0x37;
    pub const SHIFT: CGKeyCode = 0x38;
    pub const CAPS_LOCK: CGKeyCode = 0x39;
    pub const OPTION: CGKeyCode = 0x3A;
    pub const CONTROL: CGKeyCode = 0x3B;
    pub const RIGHT_COMMAND: CGKeyCode = 0x36;
    pub const RIGHT_SHIFT: CGKeyCode = 0x3C;
    pub const RIGHT_OPTION: CGKeyCode = 0x3D;
    pub const RIGHT_CONTROL: CGKeyCode = 0x3E;
    pub const FUNCTION: CGKeyCode = 0x3F;
    pub const F17: CGKeyCode = 0x40;
    pub const VOLUME_UP: CGKeyCode = 0x48;
    pub const VOLUME_DOWN: CGKeyCode = 0x49;
    pub const MUTE: CGKeyCode = 0x4A;
    pub const F18: CGKeyCode = 0x4F;
    pub const F19: CGKeyCode = 0x50;
    pub const F20: CGKeyCode = 0x5A;
    pub const F5: CGKeyCode = 0x60;
    pub const F6: CGKeyCode = 0x61;
    pub const F7: CGKeyCode = 0x62;
    pub const F3: CGKeyCode = 0x63;
    pub const F8: CGKeyCode = 0x64;
    pub const F9: CGKeyCode = 0x65;
    pub const F11: CGKeyCode = 0x67;
    pub const F13: CGKeyCode = 0x69;
    pub const F16: CGKeyCode = 0x6A;
    pub const F14: CGKeyCode = 0x6B;
    pub const F10: CGKeyCode = 0x6D;
    pub const F12: CGKeyCode = 0x6F;
    pub const F15: CGKeyCode = 0x71;
    pub const HELP: CGKeyCode = 0x72;
    pub const HOME: CGKeyCode = 0x73;
    pub const PAGE_UP: CGKeyCode = 0x74;
    pub const FORWARD_DELETE: CGKeyCode = 0x75;
    pub const F4: CGKeyCode = 0x76;
    pub const END: CGKeyCode = 0x77;
    pub const F2: CGKeyCode = 0x78;
    pub const PAGE_DOWN: CGKeyCode = 0x79;
    pub const F1: CGKeyCode = 0x7A;
    pub const LEFT_ARROW: CGKeyCode = 0x7B;
    pub const RIGHT_ARROW: CGKeyCode = 0x7C;
    pub const DOWN_ARROW: CGKeyCode = 0x7D;
    pub const UP_ARROW: CGKeyCode = 0x7E;
    pub const ISO_SECTION: CGKeyCode = 0x0A;
    pub const JIS_YEN: CGKeyCode = 0x5D;
    pub const JIS_UNDERSCORE: CGKeyCode = 0x5E;
    pub const JIS_KEYPAD_COMMA: CGKeyCode = 0x5F;
    pub const JIS_EISU: CGKeyCode = 0x66;
    pub const JIS_KANA: CGKeyCode = 0x68;
}

fn key_to_cgkeycode(key: Key) -> Option<CGKeyCode> {
    use keycodes::*;

    Some(match key {
        Key::A => ANSI_A,
        Key::B => ANSI_B,
        Key::C => ANSI_C,
        Key::D => ANSI_D,
        Key::E => ANSI_E,
        Key::F => ANSI_F,
        Key::G => ANSI_G,
        Key::H => ANSI_H,
        Key::I => ANSI_I,
        Key::J => ANSI_J,
        Key::K => ANSI_K,
        Key::L => ANSI_L,
        Key::M => ANSI_M,
        Key::N => ANSI_N,
        Key::O => ANSI_O,
        Key::P => ANSI_P,
        Key::Q => ANSI_Q,
        Key::R => ANSI_R,
        Key::S => ANSI_S,
        Key::T => ANSI_T,
        Key::U => ANSI_U,
        Key::V => ANSI_V,
        Key::W => ANSI_W,
        Key::X => ANSI_X,
        Key::Y => ANSI_Y,
        Key::Z => ANSI_Z,

        Key::Num1 => ANSI_1,
        Key::Num2 => ANSI_2,
        Key::Num3 => ANSI_3,
        Key::Num4 => ANSI_4,
        Key::Num5 => ANSI_5,
        Key::Num6 => ANSI_6,
        Key::Num7 => ANSI_7,
        Key::Num8 => ANSI_8,
        Key::Num9 => ANSI_9,
        Key::Num0 => ANSI_0,

        Key::Minus => ANSI_MINUS,
        Key::Equal => ANSI_EQUAL,
        Key::LeftBrace => ANSI_LEFT_BRACKET,
        Key::RightBrace => ANSI_RIGHT_BRACKET,
        Key::Backslash => ANSI_BACKSLASH,
        Key::Semicolon => ANSI_SEMICOLON,
        Key::Apostrophe => ANSI_QUOTE,
        Key::Grave => ANSI_GRAVE,
        Key::Comma => ANSI_COMMA,
        Key::Dot => ANSI_PERIOD,
        Key::Slash => ANSI_SLASH,

        Key::Enter => RETURN,
        Key::Tab => TAB,
        Key::Space => SPACE,
        Key::Backspace => DELETE,
        Key::Esc => ESCAPE,

        Key::LeftCtrl => CONTROL,
        Key::RightCtrl => RIGHT_CONTROL,
        Key::LeftAlt => OPTION,
        Key::RightAlt => RIGHT_OPTION,
        Key::LeftShift => SHIFT,
        Key::RightShift => RIGHT_SHIFT,
        Key::CapsLock => CAPS_LOCK,
        Key::LeftMeta => COMMAND,
        Key::RightMeta => RIGHT_COMMAND,
        Key::Help => HELP,

        Key::F1 => F1,
        Key::F2 => F2,
        Key::F3 => F3,
        Key::F4 => F4,
        Key::F5 => F5,
        Key::F6 => F6,
        Key::F7 => F7,
        Key::F8 => F8,
        Key::F9 => F9,
        Key::F10 => F10,
        Key::F11 => F11,
        Key::F12 => F12,
        Key::F13 => F13,
        Key::F14 => F14,
        Key::F15 => F15,

        Key::Home => HOME,
        Key::End => END,
        Key::PageUp => PAGE_UP,
        Key::PageDown => PAGE_DOWN,
        Key::Up => UP_ARROW,
        Key::Down => DOWN_ARROW,
        Key::Left => LEFT_ARROW,
        Key::Right => RIGHT_ARROW,

        Key::VolumeUp => VOLUME_UP,
        Key::VolumeDown => VOLUME_DOWN,
        Key::Mute => MUTE,

        Key::IntlBackslash => ISO_SECTION,
        Key::Yen => JIS_YEN,
        Key::KpJpComma => JIS_KEYPAD_COMMA,
        Key::KpEqual => ANSI_KEYPAD_EQUAL,
        Key::KpEnter => ANSI_KEYPAD_ENTER,
        Key::KpSlash => ANSI_KEYPAD_DIVIDE,
        Key::KpAsterisk => ANSI_KEYPAD_MULTIPLY,
        Key::KpMinus => ANSI_KEYPAD_MINUS,
        Key::KpPlus => ANSI_KEYPAD_PLUS,
        Key::Kp0 => ANSI_KEYPAD_0,
        Key::Kp1 => ANSI_KEYPAD_1,
        Key::Kp2 => ANSI_KEYPAD_2,
        Key::Kp3 => ANSI_KEYPAD_3,
        Key::Kp4 => ANSI_KEYPAD_4,
        Key::Kp5 => ANSI_KEYPAD_5,
        Key::Kp6 => ANSI_KEYPAD_6,
        Key::Kp7 => ANSI_KEYPAD_7,
        Key::Kp8 => ANSI_KEYPAD_8,
        Key::Kp9 => ANSI_KEYPAD_9,
        Key::KpDot => ANSI_KEYPAD_DECIMAL,
        Key::KpPlusMinus => ANSI_KEYPAD_EQUAL, // Best effort

        Key::ZenkakuHankaku
        | Key::Ro
        | Key::Katakana
        | Key::Hiragana
        | Key::Henkan
        | Key::KatakanaHiragana
        | Key::Muhenkan
        | Key::Hanguel
        | Key::Hanja
        | Key::FnEsc
        | Key::KpComma => return None,

        _ => return None, // Unimplemented / unsupported on macOS
    })
}
