use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub control: bool,
    pub option: bool,
    pub cmd: bool,
    pub shift: bool,
}

impl KeyModifiers {
    pub fn has_shortcut_modifier(self) -> bool {
        self.control || self.option || self.cmd
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawKeyEvent {
    pub key_label: String,
    pub modifiers: KeyModifiers,
}

/// Formats a captured key event into a HUD label while enforcing macOS Secure Input privacy.
/// Returns `None` whenever Secure Input (password fields, 1Password, Terminal secure entry) is on.
pub fn filter_keystroke(
    event: &RawKeyEvent,
    secure_input_active: bool,
    shortcuts_only: bool,
) -> Option<String> {
    if secure_input_active {
        return None;
    }
    let key = event.key_label.trim();
    if key.is_empty() {
        return None;
    }
    if shortcuts_only && !event.modifiers.has_shortcut_modifier() {
        return None;
    }

    let mut out = String::new();
    if event.modifiers.control {
        out.push('⌃');
    }
    if event.modifiers.option {
        out.push('⌥');
    }
    if event.modifiers.cmd {
        out.push('⌘');
    }
    if event.modifiers.shift {
        out.push('⇧');
    }
    if key.len() == 1 {
        out.push_str(&key.to_ascii_uppercase());
    } else {
        out.push_str(key);
    }
    Some(out)
}

pub fn macos_keycode_label(keycode: u16) -> Option<&'static str> {
    let label = match keycode {
        0 => "A",
        1 => "S",
        2 => "D",
        3 => "F",
        4 => "H",
        5 => "G",
        6 => "Z",
        7 => "X",
        8 => "C",
        9 => "V",
        11 => "B",
        12 => "Q",
        13 => "W",
        14 => "E",
        15 => "R",
        16 => "Y",
        17 => "T",
        18 => "1",
        19 => "2",
        20 => "3",
        21 => "4",
        22 => "6",
        23 => "5",
        25 => "9",
        26 => "7",
        28 => "8",
        29 => "0",
        31 => "O",
        32 => "U",
        34 => "I",
        35 => "P",
        36 => "Return",
        37 => "L",
        38 => "J",
        40 => "K",
        45 => "N",
        46 => "M",
        48 => "Tab",
        49 => "Space",
        51 => "Delete",
        53 => "Esc",
        123 => "←",
        124 => "→",
        125 => "↓",
        126 => "↑",
        _ => return None,
    };
    Some(label)
}
