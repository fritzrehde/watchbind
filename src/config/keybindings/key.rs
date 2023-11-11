use anyhow::{bail, Context, Error, Result};
use crossterm::event::{
    KeyCode as CrosstermKeyCode, KeyEvent as CrosstermKeyEvent,
    KeyModifiers as CrosstermKeyModifiers,
};
use itertools::Itertools;
use parse_display::{Display, FromStr};
use std::{fmt, str};
use strum::{EnumIter, EnumMessage, EnumProperty, IntoEnumIterator};

/// The specific combinations of modifiers and key codes that we allow/handle.
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
pub struct KeyEvent {
    modifier: KeyModifier,
    code: KeyCode,
}

#[derive(
    Debug,
    // For using as key in hashmap
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Clone,
    // For displaying and parsing
    Display,
    FromStr,
    // For displaying all possible variants
    EnumIter,
    EnumMessage,
    EnumProperty,
)]
#[display(style = "lowercase")]
pub enum KeyModifier {
    Alt,
    Ctrl,

    #[from_str(ignore)]
    #[strum(props(Hidden = "true"))]
    Shift,

    #[from_str(ignore)]
    #[strum(props(Hidden = "true"))]
    None,
}

#[derive(
    Debug,
    // For using as key in hashmap
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Clone,
    // For displaying and parsing
    Display,
    FromStr,
    // For displaying all possible variants
    EnumIter,
    EnumMessage,
    EnumProperty,
)]
#[display(style = "lowercase")]
pub enum KeyCode {
    Esc,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    BackTab,
    Backspace,
    Delete,
    Insert,
    Tab,
    Space,

    #[display("{0}")]
    #[strum(message = "<lowercase char>, <uppercase char>")]
    Char(char),

    // Parse only values 1 to 12
    #[from_str(regex = "f(?<0>[1-9]|1[0-2])")]
    #[display("f{0}")]
    #[strum(message = "f<1-12>")]
    F(u8),
}

impl str::FromStr for KeyEvent {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (code, modifier) = match s.split_once('+') {
            Some((modifier, code)) => (
                code.parse()
                    .with_context(|| format!("Invalid KeyCode: {}", code))?,
                modifier
                    .parse()
                    .with_context(|| format!("Invalid KeyModifier: {}", modifier))?,
            ),
            None => (
                s.parse()
                    .with_context(|| format!("Invalid KeyCode: {}", s))?,
                KeyModifier::None,
            ),
        };
        Ok(Self { modifier, code })
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.modifier {
            KeyModifier::None => write!(f, "{}", self.code)?,
            _ => write!(f, "{}+{}", self.modifier, self.code)?,
        };
        Ok(())
    }
}

impl TryFrom<CrosstermKeyEvent> for KeyEvent {
    type Error = Error;
    fn try_from(key: CrosstermKeyEvent) -> std::result::Result<Self, Self::Error> {
        let code = key.code.try_into()?;
        let mut modifier = key.modifiers.try_into()?;

        // We never internally save our modifier as Shift, because we don't
        // want the user to have to specify e.g. "shift+G" instead of just "G".
        // Therefore, we remove the Shift modifier if the code is uppercase
        // anyways.
        if let KeyCode::Char(char) = code {
            if char.is_uppercase() && modifier == KeyModifier::Shift {
                modifier = KeyModifier::None;
            }
        };

        Ok(Self { modifier, code })
    }
}

impl TryFrom<CrosstermKeyModifiers> for KeyModifier {
    type Error = Error;
    fn try_from(value: CrosstermKeyModifiers) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            CrosstermKeyModifiers::ALT => Self::Alt,
            CrosstermKeyModifiers::CONTROL => Self::Ctrl,
            CrosstermKeyModifiers::SHIFT => Self::Shift,
            CrosstermKeyModifiers::NONE => Self::None,
            // TODO: shouldn't use debug output for display output
            _ => bail!("Invalid modifier key: {:?}", value),
        })
    }
}

impl TryFrom<CrosstermKeyCode> for KeyCode {
    type Error = Error;
    fn try_from(value: CrosstermKeyCode) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            CrosstermKeyCode::Esc => KeyCode::Esc,
            CrosstermKeyCode::Enter => KeyCode::Enter,
            CrosstermKeyCode::Left => KeyCode::Left,
            CrosstermKeyCode::Right => KeyCode::Right,
            CrosstermKeyCode::Up => KeyCode::Up,
            CrosstermKeyCode::Down => KeyCode::Down,
            CrosstermKeyCode::Home => KeyCode::Home,
            CrosstermKeyCode::End => KeyCode::End,
            CrosstermKeyCode::PageUp => KeyCode::PageUp,
            CrosstermKeyCode::PageDown => KeyCode::PageDown,
            CrosstermKeyCode::BackTab => KeyCode::BackTab,
            CrosstermKeyCode::Backspace => KeyCode::Backspace,
            CrosstermKeyCode::Delete => KeyCode::Delete,
            CrosstermKeyCode::Insert => KeyCode::Insert,
            CrosstermKeyCode::F(c) => KeyCode::F(c),
            CrosstermKeyCode::Tab => KeyCode::Tab,
            CrosstermKeyCode::Char(' ') => KeyCode::Space,
            CrosstermKeyCode::Char(c) => KeyCode::Char(c),
            // TODO: shouldn't use debug output for display output
            _ => bail!("Invalid key code: {:?}", value),
        })
    }
}

/// Get string list of all possible values of `T`.
fn get_possible_values<T>() -> String
where
    T: IntoEnumIterator + EnumMessage + EnumProperty + fmt::Display,
{
    T::iter()
        // TODO: replace with strum's get_bool once available
        // Hide variants configured to be hidden.
        .filter(|variant| !matches!(variant.get_str("Hidden"), Some("true")))
        // Use strum's `message` if available, otherwise use `to_string`.
        .map(|variant| {
            variant
                .get_message()
                .map(str::to_owned)
                .unwrap_or_else(|| variant.to_string())
        })
        .join(", ")
}

impl KeyEvent {
    /// Get string help menu of all possible values of `KeyCode` and
    /// `KeyModifier`.
    pub fn all_possible_values() -> (String, String) {
        (
            get_possible_values::<KeyCode>(),
            get_possible_values::<KeyModifier>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_eq_parse_display<T>(input_str: &str, expected: T)
    where
        T: str::FromStr + fmt::Display + PartialEq + fmt::Debug,
        <T as str::FromStr>::Err: fmt::Debug,
    {
        // Test FromStr
        assert_eq!(
            expected,
            // TODO: non-ideal unwrap here
            input_str.parse().unwrap(),
            "Expected the input string '{}' to be parsed into {:?}",
            input_str,
            expected
        );

        // Test Display
        assert_eq!(
            input_str,
            expected.to_string(),
            "Expected the expected {:?} to be displayed as string '{}'",
            expected,
            input_str
        );
    }

    #[test]
    fn test_valid_function_keys() -> Result<()> {
        assert_eq_parse_display("f1", KeyCode::F(1));
        assert_eq_parse_display("f12", KeyCode::F(12));
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_invalid_function_keys() {
        let _: KeyCode = "f0".parse().unwrap();
        let _: KeyCode = "f13".parse().unwrap();
    }

    #[test]
    fn test_valid_modifiers() {
        assert_eq_parse_display(
            "c",
            KeyEvent {
                modifier: KeyModifier::None,
                code: KeyCode::Char('c'),
            },
        );

        assert_eq_parse_display(
            "alt+P",
            KeyEvent {
                modifier: KeyModifier::Alt,
                code: KeyCode::Char('P'),
            },
        );

        assert_eq_parse_display(
            "ctrl+c",
            KeyEvent {
                modifier: KeyModifier::Ctrl,
                code: KeyCode::Char('c'),
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_modifiers() {
        let _: KeyModifier = "none".parse().unwrap();
        let _: KeyModifier = "shift".parse().unwrap();
        let _: KeyModifier = "super".parse().unwrap();
        let _: KeyModifier = "alt+ctrl".parse().unwrap();
    }
}
