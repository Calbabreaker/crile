// Mostly copied from winit

/// Represents whether a key or mouse button is pressed or released
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl ButtonState {
    pub fn is_pressed(&self) -> bool {
        match self {
            ButtonState::Pressed => true,
            ButtonState::Released => false,
        }
    }

    pub(crate) fn from_winit(code: winit::event::ElementState) -> ButtonState {
        use winit::event::ElementState;
        match code {
            ElementState::Pressed => ButtonState::Pressed,
            ElementState::Released => ButtonState::Released,
        }
    }
}

/// Represents the mouse button code on a mouse
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, strum::EnumString)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other,
}

impl MouseButton {
    pub(crate) fn from_winit(code: winit::event::MouseButton) -> MouseButton {
        use winit::event::MouseButton as WMB;
        match code {
            WMB::Left => MouseButton::Left,
            WMB::Right => MouseButton::Right,
            WMB::Middle => MouseButton::Middle,
            WMB::Back => MouseButton::Back,
            WMB::Forward => MouseButton::Forward,
            WMB::Other(_) => MouseButton::Other,
        }
    }
}

/// Represents the modifiers keys being pressed (left and right)
#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct KeyModifiers {
    pub alt_key: bool,
    pub control_key: bool,
    pub shift_key: bool,
    pub super_key: bool,
}

impl KeyModifiers {
    // The supper/command key on mac and control key on other OS
    pub fn command_key(&self) -> bool {
        if cfg!(target_os = "macos") {
            self.super_key
        } else {
            self.control_key
        }
    }

    pub(crate) fn from_winit(modifiers: winit::keyboard::ModifiersState) -> KeyModifiers {
        KeyModifiers {
            alt_key: modifiers.alt_key(),
            control_key: modifiers.control_key(),
            shift_key: modifiers.shift_key(),
            super_key: modifiers.super_key(),
        }
    }
}

/// Represents the physical key on the keyboard not accounting for keyboard layouts
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, strum::EnumString)]
pub enum KeyCode {
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    SuperLeft,
    SuperRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Convert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    Escape,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Abort,
    Resume,
    Suspend,
    Again,
    Copy,
    Cut,
    Find,
    Open,
    Paste,
    Props,
    Select,
    Undo,
    Hiragana,
    Katakana,
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
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
    Unknown,
}

impl KeyCode {
    pub(crate) fn from_winit(keycode: winit::keyboard::KeyCode) -> KeyCode {
        use winit::keyboard::KeyCode as WKC;
        match keycode {
            WKC::Backquote => KeyCode::Backquote,
            WKC::Backslash => KeyCode::Backslash,
            WKC::BracketLeft => KeyCode::BracketLeft,
            WKC::BracketRight => KeyCode::BracketRight,
            WKC::Comma => KeyCode::Comma,
            WKC::Digit0 => KeyCode::Digit0,
            WKC::Digit1 => KeyCode::Digit1,
            WKC::Digit2 => KeyCode::Digit2,
            WKC::Digit3 => KeyCode::Digit3,
            WKC::Digit4 => KeyCode::Digit4,
            WKC::Digit5 => KeyCode::Digit5,
            WKC::Digit6 => KeyCode::Digit6,
            WKC::Digit7 => KeyCode::Digit7,
            WKC::Digit8 => KeyCode::Digit8,
            WKC::Digit9 => KeyCode::Digit9,
            WKC::Equal => KeyCode::Equal,
            WKC::IntlBackslash => KeyCode::IntlBackslash,
            WKC::IntlRo => KeyCode::IntlRo,
            WKC::IntlYen => KeyCode::IntlYen,
            WKC::KeyA => KeyCode::KeyA,
            WKC::KeyB => KeyCode::KeyB,
            WKC::KeyC => KeyCode::KeyC,
            WKC::KeyD => KeyCode::KeyD,
            WKC::KeyE => KeyCode::KeyE,
            WKC::KeyF => KeyCode::KeyF,
            WKC::KeyG => KeyCode::KeyG,
            WKC::KeyH => KeyCode::KeyH,
            WKC::KeyI => KeyCode::KeyI,
            WKC::KeyJ => KeyCode::KeyJ,
            WKC::KeyK => KeyCode::KeyK,
            WKC::KeyL => KeyCode::KeyL,
            WKC::KeyM => KeyCode::KeyM,
            WKC::KeyN => KeyCode::KeyN,
            WKC::KeyO => KeyCode::KeyO,
            WKC::KeyP => KeyCode::KeyP,
            WKC::KeyQ => KeyCode::KeyQ,
            WKC::KeyR => KeyCode::KeyR,
            WKC::KeyS => KeyCode::KeyS,
            WKC::KeyT => KeyCode::KeyT,
            WKC::KeyU => KeyCode::KeyU,
            WKC::KeyV => KeyCode::KeyV,
            WKC::KeyW => KeyCode::KeyW,
            WKC::KeyX => KeyCode::KeyX,
            WKC::KeyY => KeyCode::KeyY,
            WKC::KeyZ => KeyCode::KeyZ,
            WKC::Minus => KeyCode::Minus,
            WKC::Period => KeyCode::Period,
            WKC::Quote => KeyCode::Quote,
            WKC::Semicolon => KeyCode::Semicolon,
            WKC::Slash => KeyCode::Slash,
            WKC::AltLeft => KeyCode::AltLeft,
            WKC::AltRight => KeyCode::AltRight,
            WKC::Backspace => KeyCode::Backspace,
            WKC::CapsLock => KeyCode::CapsLock,
            WKC::ContextMenu => KeyCode::ContextMenu,
            WKC::ControlLeft => KeyCode::ControlLeft,
            WKC::ControlRight => KeyCode::ControlRight,
            WKC::Enter => KeyCode::Enter,
            WKC::SuperLeft => KeyCode::SuperLeft,
            WKC::SuperRight => KeyCode::SuperRight,
            WKC::ShiftLeft => KeyCode::ShiftLeft,
            WKC::ShiftRight => KeyCode::ShiftRight,
            WKC::Space => KeyCode::Space,
            WKC::Tab => KeyCode::Tab,
            WKC::Convert => KeyCode::Convert,
            WKC::KanaMode => KeyCode::KanaMode,
            WKC::Lang1 => KeyCode::Lang1,
            WKC::Lang2 => KeyCode::Lang2,
            WKC::Lang3 => KeyCode::Lang3,
            WKC::Lang4 => KeyCode::Lang4,
            WKC::Lang5 => KeyCode::Lang5,
            WKC::NonConvert => KeyCode::NonConvert,
            WKC::Delete => KeyCode::Delete,
            WKC::End => KeyCode::End,
            WKC::Help => KeyCode::Help,
            WKC::Home => KeyCode::Home,
            WKC::Insert => KeyCode::Insert,
            WKC::PageDown => KeyCode::PageDown,
            WKC::PageUp => KeyCode::PageUp,
            WKC::ArrowDown => KeyCode::ArrowDown,
            WKC::ArrowLeft => KeyCode::ArrowLeft,
            WKC::ArrowRight => KeyCode::ArrowRight,
            WKC::ArrowUp => KeyCode::ArrowUp,
            WKC::NumLock => KeyCode::NumLock,
            WKC::Numpad0 => KeyCode::Numpad0,
            WKC::Numpad1 => KeyCode::Numpad1,
            WKC::Numpad2 => KeyCode::Numpad2,
            WKC::Numpad3 => KeyCode::Numpad3,
            WKC::Numpad4 => KeyCode::Numpad4,
            WKC::Numpad5 => KeyCode::Numpad5,
            WKC::Numpad6 => KeyCode::Numpad6,
            WKC::Numpad7 => KeyCode::Numpad7,
            WKC::Numpad8 => KeyCode::Numpad8,
            WKC::Numpad9 => KeyCode::Numpad9,
            WKC::NumpadAdd => KeyCode::NumpadAdd,
            WKC::NumpadBackspace => KeyCode::NumpadBackspace,
            WKC::NumpadClear => KeyCode::NumpadClear,
            WKC::NumpadClearEntry => KeyCode::NumpadClearEntry,
            WKC::NumpadComma => KeyCode::NumpadComma,
            WKC::NumpadDecimal => KeyCode::NumpadDecimal,
            WKC::NumpadDivide => KeyCode::NumpadDivide,
            WKC::NumpadEnter => KeyCode::NumpadEnter,
            WKC::NumpadEqual => KeyCode::NumpadEqual,
            WKC::NumpadHash => KeyCode::NumpadHash,
            WKC::NumpadMemoryAdd => KeyCode::NumpadMemoryAdd,
            WKC::NumpadMemoryClear => KeyCode::NumpadMemoryClear,
            WKC::NumpadMemoryRecall => KeyCode::NumpadMemoryRecall,
            WKC::NumpadMemoryStore => KeyCode::NumpadMemoryStore,
            WKC::NumpadMemorySubtract => KeyCode::NumpadMemorySubtract,
            WKC::NumpadMultiply => KeyCode::NumpadMultiply,
            WKC::NumpadParenLeft => KeyCode::NumpadParenLeft,
            WKC::NumpadParenRight => KeyCode::NumpadParenRight,
            WKC::NumpadStar => KeyCode::NumpadStar,
            WKC::NumpadSubtract => KeyCode::NumpadSubtract,
            WKC::Escape => KeyCode::Escape,
            WKC::Fn => KeyCode::Fn,
            WKC::FnLock => KeyCode::FnLock,
            WKC::PrintScreen => KeyCode::PrintScreen,
            WKC::ScrollLock => KeyCode::ScrollLock,
            WKC::Pause => KeyCode::Pause,
            WKC::BrowserBack => KeyCode::BrowserBack,
            WKC::BrowserFavorites => KeyCode::BrowserFavorites,
            WKC::BrowserForward => KeyCode::BrowserForward,
            WKC::BrowserHome => KeyCode::BrowserHome,
            WKC::BrowserRefresh => KeyCode::BrowserRefresh,
            WKC::BrowserSearch => KeyCode::BrowserSearch,
            WKC::BrowserStop => KeyCode::BrowserStop,
            WKC::Eject => KeyCode::Eject,
            WKC::LaunchApp1 => KeyCode::LaunchApp1,
            WKC::LaunchApp2 => KeyCode::LaunchApp2,
            WKC::LaunchMail => KeyCode::LaunchMail,
            WKC::MediaPlayPause => KeyCode::MediaPlayPause,
            WKC::MediaSelect => KeyCode::MediaSelect,
            WKC::MediaStop => KeyCode::MediaStop,
            WKC::MediaTrackNext => KeyCode::MediaTrackNext,
            WKC::MediaTrackPrevious => KeyCode::MediaTrackPrevious,
            WKC::Power => KeyCode::Power,
            WKC::Sleep => KeyCode::Sleep,
            WKC::AudioVolumeDown => KeyCode::AudioVolumeDown,
            WKC::AudioVolumeMute => KeyCode::AudioVolumeMute,
            WKC::AudioVolumeUp => KeyCode::AudioVolumeUp,
            WKC::WakeUp => KeyCode::WakeUp,
            WKC::Abort => KeyCode::Abort,
            WKC::Resume => KeyCode::Resume,
            WKC::Suspend => KeyCode::Suspend,
            WKC::Again => KeyCode::Again,
            WKC::Copy => KeyCode::Copy,
            WKC::Cut => KeyCode::Cut,
            WKC::Find => KeyCode::Find,
            WKC::Open => KeyCode::Open,
            WKC::Paste => KeyCode::Paste,
            WKC::Props => KeyCode::Props,
            WKC::Select => KeyCode::Select,
            WKC::Undo => KeyCode::Undo,
            WKC::Hiragana => KeyCode::Hiragana,
            WKC::Katakana => KeyCode::Katakana,
            WKC::F1 => KeyCode::F1,
            WKC::F2 => KeyCode::F2,
            WKC::F3 => KeyCode::F3,
            WKC::F4 => KeyCode::F4,
            WKC::F5 => KeyCode::F5,
            WKC::F6 => KeyCode::F6,
            WKC::F7 => KeyCode::F7,
            WKC::F8 => KeyCode::F8,
            WKC::F9 => KeyCode::F9,
            WKC::F10 => KeyCode::F10,
            WKC::F11 => KeyCode::F11,
            WKC::F12 => KeyCode::F12,
            WKC::F13 => KeyCode::F13,
            WKC::F14 => KeyCode::F14,
            WKC::F15 => KeyCode::F15,
            WKC::F16 => KeyCode::F16,
            WKC::F17 => KeyCode::F17,
            WKC::F18 => KeyCode::F18,
            WKC::F19 => KeyCode::F19,
            WKC::F20 => KeyCode::F20,
            WKC::F21 => KeyCode::F21,
            WKC::F22 => KeyCode::F22,
            WKC::F23 => KeyCode::F23,
            WKC::F24 => KeyCode::F24,
            WKC::F25 => KeyCode::F25,
            WKC::F26 => KeyCode::F26,
            WKC::F27 => KeyCode::F27,
            WKC::F28 => KeyCode::F28,
            WKC::F29 => KeyCode::F29,
            WKC::F30 => KeyCode::F30,
            WKC::F31 => KeyCode::F31,
            WKC::F32 => KeyCode::F32,
            WKC::F33 => KeyCode::F33,
            WKC::F34 => KeyCode::F34,
            WKC::F35 => KeyCode::F35,
            WKC::Meta => KeyCode::SuperLeft,
            _ => KeyCode::Unknown,
        }
    }
}
