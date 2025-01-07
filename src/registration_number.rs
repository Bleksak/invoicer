use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

/// Representation of czech company registration number(IČO).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RegistrationNumber(String);

#[derive(Debug)]
pub enum RegistrationNumberError {
    InvalidNumber,
}

impl Display for RegistrationNumberError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            RegistrationNumberError::InvalidNumber => {
                write!(
                    f,
                    "Neplatné IČO"
                )
            }
        }
    }
}

impl Error for RegistrationNumberError {}

impl RegistrationNumber {
    /// Checks if the Czech registration number is valid.
    fn valid(number: &str) -> Option<()> {
        let control_char = number
            .chars()
            .last()?;
        let mut calculated_control = 0;

        for (i, ch) in number
            .chars()
            .rev()
            .skip(1)
            .enumerate()
        {
            let digit: u32 = ch.to_digit(10)?;

            let weight = i + 2;

            calculated_control += digit * (weight as u32);
        }

        let calculated_control = (11 - (calculated_control % 11)) % 10;

        if calculated_control != control_char.to_digit(10)? {
            return None;
        }

        Some(())
    }

    /// Returns the registration number as a string.
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl Display for RegistrationNumber {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.0
            .fmt(f)
    }
}

impl FromStr for RegistrationNumber {
    type Err = RegistrationNumberError;

    fn from_str(number: &str) -> Result<Self, Self::Err> {
        if number.len() != 8 {
            return Err(Self::Err::InvalidNumber);
        }

        Self::valid(number)
            .ok_or(Self::Err::InvalidNumber)
            .map(|_| Self(number.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::RegistrationNumber;
    use super::RegistrationNumberError;

    #[test]
    fn test_registration_number() {
        assert!(RegistrationNumber::valid("27082440").is_some());
        assert!(RegistrationNumber::valid("CZ00000000").is_none());
    }

    #[test]
    fn test_registration_number_parse() {
        let number: Result<RegistrationNumber, RegistrationNumberError> = "27082440".parse();

        assert!(number.is_ok());
        assert_eq!(
            number
                .unwrap()
                .get(),
            "27082440"
        );
    }
}
