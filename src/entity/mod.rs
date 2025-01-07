use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod eu;
pub mod us;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entity {
    Eu(eu::Entity),
    Us,
    Canada,
    China,
    India,
    Japan,
    Korea,
    Australia,
    Brazil,
    Argentina,
    Africa,

}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Contractor,
    Client,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                EntityType::Contractor => "DODAVATEL",
                EntityType::Client => "ODBĚRATEL",
            },
        )
    }
}