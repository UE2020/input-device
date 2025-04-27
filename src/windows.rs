use thiserror::Error;
use windows::Win32::UI::Input::KeyboardAndMouse;
use windows::Win32::UI::WindowsAndMessaging;

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Windows error: {0}")]
    WindowsError(#[from] windows::core::Error),
}

pub(crate) struct PlatformImpl;

impl PlatformImpl {
    pub(crate) fn new() -> Result<Self, SimulationError> {
        Ok(Self)
    }

    pub(crate) fn move_mouse_abs(&mut self, x: i32, y: i32) -> Result<(), SimulationError> {
        Ok(unsafe { WindowsAndMessaging::SetCursorPos(x, y)? })
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
