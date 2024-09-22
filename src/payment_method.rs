use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaymentMethod {
    Cash,
    Card(String),
    BankTransfer(String), // variable symbol
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(
                f,
                "Hotově"
            ),
            PaymentMethod::Card(card_number) => write!(
                f,
                "Platbení kartou: {}",
                card_number
            ),
            PaymentMethod::BankTransfer(_) => write!(
                f,
                "Bankovním převodem: "
            ),
        }
    }
}

impl FromStr for PaymentMethod {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "cash" {
            return Ok(PaymentMethod::Cash);
        }

        if value.starts_with("card") {
            let card_number = value.split(' ').skip(1).collect::<Vec<&str>>().join(" ");
            return Ok(PaymentMethod::Card(card_number));
        }

        if value.starts_with("bank") {
            let var_symbol = value.split(' ').skip(1).collect::<Vec<&str>>().join(" ");

            return Ok(PaymentMethod::BankTransfer(var_symbol));
        }

        Err(
            format!(
                "Unknown payment method: {}",
                value
            ),
        )
    }
}
