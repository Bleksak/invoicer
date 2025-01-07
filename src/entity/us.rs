use std::{error::Error, fmt::Display, str::FromStr};

pub enum RegistrationNumber {
    Tin(Tin),
    Ein(Ein),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TinError {
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EinError {
    Invalid,
}


impl Display for TinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TinError::Invalid => write!(f, "Nesprávné TIN"),
        }
    }
}

impl Display for EinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EinError::Invalid => write!(f, "Nesprávné EIN"),
        }
    }
}

impl Error for TinError {}
impl Error for EinError {}

pub struct Tin(String);
impl FromStr for Tin {
    type Err = TinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 11 {
            return Err(TinError::Invalid);
        }

        let mut split = s.splitn(3, "-");
        
        if !split.all(|chunk|{
            chunk.chars().all(|c| c.is_ascii_digit())
        }){
            return Err(TinError::Invalid);
        }
        Ok(Tin(s.into()))
        
    }
}

pub struct Ein(String);
impl FromStr for Ein {
    type Err = EinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 10 {
            return Err(EinError::Invalid);
        }

        let mut split = s.splitn(2, "-");
        
        if !split.all(|chunk|{
            chunk.chars().all(|c| c.is_ascii_digit())
        }){
            return Err(EinError::Invalid);
        }
        Ok(Ein(s.into()))
        
    }
}

impl Tin {
    pub fn get(&self)->&str{
        &self.0
    }
}

impl Ein {
    pub fn get(&self)->&str{
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tin() {
        let tin: Result<Tin, _> = "123-45-6789".parse();
        assert!(tin.is_ok());
        assert_eq!(tin.unwrap().get(), "123-45-6789");
    }

    #[test]
    fn test_invalid_tin() {
        let invalid_tins = [
            "123-45-678",     // Too short
            "1234-56-7890",   // Incorrect segment length
            "123-45-678A",    // Non-digit character
            "1234567890",     // Missing dashes
            "123--45-6789",   // Extra dash
        ];

        for tin in invalid_tins {
            assert!(tin.parse::<Tin>().is_err());
        }
    }

    #[test]
    fn test_valid_ein() {
        let ein: Result<Ein, _> = "12-3456789".parse();
        assert!(ein.is_ok());
        assert_eq!(ein.unwrap().get(), "12-3456789");
    }

    #[test]
    fn test_invalid_ein() {
        let invalid_eins = [
            "12-345678",      // Too short
            "123-456789",     // Incorrect segment length
            "12-345678A",     // Non-digit character
            "123456789",      // Missing dash
            "12--3456789",    // Extra dash
        ];

        for ein in invalid_eins {
            assert!(ein.parse::<Ein>().is_err());
        }
    }
}