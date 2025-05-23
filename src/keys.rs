use strum_macros::EnumIter;

/// A physical keyboard key.
///
/// These keys represent __physical__ keyboard keys, which
/// means that each variant only represents a certain
/// location on the keyboard. In other terms, they are
/// layout independent. For example, simulating a press of
/// [Key::Q] will trigger the key located where the 'Q' key
/// appears on a standard QWERTY layout, even if the active
/// keyboard layout maps that physical key to a different
/// character (such as 'A' in an AZERTY layout).
///
/// This makes [`Key`] suitable for applications that need
/// to work based on the physical key positions rather than
/// the characters they produce, such as games or custom
/// input handling.
#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Esc,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equal,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBrace,
    RightBrace,
    Enter,
    LeftCtrl,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Apostrophe,
    Grave,
    LeftShift,
    Backslash,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Dot,
    Slash,
    RightShift,
    KpAsterisk,
    LeftAlt,
    Space,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    NumLock,
    ScrollLock,
    Kp7,
    Kp8,
    Kp9,
    KpMinus,
    Kp4,
    Kp5,
    Kp6,
    KpPlus,
    Kp1,
    Kp2,
    Kp3,
    Kp0,
    KpDot,
    ZenkakuHankaku,
    IntlBackslash, // 102ND key (ISO keyboards)
    F11,
    F12,
    Ro,
    Katakana,
    Hiragana,
    Henkan,
    KatakanaHiragana,
    Muhenkan,
    KpJpComma,
    KpEnter,
    RightCtrl,
    KpSlash,
    SysRq,
    RightAlt,
    Home,
    Up,
    PageUp,
    Left,
    Right,
    End,
    Down,
    PageDown,
    Insert,
    Delete,
    Macro,
    Mute,
    VolumeDown,
    VolumeUp,
    Power,
    KpEqual,
    KpPlusMinus,
    Pause,
    KpComma,
    Hanguel,
    Hanja,
    Yen,
    /// Corresponds to Command on macOS
    LeftMeta,
    RightMeta,
    Compose,
    Stop,
    Help,
    Calc,
    Sleep,
    WakeUp,
    ScreenLock,
    Mail,
    Bookmarks,
    Computer,
    Back,
    Forward,
    NextSong,
    PlayPause,
    PreviousSong,
    StopCD,
    Homepage,
    Refresh,
    F13,
    F14,
    F15,
    F23,
    Camera,
    Search,
    BrightnessDown,
    BrightnessUp,
    Media,
    SwitchVideoMode,
    Battery,
    Wlan,
    BrightnessZero,
    Dvd,
    FnEsc,
}
