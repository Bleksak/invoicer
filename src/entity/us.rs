use std::fmt::Display;
use std::str::FromStr;

pub enum Vat {
    Tin(Tin),
    Ein(Ein),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    Invalid,
}

impl Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Invalid => {
                write!(
                    f,
                    "Invalid TIN/EIN"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

pub struct Tin(String);

impl Tin {
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl FromStr for Tin {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 11 {
            return Err(Error::Invalid);
        }

        let split = s.splitn(
            3, "-",
        );

        if !split
            .enumerate()
            .all(
                |(idx, chunk)| {
                    match idx {
                        0 => {
                            if chunk.len() != 3 {
                                return false;
                            }
                        }
                        1 => {
                            if chunk.len() != 2 {
                                return false;
                            }
                        }
                        2 => {
                            if chunk.len() != 4 {
                                return false;
                            }
                        }
                        _ => unreachable!(),
                    }
                    chunk
                        .chars()
                        .all(|c| c.is_ascii_digit())
                },
            )
        {
            return Err(Error::Invalid);
        }

        Ok(Tin(s.into()))
    }
}

pub struct Ein(String);

impl Ein {
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl FromStr for Ein {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 10 {
            return Err(Error::Invalid);
        }

        let split = s.splitn(
            2, "-",
        );

        if !split
            .enumerate()
            .all(
                |(idx, chunk)| {
                    match idx {
                        0 => {
                            if chunk.len() != 2 {
                                return false;
                            }
                        }
                        1 => {
                            if chunk.len() != 7 {
                                return false;
                            }
                        }
                        _ => unreachable!(),
                    }
                    chunk
                        .chars()
                        .all(|c| c.is_ascii_digit())
                },
            )
        {
            return Err(Error::Invalid);
        }

        Ok(Ein(s.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tin() {
        let tin: Result<Tin, _> = "123-45-6789".parse();
        assert!(tin.is_ok());
        assert_eq!(
            tin.unwrap()
                .get(),
            "123-45-6789"
        );
    }

    #[test]
    fn test_invalid_tin() {
        let invalid_tins = [
            "123-45-678",   // Too short
            "1234-56-7890", // Incorrect segment length
            "123-45-678A",  // Non-digit character
            "1234567890",   // Missing dashes
            "123--45-6789", // Extra dash
        ];

        for tin in invalid_tins {
            assert!(
                tin.parse::<Tin>()
                    .is_err()
            );
        }
    }

    #[test]
    fn test_valid_ein() {
        let ein: Result<Ein, _> = "12-3456789".parse();
        assert!(ein.is_ok());
        assert_eq!(
            ein.unwrap()
                .get(),
            "12-3456789"
        );
    }

    #[test]
    fn test_invalid_ein() {
        let invalid_eins = [
            "12--4-678",   // Too short
            "123-456789",  // Incorrect segment length
            "12-345678A",  // Non-digit character
            "123456789",   // Missing dash
            "12345678-",   // Bad dash position
            "12--3456789", // Extra dash
        ];

        for ein in invalid_eins {
            assert!(
                ein.parse::<Ein>()
                    .is_err()
            );
        }
    }
}
