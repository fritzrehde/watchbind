use anyhow::{bail, Context, Error, Result};
use crossterm::event::{KeyCode as CKeyCode, KeyEvent as CKeyEvent, KeyModifiers as CKeyModifiers};
use derive_more::From;
use parse_display::{Display, FromStr};
use std::{fmt, str};

/// The specific combinations of modifiers and key codes that we allow/handle.
#[derive(Hash, Eq, PartialEq, From, Clone, Debug)]
pub struct KeyEvent {
    modifier: KeyModifier,
    code: KeyCode,
}

#[derive(Hash, Eq, PartialEq, From, Clone, Debug, Display, FromStr)]
#[display(style = "lowercase")]
enum KeyModifier {
    Alt,
    Ctrl,
    #[from_str(ignore)]
    Shift,
    #[from_str(ignore)]
    None,
}

#[derive(Hash, Eq, PartialEq, From, Clone, Debug, Display, FromStr)]
#[display(style = "lowercase")]
enum KeyCode {
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
    Char(char),

    // Parse only values 1 to 12
    #[from_str(regex = "f(?<0>[1-9]|1[0-2])")]
    #[display("f{0}")]
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
        if self.modifier == KeyModifier::None {
            write!(f, "{}", self.code)?;
        } else {
            write!(f, "{}+{}", self.modifier, self.code)?;
        }
        Ok(())
    }
}

impl TryFrom<CKeyEvent> for KeyEvent {
    type Error = Error;
    fn try_from(key: CKeyEvent) -> std::result::Result<Self, Self::Error> {
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

impl TryFrom<CKeyModifiers> for KeyModifier {
    type Error = Error;
    fn try_from(value: CKeyModifiers) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            CKeyModifiers::ALT => Self::Alt,
            CKeyModifiers::CONTROL => Self::Ctrl,
            CKeyModifiers::SHIFT => Self::Shift,
            CKeyModifiers::NONE => Self::None,
            // TODO: shouldn't use debug output for display output
            _ => bail!("Invalid modifier key: {:?}", value),
        })
    }
}

impl TryFrom<CKeyCode> for KeyCode {
    type Error = Error;
    fn try_from(value: CKeyCode) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            CKeyCode::Esc => KeyCode::Esc,
            CKeyCode::Enter => KeyCode::Enter,
            CKeyCode::Left => KeyCode::Left,
            CKeyCode::Right => KeyCode::Right,
            CKeyCode::Up => KeyCode::Up,
            CKeyCode::Down => KeyCode::Down,
            CKeyCode::Home => KeyCode::Home,
            CKeyCode::End => KeyCode::End,
            CKeyCode::PageUp => KeyCode::PageUp,
            CKeyCode::PageDown => KeyCode::PageDown,
            CKeyCode::BackTab => KeyCode::BackTab,
            CKeyCode::Backspace => KeyCode::Backspace,
            CKeyCode::Delete => KeyCode::Delete,
            CKeyCode::Insert => KeyCode::Insert,
            CKeyCode::F(c) => KeyCode::F(c),
            CKeyCode::Tab => KeyCode::Tab,
            CKeyCode::Char(' ') => KeyCode::Space,
            CKeyCode::Char(c) => KeyCode::Char(c),
            // TODO: shouldn't use debug output for display output
            _ => bail!("Invalid key code: {:?}", value),
        })
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
