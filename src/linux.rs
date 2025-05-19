use crate::Key;
use evdev::{
    AbsInfo, AbsoluteAxisCode, AttributeSet, EventType, InputEvent, KeyCode, PropType,
    RelativeAxisCode, UinputAbsSetup, uinput::VirtualDevice,
};
use log::info;
use strum::IntoEnumIterator;
use thiserror::Error;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xtest::ConnectionExt as XtestConnectionExt;
use x11rb::{connection::Connection, rust_connection::RustConnection};

/// An error returned by the [InputSimulator](crate::InputSimulator).
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

pub(crate) struct PlatformImpl {
    conn: RustConnection,
    rel_mouse_device: VirtualDevice,
    keyboard_device: VirtualDevice,
    touch_device: VirtualDevice,
    pen_device: VirtualDevice,
    wheel_x: i32,
    wheel_y: i32,
    last_pressure: f64
}

impl PlatformImpl {
    /// Create a new input simulator.
    pub(crate) fn new() -> Result<Self, SimulationError> {
        let mut keyboard_device = VirtualDevice::builder()?
            .name("Simulated input-device Keyboard")
            .with_keys(&AttributeSet::from_iter(
                Key::iter().map(|k| KeyCode::from(k)),
            ))?
            .build()?;

        for path in keyboard_device.enumerate_dev_nodes_blocking()? {
            let path = path?;
            info!("Keyboard device available as {}", path.display());
        }

        let mut rel_mouse_device = VirtualDevice::builder()?
            .name("Simulated input-device Relative Mouse")
            .with_keys(&AttributeSet::from_iter([
                KeyCode::BTN_LEFT,
                KeyCode::BTN_MIDDLE,
                KeyCode::BTN_RIGHT,
            ]))?
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
            info!("Relative mouse device available as {}", path.display());
        }

        let mut touch_device = VirtualDevice::builder()?
            .name("Simulated input-device Touchscreen")
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_SLOT,
                AbsInfo::new(0, 0, 9, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_POSITION_X,
                AbsInfo::new(0, 0, 100_000, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_POSITION_Y,
                AbsInfo::new(0, 0, 100_000, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_MT_TRACKING_ID,
                AbsInfo::new(0, 0, 65535, 0, 0, 0),
            ))?
            .with_properties(&AttributeSet::from_iter([PropType::DIRECT]))?
            .build()?;

        for path in touch_device.enumerate_dev_nodes_blocking()? {
            let path = path?;
            info!("Touchscreen device available as {}", path.display());
        }

        let mut pen_device = VirtualDevice::builder()?
            .name("Simulated input-device Pen Device")
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                AbsInfo::new(0, 0, 100_000, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                AbsInfo::new(0, 0, 100_000, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_PRESSURE,
                AbsInfo::new(0, 0, 100_000, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_TILT_X,
                AbsInfo::new(0, -90, 90, 0, 0, 0),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_TILT_Y,
                AbsInfo::new(0, -90, 90, 0, 0, 0),
            ))?
            .with_keys(&AttributeSet::from_iter([KeyCode::BTN_TOOL_PEN, KeyCode::BTN_TOUCH]))?
            .with_properties(&AttributeSet::from_iter([PropType::DIRECT]))?
            .with_properties(&AttributeSet::from_iter([PropType::POINTER]))?
            .build()?;

        for path in pen_device.enumerate_dev_nodes_blocking()? {
            let path = path?;
            info!("Pen device available as {}", path.display());
        }

        let (conn, _screen_num) = x11rb::connect(None)?;

        Ok(Self {
            wheel_x: 0,
            wheel_y: 0,
            last_pressure: 0.0,
            rel_mouse_device,
            keyboard_device,
            touch_device,
            pen_device,
            conn,
        })
    }

    pub(crate) fn move_mouse_abs(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        let root_window = self.conn.setup().roots[0].root;
        self.conn.warp_pointer(x11rb::NONE, root_window, 0, 0, 0, 0, x as i16, y as i16)?;
        self.conn.flush()?;
        Ok(())
    }

    pub(crate) fn move_mouse_rel(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        self.rel_mouse_device.emit(&[
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, x),
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, y),
        ])?;
        Ok(())
    }

    pub(crate) fn left_mouse_down(&mut self) -> Result<(), SimulationError> {
        self.conn.xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0)?;
        self.conn.flush()?;
        Ok(())
    }

    pub(crate) fn middle_mouse_down(&mut self) -> Result<(), SimulationError> {
        self.conn.xtest_fake_input(4, 2, 0, x11rb::NONE, 0, 0, 0)?;
        self.conn.flush()?;
        Ok(())
    }

    pub(crate) fn right_mouse_down(&mut self) -> Result<(), SimulationError> {
        self.conn.xtest_fake_input(4, 3, 0, x11rb::NONE, 0, 0, 0)?;
        self.conn.flush()?;
        Ok(())
    }

    pub(crate) fn left_mouse_up(&mut self) -> Result<(), SimulationError> {
        self.conn.xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0)?;
        self.conn.flush()?;
        Ok(())
    }

    pub(crate) fn middle_mouse_up(&mut self) -> Result<(), SimulationError> {
        self.conn.xtest_fake_input(5, 2, 0, x11rb::NONE, 0, 0, 0)?;
        self.conn.flush()?;
        Ok(())
    }

    pub(crate) fn right_mouse_up(&mut self) -> Result<(), SimulationError> {
        self.conn.xtest_fake_input(5, 3, 0, x11rb::NONE, 0, 0, 0)?;
        self.conn.flush()?;
        Ok(())
    }

    pub(crate) fn wheel(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        self.wheel_x += x;
        self.wheel_y += y;
        let mut events = vec![
            InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_HWHEEL_HI_RES.0,
                x,
            ),
            InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_WHEEL_HI_RES.0,
                y,
            ),
        ];
        if self.wheel_x.abs() > 120 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_HWHEEL.0,
                self.wheel_x / 120,
            ));
            self.wheel_x = self.wheel_x % 120;
        }
        if self.wheel_y.abs() > 120 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_WHEEL.0,
                self.wheel_y / 120,
            ));
            self.wheel_y = self.wheel_y % 120;
        }
        self.rel_mouse_device.emit(&events)?;
        Ok(())
    }

    pub(crate) fn key_down(&mut self, key: Key) -> Result<(), SimulationError> {
        self.keyboard_device
            .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::from(key).0, 1)])?;
        Ok(())
    }

    pub(crate) fn key_up(&mut self, key: Key) -> Result<(), SimulationError> {
        self.keyboard_device
            .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::from(key).0, 0)])?;
        Ok(())
    }

    pub(crate) fn touch_down(&mut self, slot: i32, x: i32, y: i32) -> Result<(), SimulationError> {
        let (width, height) = self.get_screen_size()?;
        let (x, y) = (
            (x as f64 / width as f64 * 100_000.0).round() as i32,
            (y as f64 / height as f64 * 100_000.0).round() as i32,
        );
        self.touch_device.emit(&[
            InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_MT_SLOT.0, slot),
            InputEvent::new(
                EventType::ABSOLUTE.0,
                AbsoluteAxisCode::ABS_MT_TRACKING_ID.0,
                slot,
            ),
            InputEvent::new(
                EventType::ABSOLUTE.0,
                AbsoluteAxisCode::ABS_MT_POSITION_X.0,
                x,
            ),
            InputEvent::new(
                EventType::ABSOLUTE.0,
                AbsoluteAxisCode::ABS_MT_POSITION_Y.0,
                y,
            ),
        ])?;
        Ok(())
    }

    pub(crate) fn touch_up(&mut self, slot: i32) -> Result<(), SimulationError> {
        self.touch_device.emit(&[
            InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_MT_SLOT.0, slot),
            InputEvent::new(
                EventType::ABSOLUTE.0,
                AbsoluteAxisCode::ABS_MT_TRACKING_ID.0,
                -1,
            ),
        ])?;
        Ok(())
    }

    pub(crate) fn touch_move(&mut self, slot: i32, x: i32, y: i32) -> Result<(), SimulationError> {
        let (width, height) = self.get_screen_size()?;
        let (x, y) = (
            (x as f64 / width as f64 * 100_000.0).round() as i32,
            (y as f64 / height as f64 * 100_000.0).round() as i32,
        );
        self.touch_device.emit(&[
            InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_MT_SLOT.0, slot),
            InputEvent::new(
                EventType::ABSOLUTE.0,
                AbsoluteAxisCode::ABS_MT_POSITION_X.0,
                x,
            ),
            InputEvent::new(
                EventType::ABSOLUTE.0,
                AbsoluteAxisCode::ABS_MT_POSITION_Y.0,
                y,
            ),
        ])?;
        Ok(())
    }

    pub(crate) fn pen(&mut self, x: i32, y: i32, pressure: f64, tilt_x: i32, tilt_y: i32) -> Result<(), SimulationError> {
        let (width, height) = self.get_screen_size()?;
        let (x, y) = (
            (x as f64 / width as f64 * 100_000.0).round() as i32,
            (y as f64 / height as f64 * 100_000.0).round() as i32,
        );
        let scaled_pressure = (pressure * 100_000.0).round() as i32;
        let mut events = vec![];
        if self.last_pressure < 0.00001 && pressure >= 0.00001 {
            events.push(InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOOL_PEN.0, 1));
            events.push(InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOUCH.0, 1));
        }
        events.push(InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, x));
        events.push(InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_Y.0, y));
        events.push(InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_PRESSURE.0, scaled_pressure));
        events.push(InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_TILT_X.0, tilt_x));
        events.push(InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_TILT_Y.0, tilt_y));
        if self.last_pressure >= 0.00001 && pressure < 0.00001 {
            events.push(InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOUCH.0, 0));
            events.push(InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOOL_PEN.0, 0));
        }
        self.pen_device.emit(&events)?;
        self.last_pressure = pressure;
        Ok(())
    }

    pub(crate) fn get_screen_size(&self) -> Result<(i32, i32), SimulationError> {
        let root_window = self.conn.setup().roots[0].root;
        let geometry = self.conn.get_geometry(root_window)?.reply()?;
        Ok((geometry.width as _, geometry.height as _))
    }
}

impl From<Key> for KeyCode {
    fn from(key: Key) -> Self {
        match key {
            Key::Esc => KeyCode::KEY_ESC,
            Key::Num1 => KeyCode::KEY_1,
            Key::Num2 => KeyCode::KEY_2,
            Key::Num3 => KeyCode::KEY_3,
            Key::Num4 => KeyCode::KEY_4,
            Key::Num5 => KeyCode::KEY_5,
            Key::Num6 => KeyCode::KEY_6,
            Key::Num7 => KeyCode::KEY_7,
            Key::Num8 => KeyCode::KEY_8,
            Key::Num9 => KeyCode::KEY_9,
            Key::Num0 => KeyCode::KEY_0,
            Key::Minus => KeyCode::KEY_MINUS,
            Key::Equal => KeyCode::KEY_EQUAL,
            Key::Backspace => KeyCode::KEY_BACKSPACE,
            Key::Tab => KeyCode::KEY_TAB,
            Key::Q => KeyCode::KEY_Q,
            Key::W => KeyCode::KEY_W,
            Key::E => KeyCode::KEY_E,
            Key::R => KeyCode::KEY_R,
            Key::T => KeyCode::KEY_T,
            Key::Y => KeyCode::KEY_Y,
            Key::U => KeyCode::KEY_U,
            Key::I => KeyCode::KEY_I,
            Key::O => KeyCode::KEY_O,
            Key::P => KeyCode::KEY_P,
            Key::LeftBrace => KeyCode::KEY_LEFTBRACE,
            Key::RightBrace => KeyCode::KEY_RIGHTBRACE,
            Key::Enter => KeyCode::KEY_ENTER,
            Key::LeftCtrl => KeyCode::KEY_LEFTCTRL,
            Key::A => KeyCode::KEY_A,
            Key::S => KeyCode::KEY_S,
            Key::D => KeyCode::KEY_D,
            Key::F => KeyCode::KEY_F,
            Key::G => KeyCode::KEY_G,
            Key::H => KeyCode::KEY_H,
            Key::J => KeyCode::KEY_J,
            Key::K => KeyCode::KEY_K,
            Key::L => KeyCode::KEY_L,
            Key::Semicolon => KeyCode::KEY_SEMICOLON,
            Key::Apostrophe => KeyCode::KEY_APOSTROPHE,
            Key::Grave => KeyCode::KEY_GRAVE,
            Key::LeftShift => KeyCode::KEY_LEFTSHIFT,
            Key::Backslash => KeyCode::KEY_BACKSLASH,
            Key::Z => KeyCode::KEY_Z,
            Key::X => KeyCode::KEY_X,
            Key::C => KeyCode::KEY_C,
            Key::V => KeyCode::KEY_V,
            Key::B => KeyCode::KEY_B,
            Key::N => KeyCode::KEY_N,
            Key::M => KeyCode::KEY_M,
            Key::Comma => KeyCode::KEY_COMMA,
            Key::Dot => KeyCode::KEY_DOT,
            Key::Slash => KeyCode::KEY_SLASH,
            Key::RightShift => KeyCode::KEY_RIGHTSHIFT,
            Key::KpAsterisk => KeyCode::KEY_KPASTERISK,
            Key::LeftAlt => KeyCode::KEY_LEFTALT,
            Key::Space => KeyCode::KEY_SPACE,
            Key::CapsLock => KeyCode::KEY_CAPSLOCK,
            Key::F1 => KeyCode::KEY_F1,
            Key::F2 => KeyCode::KEY_F2,
            Key::F3 => KeyCode::KEY_F3,
            Key::F4 => KeyCode::KEY_F4,
            Key::F5 => KeyCode::KEY_F5,
            Key::F6 => KeyCode::KEY_F6,
            Key::F7 => KeyCode::KEY_F7,
            Key::F8 => KeyCode::KEY_F8,
            Key::F9 => KeyCode::KEY_F9,
            Key::F10 => KeyCode::KEY_F10,
            Key::NumLock => KeyCode::KEY_NUMLOCK,
            Key::ScrollLock => KeyCode::KEY_SCROLLLOCK,
            Key::Kp7 => KeyCode::KEY_KP7,
            Key::Kp8 => KeyCode::KEY_KP8,
            Key::Kp9 => KeyCode::KEY_KP9,
            Key::KpMinus => KeyCode::KEY_KPMINUS,
            Key::Kp4 => KeyCode::KEY_KP4,
            Key::Kp5 => KeyCode::KEY_KP5,
            Key::Kp6 => KeyCode::KEY_KP6,
            Key::KpPlus => KeyCode::KEY_KPPLUS,
            Key::Kp1 => KeyCode::KEY_KP1,
            Key::Kp2 => KeyCode::KEY_KP2,
            Key::Kp3 => KeyCode::KEY_KP3,
            Key::Kp0 => KeyCode::KEY_KP0,
            Key::KpDot => KeyCode::KEY_KPDOT,
            Key::ZenkakuHankaku => KeyCode::KEY_ZENKAKUHANKAKU,
            Key::IntlBackslash => KeyCode::KEY_102ND,
            Key::F11 => KeyCode::KEY_F11,
            Key::F12 => KeyCode::KEY_F12,
            Key::Ro => KeyCode::KEY_RO,
            Key::Katakana => KeyCode::KEY_KATAKANA,
            Key::Hiragana => KeyCode::KEY_HIRAGANA,
            Key::Henkan => KeyCode::KEY_HENKAN,
            Key::KatakanaHiragana => KeyCode::KEY_KATAKANAHIRAGANA,
            Key::Muhenkan => KeyCode::KEY_MUHENKAN,
            Key::KpJpComma => KeyCode::KEY_KPJPCOMMA,
            Key::KpEnter => KeyCode::KEY_KPENTER,
            Key::RightCtrl => KeyCode::KEY_RIGHTCTRL,
            Key::KpSlash => KeyCode::KEY_KPSLASH,
            Key::SysRq => KeyCode::KEY_SYSRQ,
            Key::RightAlt => KeyCode::KEY_RIGHTALT,
            Key::Home => KeyCode::KEY_HOME,
            Key::Up => KeyCode::KEY_UP,
            Key::PageUp => KeyCode::KEY_PAGEUP,
            Key::Left => KeyCode::KEY_LEFT,
            Key::Right => KeyCode::KEY_RIGHT,
            Key::End => KeyCode::KEY_END,
            Key::Down => KeyCode::KEY_DOWN,
            Key::PageDown => KeyCode::KEY_PAGEDOWN,
            Key::Insert => KeyCode::KEY_INSERT,
            Key::Delete => KeyCode::KEY_DELETE,
            Key::Macro => KeyCode::KEY_MACRO,
            Key::Mute => KeyCode::KEY_MUTE,
            Key::VolumeDown => KeyCode::KEY_VOLUMEDOWN,
            Key::VolumeUp => KeyCode::KEY_VOLUMEUP,
            Key::Power => KeyCode::KEY_POWER,
            Key::KpEqual => KeyCode::KEY_KPEQUAL,
            Key::KpPlusMinus => KeyCode::KEY_KPPLUSMINUS,
            Key::Pause => KeyCode::KEY_PAUSE,
            Key::KpComma => KeyCode::KEY_KPCOMMA,
            Key::Hanguel => KeyCode::KEY_HANGEUL,
            Key::Hanja => KeyCode::KEY_HANJA,
            Key::Yen => KeyCode::KEY_YEN,
            Key::LeftMeta => KeyCode::KEY_LEFTMETA,
            Key::RightMeta => KeyCode::KEY_RIGHTMETA,
            Key::Compose => KeyCode::KEY_COMPOSE,
            Key::Stop => KeyCode::KEY_STOP,
            Key::Help => KeyCode::KEY_HELP,
            Key::Calc => KeyCode::KEY_CALC,
            Key::Sleep => KeyCode::KEY_SLEEP,
            Key::WakeUp => KeyCode::KEY_WAKEUP,
            Key::Mail => KeyCode::KEY_MAIL,
            Key::Bookmarks => KeyCode::KEY_BOOKMARKS,
            Key::Computer => KeyCode::KEY_COMPUTER,
            Key::Back => KeyCode::KEY_BACK,
            Key::Forward => KeyCode::KEY_FORWARD,
            Key::NextSong => KeyCode::KEY_NEXTSONG,
            Key::PlayPause => KeyCode::KEY_PLAYPAUSE,
            Key::PreviousSong => KeyCode::KEY_PREVIOUSSONG,
            Key::StopCD => KeyCode::KEY_STOPCD,
            Key::Homepage => KeyCode::KEY_HOMEPAGE,
            Key::Refresh => KeyCode::KEY_REFRESH,
            Key::F13 => KeyCode::KEY_F13,
            Key::F14 => KeyCode::KEY_F14,
            Key::F15 => KeyCode::KEY_F15,
            Key::F23 => KeyCode::KEY_F23,
            Key::Camera => KeyCode::KEY_CAMERA,
            Key::Search => KeyCode::KEY_SEARCH,
            Key::BrightnessDown => KeyCode::KEY_BRIGHTNESSDOWN,
            Key::BrightnessUp => KeyCode::KEY_BRIGHTNESSUP,
            Key::Media => KeyCode::KEY_MEDIA,
            Key::SwitchVideoMode => KeyCode::KEY_SWITCHVIDEOMODE,
            Key::Battery => KeyCode::KEY_BATTERY,
            Key::Wlan => KeyCode::KEY_WLAN,
            Key::Dvd => KeyCode::KEY_DVD,
            Key::FnEsc => KeyCode::KEY_FN_ESC,
            _ => KeyCode::KEY_UNKNOWN,
        }
    }
}
