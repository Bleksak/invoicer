use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum PaymentMethod {
    Cash,
    Card(String),
    BankTransfer(String), // variable symbol
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "Hotově"),
            PaymentMethod::Card(card_number) => write!(f, "Platbení kartou: {}", card_number),
            PaymentMethod::BankTransfer(_) => write!(f, "Bankovním převodem: "),
        }
    }
}
