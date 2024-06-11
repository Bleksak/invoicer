use serde::Serialize;

#[derive(Debug, Serialize)]
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

    pub fn get_first_line(&self) -> String {
        let mut result = format!("{} {}", self.street, self.house_number);

        if let Some(orientation_number) = self.orientation_number {
            result.push_str(&format!("/{}", orientation_number));
        }

        result
    }

    pub fn get_second_line(&self) -> String {
        let mut number_str = format!("{:05}", self.postal_code);
        number_str.insert(3, ' ');
        number_str.push(' ');
        number_str.push_str(&self.city);
        number_str
    }
}
