use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_more::From;
use std::str::FromStr;

#[cfg_attr(test, derive(Debug))]
#[derive(Hash, Eq, PartialEq, From, Clone)]
pub struct Key(KeyEvent);

impl FromStr for Key {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let event = match s.split_once('+') {
            Some((s1, s2)) => {
                let mut event = parse_code(s2)?;
                event.modifiers.insert(parse_modifier(s1)?);
                event
            }
            None => parse_code(s)?,
        };
        Ok(Key(event))
    }
}

fn parse_modifier(s: &str) -> Result<KeyModifiers> {
    Ok(match s {
        "alt" => KeyModifiers::ALT,
        "ctrl" => KeyModifiers::CONTROL,
        invalid => bail!("Invalid key modifier provided in keybinding: {}", invalid),
    })
}

fn parse_code(s: &str) -> Result<KeyEvent> {
    let code = match s {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "del" => KeyCode::Delete,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "ins" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
        invalid => bail!("Invalid key code provided in keybinding: {}", invalid),
    };
    Ok(KeyEvent::from(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lowercase_key() -> Result<()> {
        assert_eq!("k".parse::<Key>()?, Key(KeyCode::Char('k').into()));
        Ok(())
    }

    #[test]
    fn test_parse_uppercase_key() -> Result<()> {
        assert_eq!("G".parse::<Key>()?, Key(KeyCode::Char('G').into()));
        Ok(())
    }

    #[test]
    fn test_parse_ctrl_modifier() -> Result<()> {
        assert_eq!(
            "ctrl+c".parse::<Key>()?,
            Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
        );
        // TODO: passes test, but doesn't practically work in all terminals
        assert_eq!(
            "ctrl+S".parse::<Key>()?,
            Key(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::CONTROL))
        );
        Ok(())
    }

    #[test]
    fn test_parse_alt_modifier() -> Result<()> {
        assert_eq!(
            "alt+z".parse::<Key>()?,
            Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::ALT))
        );
        Ok(())
    }

    #[test]
    fn test_parse_invalid_modifiers() {
        assert!("shift+a".parse::<Key>().is_err());
        assert!("super+a".parse::<Key>().is_err());
        assert!("alt+shift+a".parse::<Key>().is_err());
        assert!("alt+ctrl+a".parse::<Key>().is_err());
    }
}
