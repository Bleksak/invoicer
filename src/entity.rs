use std::fmt::Display;

use serde::Serialize;

use crate::{address::Address, registration_number::RegistrationNumber};

#[derive(Debug, Serialize, Clone, Copy)]
pub enum EntityType {
    Contractor,
    Client,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EntityType::Contractor => "DODAVATEL",
            EntityType::Client => "ODBĚRATEL",
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Entity {
    pub identifier: RegistrationNumber,
    pub name: String,

    pub address: Address,
    pub vat_number: Option<String>,
}

impl Entity {
    pub fn new(
        identifier: RegistrationNumber,
        name: String,
        address: Address,
        vat_number: Option<String>,
    ) -> Self {
        Self {
            identifier,
            name,
            address,
            vat_number,
        }
    }
}

impl TryFrom<RegistrationNumber> for Entity {
    type Error = anyhow::Error;

    fn try_from(value: RegistrationNumber) -> Result<Self, Self::Error> {
        use crate::ares;

        ares::fetch_from_ares(value)
    }
}
