use serde::{Deserialize, Serialize};

/// Represents an address
#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    city: String,
    street: String,
    postal_code: String,
    house_number: u32,
    orientation_number: Option<u32>,
}

impl Address {
    pub fn new(
        city: String,
        street: String,
        postal_code: String,
        house_number: u32,
        orientation_number: Option<u32>,
    ) -> Self {
        Self {
            city,
            street,
            postal_code,
            house_number,
            orientation_number,
        }
    }

    /// Get the first line of the address
    pub fn get_first_line(&self) -> String {
        let mut result = format!(
            "{} {}",
            self.street, self.house_number
        );

        if let Some(orientation_number) = self.orientation_number {
            result.push_str(
                &format!(
                    "/{}",
                    orientation_number
                ),
            );
        }

        result
    }

    /// Get the second line of the address
    pub fn get_second_line(&self) -> String {
        let mut number_str = format!(
            "{:05}",
            self.postal_code
        );
        number_str.insert(
            3, ' ',
        );
        number_str.push(' ');
        number_str.push_str(&self.city);
        number_str
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_address() {
        let address = super::Address::new(
            "Praha".to_string(),
            "Husova".to_string(),
            "12000".to_string(),
            123,
            None,
        );

        assert_eq!(
            address.get_first_line(),
            "Husova 123"
        );
        assert_eq!(
            address.get_second_line(),
            "120 00 Praha"
        );
    }
}
