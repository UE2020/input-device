use crate::Key;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use windows::Win32::System::Performance;
use windows::Win32::UI::Controls;
use windows::Win32::UI::HiDpi;
use windows::Win32::UI::Input::KeyboardAndMouse;
use windows::Win32::UI::Input::Pointer;
use windows::Win32::UI::WindowsAndMessaging;

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Windows error: {0}")]
    WindowsError(#[from] windows::core::Error),
}

#[derive(Clone, Copy)]
struct Touch {
    x: i32,
    y: i32,
    active: bool,
    /// If set, the next tick will emit this exact transition flag once.
    /// After emission, it's cleared (and `active` set false if it was UP).
    pending: Option<Pointer::POINTER_FLAGS>,
}

impl Default for Touch {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            active: false,
            pending: None,
        }
    }
}

impl Touch {
    fn into_touchinfo(
        self,
        flags: Pointer::POINTER_FLAGS,
        time: i64,
        index: u32,
    ) -> Pointer::POINTER_TOUCH_INFO {
        let mut touch_info: Pointer::POINTER_TOUCH_INFO = unsafe { std::mem::zeroed() };
        touch_info.pointerInfo.pointerType = WindowsAndMessaging::PT_TOUCH;
        touch_info.pointerInfo.pointerId = index;
        touch_info.pointerInfo.PerformanceCount = time as u64;
        touch_info.pointerInfo.ptPixelLocation.x = self.x;
        touch_info.pointerInfo.ptPixelLocation.y = self.y;
        touch_info.pointerInfo.pointerFlags = flags;

        // Provide a stable contact area + orientation; some stacks rely on this.
        touch_info.touchMask =
            WindowsAndMessaging::TOUCH_MASK_CONTACTAREA | WindowsAndMessaging::TOUCH_MASK_ORIENTATION;
        touch_info.rcContact.top = self.y - 2;
        touch_info.rcContact.bottom = self.y + 2;
        touch_info.rcContact.left = self.x - 2;
        touch_info.rcContact.right = self.x + 2;
        touch_info.orientation = 90;

        touch_info
    }
}

pub(crate) struct PlatformImpl {
    pen_device: Controls::HSYNTHETICPOINTERDEVICE,
    touches: Arc<Mutex<[Touch; 10]>>,
    last_pressure: f64,
}

impl PlatformImpl {
    pub(crate) fn new() -> Result<Self, SimulationError> {
        unsafe {
            HiDpi::SetProcessDpiAwareness(HiDpi::PROCESS_PER_MONITOR_DPI_AWARE)?;
            Pointer::InitializeTouchInjection(10, Pointer::TOUCH_FEEDBACK_DEFAULT)?;
        }

        let touches = Arc::new(Mutex::new([Touch::default(); 10]));
        let touches_clone = Arc::downgrade(&touches);

        // Worker thread: the ONLY place we call InjectTouchInput.
        std::thread::spawn(move || loop {
            let stored = match touches_clone.upgrade() {
                Some(s) => s,
                None => break,
            };

            // Build one batch per tick with consistent timestamp.
            let mut time: i64 = 0;
            unsafe { Performance::QueryPerformanceCounter(&mut time) };

            // We'll mutate state based on what we successfully inject. To avoid
            // racing with user code, we hold the lock during injection.
            let mut guard = stored.lock().unwrap();

            let mut events: Vec<Pointer::POINTER_TOUCH_INFO> = Vec::with_capacity(10);
            // Track which indices emitted UP so we can deactivate them.
            let mut emitted_up: [bool; 10] = [false; 10];

            for (i, t) in guard.iter().copied().enumerate() {
                // Decide flags for this tick.
                let flags = if let Some(pending) = t.pending {
                    // Apply app-requested transition exactly once.
                    match pending {
                        f if f.contains(Pointer::POINTER_FLAG_DOWN) => {
                            Some(pending | Pointer::POINTER_FLAG_INRANGE | Pointer::POINTER_FLAG_INCONTACT)
                        }
                        f if f.contains(Pointer::POINTER_FLAG_UP) => {
                            emitted_up[i] = true;
                            Some(Pointer::POINTER_FLAG_UP)
                        }
                        _ => {
                            // Treat any other pending as an UPDATE-like state.
                            Some(Pointer::POINTER_FLAG_UPDATE | Pointer::POINTER_FLAG_INRANGE | Pointer::POINTER_FLAG_INCONTACT)
                        }
                    }
                } else if t.active {
                    // Refresh frame to keep the contact alive.
                    Some(
                        Pointer::POINTER_FLAG_UPDATE
                            | Pointer::POINTER_FLAG_INRANGE
                            | Pointer::POINTER_FLAG_INCONTACT,
                    )
                } else {
                    None
                };

                if let Some(f) = flags {
                    events.push(t.into_touchinfo(f, time, i as u32));
                }
            }

            if !events.is_empty() {
                unsafe {
                    // If injection fails, we *do not* clear pending/active,
                    // so it will retry next tick.
                    let _ = Pointer::InjectTouchInput(&events);
                }

                // Clear pendings we just attempted; deactivate any that had UP.
                for i in 0..guard.len() {
                    if guard[i].pending.is_some() {
                        guard[i].pending = None;
                    }
                    if emitted_up[i] {
                        guard[i].active = false;
                    }
                }
            }

            drop(guard);
            std::thread::sleep(std::time::Duration::from_millis(16));
        });

        Ok(Self {
            pen_device: unsafe {
                Controls::CreateSyntheticPointerDevice(
                    WindowsAndMessaging::PT_PEN,
                    1,
                    Controls::POINTER_FEEDBACK_DEFAULT,
                )?
            },
            touches,
            last_pressure: 0.0,
        })
    }

    pub(crate) fn move_mouse_abs(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        let (w, h) = self.get_screen_size()?;
        input.Anonymous.mi.dx = (x * 65535) / w;
        input.Anonymous.mi.dy = (y * 65535) / h;
        input.Anonymous.mi.dwFlags =
            KeyboardAndMouse::MOUSEEVENTF_MOVE | KeyboardAndMouse::MOUSEEVENTF_ABSOLUTE;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn move_mouse_rel(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.mi.dx = x;
        input.Anonymous.mi.dy = y;
        input.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_MOVE;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn left_mouse_down(&mut self) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_LEFTDOWN;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn middle_mouse_down(&mut self) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_MIDDLEDOWN;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn right_mouse_down(&mut self) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_RIGHTDOWN;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn left_mouse_up(&mut self) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_LEFTUP;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn middle_mouse_up(&mut self) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_MIDDLEUP;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn right_mouse_up(&mut self) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_RIGHTUP;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn wheel(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        let mut input_vertical = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input_vertical.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_WHEEL;
        input_vertical.Anonymous.mi.mouseData = y as u32;

        let mut input_horizontal = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input_horizontal.Anonymous.mi.dwFlags = KeyboardAndMouse::MOUSEEVENTF_HWHEEL;
        input_horizontal.Anonymous.mi.mouseData = x as u32;

        unsafe {
            KeyboardAndMouse::SendInput(
                &[input_vertical, input_horizontal],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn key_down(&mut self, key: Key) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_KEYBOARD,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.ki.dwFlags = KeyboardAndMouse::KEYEVENTF_SCANCODE;
        input.Anonymous.ki.wScan = key.into();
        unsafe {
            if input.Anonymous.ki.wScan & 0xE000 == 0xE000 {
                input.Anonymous.ki.dwFlags |= KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
            }
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    pub(crate) fn key_up(&mut self, key: Key) -> Result<(), SimulationError> {
        let mut input = KeyboardAndMouse::INPUT {
            r#type: KeyboardAndMouse::INPUT_KEYBOARD,
            Anonymous: unsafe { std::mem::zeroed() },
        };
        input.Anonymous.ki.dwFlags =
            KeyboardAndMouse::KEYEVENTF_KEYUP | KeyboardAndMouse::KEYEVENTF_SCANCODE;
        input.Anonymous.ki.wScan = key.into();
        unsafe {
            if input.Anonymous.ki.wScan & 0xE000 == 0xE000 {
                input.Anonymous.ki.dwFlags |= KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
            }
            KeyboardAndMouse::SendInput(
                &[input],
                std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
            );
        }
        Ok(())
    }

    // -------- TOUCH API (no direct injection here) --------

    pub fn touch_down(&mut self, slot: i32, x: i32, y: i32) -> Result<(), SimulationError> {
        let mut t = self.touches.lock().unwrap();
        let idx = slot as usize;
        t[idx].x = x;
        t[idx].y = y;
        t[idx].active = true;
        t[idx].pending = Some(Pointer::POINTER_FLAG_DOWN);
        Ok(())
    }

    pub fn touch_move(&mut self, slot: i32, x: i32, y: i32) -> Result<(), SimulationError> {
        let mut t = self.touches.lock().unwrap();
        let idx = slot as usize;
        t[idx].x = x;
        t[idx].y = y;
        // Keep contact alive. If the contact was not active yet,
        // mark it active so the worker starts emitting UPDATEs.
        if !t[idx].active {
            t[idx].active = true;
        }
        // No pending flag needed; worker will emit UPDATE each tick while active.
        Ok(())
    }

    pub fn touch_up(&mut self, slot: i32) -> Result<(), SimulationError> {
        let mut t = self.touches.lock().unwrap();
        let idx = slot as usize;
        // Do not clear `active` yet; the worker will emit one UP and then deactivate.
        t[idx].pending = Some(Pointer::POINTER_FLAG_UP);
        Ok(())
    }

    // ------------------ PEN (unchanged) -------------------

    pub(crate) fn pen(
        &mut self,
        x: i32,
        y: i32,
        pressure: f64,
        tilt_x: i32,
        tilt_y: i32,
    ) -> Result<(), SimulationError> {
        let flags = if pressure == 0.0 {
            Pointer::POINTER_FLAG_UP
        } else if self.last_pressure == 0.0 {
            Pointer::POINTER_FLAG_DOWN
                | Pointer::POINTER_FLAG_INRANGE
                | Pointer::POINTER_FLAG_INCONTACT
        } else {
            Pointer::POINTER_FLAG_UPDATE
                | Pointer::POINTER_FLAG_INRANGE
                | Pointer::POINTER_FLAG_INCONTACT
        };

        let mut input: Controls::POINTER_TYPE_INFO = unsafe { std::mem::zeroed() };
        input.r#type = WindowsAndMessaging::PT_PEN;
        input.Anonymous.penInfo.pointerInfo.pointerType = WindowsAndMessaging::PT_PEN;
        input.Anonymous.penInfo.pointerInfo.pointerFlags = flags;
        input.Anonymous.penInfo.penMask = WindowsAndMessaging::PEN_MASK_PRESSURE
            | WindowsAndMessaging::PEN_MASK_TILT_X
            | WindowsAndMessaging::PEN_MASK_TILT_Y;
        input.Anonymous.penInfo.pointerInfo.ptPixelLocation.x = x;
        input.Anonymous.penInfo.pointerInfo.ptPixelLocation.y = y;
        input.Anonymous.penInfo.pressure = (pressure * 1024.0) as u32;
        input.Anonymous.penInfo.tiltX = tilt_x;
        input.Anonymous.penInfo.tiltY = tilt_y;

        self.last_pressure = pressure;

        unsafe {
            Pointer::InjectSyntheticPointerInput(self.pen_device, &[input])?;
        }
        Ok(())
    }

    pub(crate) fn get_screen_size(&self) -> Result<(i32, i32), SimulationError> {
        Ok((
            unsafe {
                WindowsAndMessaging::GetSystemMetrics(WindowsAndMessaging::SM_CXVIRTUALSCREEN)
            },
            unsafe {
                WindowsAndMessaging::GetSystemMetrics(WindowsAndMessaging::SM_CYVIRTUALSCREEN)
            },
        ))
    }
}

impl Drop for PlatformImpl {
    fn drop(&mut self) {
        unsafe {
            Controls::DestroySyntheticPointerDevice(self.pen_device);
        }
    }
}

impl From<Key> for u16 {
    fn from(key: Key) -> Self {
        match key {
            Key::Esc => 0x01,
            Key::Num1 => 0x02,
            Key::Num2 => 0x03,
            Key::Num3 => 0x04,
            Key::Num4 => 0x05,
            Key::Num5 => 0x06,
            Key::Num6 => 0x07,
            Key::Num7 => 0x08,
            Key::Num8 => 0x09,
            Key::Num9 => 0x0A,
            Key::Num0 => 0x0B,
            Key::Minus => 0x0C,
            Key::Equal => 0x0D,
            Key::Backspace => 0x0E,
            Key::Tab => 0x0F,
            Key::Q => 0x10,
            Key::W => 0x11,
            Key::E => 0x12,
            Key::R => 0x13,
            Key::T => 0x14,
            Key::Y => 0x15,
            Key::U => 0x16,
            Key::I => 0x17,
            Key::O => 0x18,
            Key::P => 0x19,
            Key::LeftBrace => 0x1A,
            Key::RightBrace => 0x1B,
            Key::Enter => 0x1C,
            Key::LeftCtrl => 0x1D,
            Key::A => 0x1E,
            Key::S => 0x1F,
            Key::D => 0x20,
            Key::F => 0x21,
            Key::G => 0x22,
            Key::H => 0x23,
            Key::J => 0x24,
            Key::K => 0x25,
            Key::L => 0x26,
            Key::Semicolon => 0x27,
            Key::Apostrophe => 0x28,
            Key::Grave => 0x29,
            Key::LeftShift => 0x2A,
            Key::Backslash => 0x2B,
            Key::Z => 0x2C,
            Key::X => 0x2D,
            Key::C => 0x2E,
            Key::V => 0x2F,
            Key::B => 0x30,
            Key::N => 0x31,
            Key::M => 0x32,
            Key::Comma => 0x33,
            Key::Dot => 0x34,
            Key::Slash => 0x35,
            Key::RightShift => 0x36,
            Key::KpAsterisk => 0x37,
            Key::LeftAlt => 0x38,
            Key::Space => 0x39,
            Key::CapsLock => 0x3A,
            Key::F1 => 0x3B,
            Key::F2 => 0x3C,
            Key::F3 => 0x3D,
            Key::F4 => 0x3E,
            Key::F5 => 0x3F,
            Key::F6 => 0x40,
            Key::F7 => 0x41,
            Key::F8 => 0x42,
            Key::F9 => 0x43,
            Key::F10 => 0x44,
            Key::NumLock => 0x45,
            Key::ScrollLock => 0x46,
            Key::Kp7 => 0x47,
            Key::Kp8 => 0x48,
            Key::Kp9 => 0x49,
            Key::KpMinus => 0x4A,
            Key::Kp4 => 0x4B,
            Key::Kp5 => 0x4C,
            Key::Kp6 => 0x4D,
            Key::KpPlus => 0x4E,
            Key::Kp1 => 0x4F,
            Key::Kp2 => 0x50,
            Key::Kp3 => 0x51,
            Key::Kp0 => 0x52,
            Key::KpDot => 0x53,
            Key::ZenkakuHankaku => 0x56,
            Key::IntlBackslash => 0x64,
            Key::F11 => 0x57,
            Key::F12 => 0x58,
            Key::Ro => 0x59,
            Key::Katakana => 0x5A,
            Key::Hiragana => 0x5B,
            Key::Henkan => 0x5C,
            Key::KatakanaHiragana => 0x5D,
            Key::Muhenkan => 0x5E,
            Key::KpJpComma => 0x5F,
            Key::KpEnter => 0x60,
            Key::RightCtrl => 0x61,
            Key::KpSlash => 0x62,
            Key::SysRq => 0x63,
            Key::RightAlt => 0x64,
            Key::Home => 0xE047,
            Key::Up => 0xE048,
            Key::PageUp => 0xE049,
            Key::Left => 0xE04B,
            Key::Right => 0xE04D,
            Key::End => 0xE04F,
            Key::Down => 0xE050,
            Key::PageDown => 0xE051,
            Key::Insert => 0xE052,
            Key::Delete => 0xE9,
            Key::Macro => 0xEA,
            Key::Mute => 0xEB,
            Key::VolumeDown => 0xEC,
            Key::VolumeUp => 0xED,
            Key::Power => 0xEE,
            Key::KpEqual => 0xEF,
            Key::KpPlusMinus => 0xF0,
            Key::Pause => 0xF1,
            Key::KpComma => 0xF2,
            Key::Hanguel => 0xF3,
            Key::Hanja => 0xF4,
            Key::Yen => 0xF5,
            Key::LeftMeta => 0xE05B,
            Key::RightMeta => 0xE05C,
            Key::Compose => 0xF8,
            Key::Stop => 0xF9,
            Key::Help => 0xFA,
            Key::Calc => 0xFB,
            Key::Sleep => 0xFC,
            Key::WakeUp => 0xFD,
            Key::Mail => 0xFE,
            Key::Bookmarks => 0xFF,
            Key::Computer => 0x100,
            Key::Back => 0x101,
            Key::Forward => 0x102,
            Key::NextSong => 0x103,
            Key::PlayPause => 0x104,
            Key::PreviousSong => 0x105,
            Key::StopCD => 0x106,
            Key::Homepage => 0x107,
            Key::Refresh => 0x108,
            Key::F13 => 0x109,
            Key::F14 => 0x10A,
            Key::F15 => 0x10B,
            Key::F23 => 0x10C,
            Key::Camera => 0x10D,
            Key::Search => 0x10E,
            Key::BrightnessDown => 0x10F,
            Key::BrightnessUp => 0x110,
            Key::Media => 0x111,
            Key::SwitchVideoMode => 0x112,
            Key::Battery => 0x113,
            Key::Wlan => 0x114,
            Key::Dvd => 0x115,
            Key::FnEsc => 0x116,
            _ => 0xFF, // Unknown key
        }
    }
}
