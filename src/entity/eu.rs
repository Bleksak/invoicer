use maud::html;
use serde::{Deserialize, Serialize};

use crate::{address::Address, ares, registration_number::RegistrationNumber};



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

    pub fn to_html(&self) -> maud::Markup {
        html!(
            div class="entity-info" {
                strong class="entity-name" { (self.name) }

                div class="entity-address" {
                    p class="text-grayed" { (self.address.get_first_line()) };
                    p class="text-grayed" { (self.address.get_second_line()) };
                }

                div class="entity-billing-info" {
                    div class="space-between" {
                        p class="text-grayed" { "IČO" };
                        p { (self.identifier) };
                    }

                    div class="space-between" {
                        @if let Some(vat_number) = &self.vat_number {
                                p class="text-grayed" { "DPH" };
                                p { (vat_number) };
                        } @else {
                            p { "Neplátce DPH" }
                        }
                    }
                }
            }
        )
    }
}

impl TryFrom<RegistrationNumber> for Entity {
    type Error = ares::Error;

    fn try_from(value: RegistrationNumber) -> Result<Self, Self::Error> {
        use crate::ares;

        ares::fetch_from_ares(value)
    }
}
