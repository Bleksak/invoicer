use serde::{Deserialize, Serialize};

use crate::{address::Address, entity::Entity, registration_number::RegistrationNumber};
use std::io::Read;

// #[derive(Debug, Serialize, Deserialize)] pub struct AresListOfRegistrations {
//     #[serde(rename = "stavZdrojeVr")]
//     vr: String,
//
//     #[serde(rename = "stavZdrojeRes")]
//     res: String,
//
//     #[serde(rename = "stavZdrojeRzp")]
//     rzp: String,
//
//     #[serde(rename = "stavZdrojeNrpzs")]
//     nrpzs: String,
//
//     #[serde(rename = "stavZdrojeRpsh")]
//     rpsh: String,
//
//     #[serde(rename = "stavZdrojeRcns")]
//     rcns: String,
//
//     #[serde(rename = "stavZdrojeSzr")]
//     szr: String,
//
//     #[serde(rename = "stavZdrojeDph")]
//     dph: String,
//
//     #[serde(rename = "stavZdrojeSd")]
//     sd: String,
//
//     #[serde(rename = "stavZdrojeIr")]
//     ir: String,
//
//     #[serde(rename = "stavZdrojeCeu")]
//     ceu: String,
//
//     #[serde(rename = "stavZdrojeRs")]
//     rs: String,
//
//     #[serde(rename = "stavZdrojeRed")]
//     red: String,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct AresSidlo {
    #[serde(rename = "kodStatu")]
    country_code: String,

    #[serde(rename = "nazevStatu")]
    country_name: String,

    #[serde(rename = "kodKraje")]
    region_code: u32,

    #[serde(rename = "nazevKraje")]
    region_name: String,

    #[serde(rename = "kodOkresu")]
    district_code: Option<u32>,

    #[serde(rename = "nazevOkresu")]
    district_name: Option<String>,

    #[serde(rename = "kodObce")]
    municipality_code: u32,

    #[serde(rename = "nazevObce")]
    municipality_name: String,

    #[serde(rename = "cisloDomovni")]
    house_number: u32,

    #[serde(rename = "kodCastiObce")]
    municipality_part: u32,

    #[serde(rename = "nazevCastiObce")]
    municipality_part_name: String,

    #[serde(rename = "kodAdresnihoMista")]
    address_place_code: u32,

    #[serde(rename = "psc")]
    postal_code: u32,

    #[serde(rename = "textovaAdresa")]
    text_address: String,

    #[serde(rename = "typCisloDomovni")]
    house_number_type: u32,

    #[serde(rename = "standardizaceAdresy")]
    address_normalized: bool,

    #[serde(rename = "cisloOrientacni")]
    orientation_number: Option<u32>,

    #[serde(rename = "nazevUlice")]
    street: Option<String>,

    #[serde(rename = "nazevMestskeCastiObvodu")]
    city_part: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AresAdresa {
    #[serde(rename = "radekAdresy1")]
    first_line: String,

    #[serde(rename = "radekAdresy2")]
    second_line: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AresResponse {
    ico: String,

    #[serde(rename = "obchodniJmeno")]
    name: String,

    #[serde(rename = "sidlo")]
    office: AresSidlo,

    // #[serde(rename = "pravniForma")]
    // legal_form: String,

    // #[serde(rename = "financniUrad")]
    // tax_office: String,

    // #[serde(rename = "datumVzniku")]
    // created_at: String,

    // #[serde(rename = "datumAktualizace")]
    // updated_at: String,

    // #[serde(rename = "icoId")]
    // registration_number_id: String,
    #[serde(rename = "adresaDorucovaci")]
    address: AresAdresa,

    // #[serde(rename = "seznamRegistraci")]
    // list_of_registrations: AresListOfRegistrations,

    // #[serde(rename = "czNace")]
    // nace: Vec<String>,
    dic: Option<String>,
}

/// Fetches data from ARES registry
pub fn fetch_from_ares(number: RegistrationNumber) -> anyhow::Result<Entity> {
    let url = format!(
        "https://ares.gov.cz/ekonomicke-subjekty-v-be/rest/ekonomicke-subjekty/{}",
        number.get()
    );

    let mut result = String::new();

    reqwest::blocking::get(url)?.read_to_string(&mut result)?;

    let ares_response: AresResponse = serde_json::from_str(&result)?;

    Ok(
        Entity::new(
            number,
            ares_response.name,
            Address::new(
                ares_response
                    .office
                    .city_part
                    .map(|x| x.split('-').collect::<Vec<&str>>().join(" - "))
                    .unwrap_or(ares_response.office.municipality_name),
                ares_response
                    .office
                    .street
                    .unwrap_or(ares_response.office.municipality_part_name),
                ares_response.office.postal_code.to_string(),
                ares_response.office.house_number,
                ares_response.office.orientation_number,
            ),
            ares_response.dic,
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::RegistrationNumber;

    #[test]
    fn test_fetch_from_ares() {
        let registration_number: RegistrationNumber =
            "27082440".parse().expect("Invalid registration number");
        let result =
            super::fetch_from_ares(registration_number).expect("Failed to fetch from ARES");
        assert_eq!(
            result.name,
            "Alza.cz a.s."
        );
    }
}
