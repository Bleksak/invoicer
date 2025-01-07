use std::fmt::Display;
use std::process::Command;
use std::str::FromStr;

use crate::accounting;
use crate::entity::eu::Entity;
use crate::payment_method::PaymentMethod;
use crate::time::Time;

use chrono::NaiveDate;
use fast_qr::convert::svg::SvgBuilder;
use fast_qr::convert::Builder;
use fast_qr::convert::Shape;
use fast_qr::qr;
use iban::Iban;
use iban::IbanLike;
use iso_currency::Currency;
use maud::html;
use maud::PreEscaped;
use maud::DOCTYPE;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;
use spayd::Spayd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InvoiceItemType {
    Hours(Time),
    Quantity(u32),
    Other(String),
}

impl FromStr for InvoiceItemType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(time) = value.parse() {
            return Ok(Self::Hours(time));
        }

        if let Ok(quantity) = value.parse() {
            return Ok(Self::Quantity(quantity));
        }

        Ok(Self::Other(value.to_string()))
    }
}

impl Display for InvoiceItemType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            InvoiceItemType::Hours(time) => {
                let result = write!(
                    f,
                    "{} hod",
                    time.hours()
                );

                if time.minutes() > 0 {
                    return write!(
                        f,
                        "{} min",
                        time.minutes()
                    );
                }

                result
            }
            InvoiceItemType::Quantity(quantity) => {
                write!(
                    f,
                    "{} ks",
                    quantity
                )
            }
            InvoiceItemType::Other(other) => {
                write!(
                    f,
                    "{}",
                    other
                )
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceItem {
    item_type: InvoiceItemType,
    description: String,
    price_per_unit: Decimal,
}

impl InvoiceItem {
    pub fn new(
        item_type: InvoiceItemType,
        description: impl Into<String>,
        price_per_unit: Decimal,
    ) -> Self {
        Self {
            item_type,
            description: description.into(),
            price_per_unit,
        }
    }

    pub fn to_html(
        &self,
        accounting: &accounting::Accounting,
    ) -> maud::Markup {
        html!(
            div class="invoice-item" {
                td class="align-right no-wrap" {
                    @match &self.item_type {
                        InvoiceItemType::Hours(time) => {
                            (time.hour_multiplicator()) " hod"
                        }
                        InvoiceItemType::Quantity(quantity) => {
                            (quantity) " ks"
                        }
                        InvoiceItemType::Other(other) => {
                            (other)
                        }
                    }
                }

                td {
                    (self.description);
                }

                td class="align-right no-wrap" {
                    (accounting.format_money(self.price_per_unit))
                }

                td class="align-right no-wrap" {
                    (accounting.format_money(self.price()))
                }
            }
        )
    }

    pub fn price(&self) -> Decimal {
        match &self.item_type {
            InvoiceItemType::Hours(time) => {
                Decimal::from_f64(time.hour_multiplicator()).unwrap() * self.price_per_unit
            }
            InvoiceItemType::Quantity(quantity) => {
                Decimal::from_u32(*quantity).unwrap() * self.price_per_unit
            }
            InvoiceItemType::Other(_) => self.price_per_unit,
        }
    }
}

impl FromStr for InvoiceItem {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split(' ');

        let item_type = parts
            .next()
            .ok_or("No item type")?;
        let price_per_unit = parts
            .next()
            .ok_or("No price per unit")?;
        let description = parts
            .collect::<Vec<&str>>()
            .join(" ");

        Ok(
            Self::new(
                InvoiceItemType::from_str(item_type)?,
                description,
                price_per_unit
                    .parse()
                    .or(Err("Invalid price"))?,
            ),
        )
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
    number: Decimal,
    contractor: Entity,
    client: Entity,
    iban: Iban,
    payment_method: PaymentMethod,
    items: Vec<InvoiceItem>,
    date: NaiveDate,
    due_date: NaiveDate,
    currency: Currency,
    note: Option<String>,
}

impl Invoice {
    pub fn new(
        number: Decimal,
        contractor: Entity,
        client: Entity,
        iban: Iban,
        payment_method: PaymentMethod,
        items: Vec<InvoiceItem>,
        date: NaiveDate,
        due_date: NaiveDate,
        currency: Currency,
        note: Option<impl Into<String>>,
    ) -> Self {
        Self {
            number,
            contractor,
            client,
            iban,
            payment_method,
            items,
            date,
            due_date,
            currency,
            note: note.map(|x| x.into()),
        }
    }
}

impl Invoice {
    fn qr_code(
        &self,
        items_sum: &Decimal,
    ) -> Option<String> {
        if let PaymentMethod::BankTransfer(symbol) = &self.payment_method {
            let spayd = Spayd::new_v1_0(
                [
                    (
                        spayd::fields::ACCOUNT,
                        &self
                            .iban
                            .electronic_str()
                            .to_string(),
                    ),
                    (
                        spayd::fields::AMOUNT,
                        &items_sum.to_string(),
                    ),
                    (
                        spayd::fields::CURRENCY,
                        &self
                            .currency
                            .code()
                            .to_string(),
                    ),
                    (
                        "X-VS", symbol,
                    ),
                ],
            );

            if let Ok(qr) = qr::QRBuilder::new(spayd.to_string()).build() {
                return Some(
                    SvgBuilder::default()
                        .shape(Shape::RoundedSquare)
                        .background_color(
                            [
                                255, 255, 255, 0,
                            ],
                        )
                        .margin(0)
                        .to_str(&qr),
                );
            }
        }

        None
    }

    pub fn to_html(&self) -> maud::Markup {
        let ac = accounting::create_accounting_from_currency(self.currency);
        let fmt = "%d. %m. %Y";

        let items_sum: Decimal = self
            .items
            .iter()
            .map(|x| x.price())
            .sum();

        let qr_code = self.qr_code(&items_sum);

        html!(
            (DOCTYPE)
            html {
                head {
                    title { "Faktura " (self.number) };
                    link rel="stylesheet" href="templates/style.css";
                }
                body {
                    div class="space-between block" {
                        div {}
                        div class="block-right" {
                            h1 class="line-above-bold" {
                                "Faktura " span .invoice-number { (self.number) }
                            }
                        }
                    }
                    div class="block" {
                        div class="entity" {
                            h2 { "DODAVATEL" }
                            (self.contractor.to_html());
                        }

                        div class="entity block-right" {
                            h2 { "ODBĚRATEL" }
                            (self.client.to_html());
                        }
                    }

                    div class = "block" {
                        div class = "payment-info" {
                            div class="space-between" {
                                p class="text-grayed" {
                                    "Bankovní účet"
                                }

                                p {
                                    (self.iban.to_bank_account_number())
                                }
                            }


                            @if let PaymentMethod::BankTransfer(var_symbol) = &self.payment_method {
                                div class="space-between" {
                                    p class = "text-grayed" {
                                        "Variabilní symbol"
                                    }

                                    p {
                                        (var_symbol)
                                    }
                                }
                            }

                            div class="space-between" {
                                p class="text-grayed" {
                                    "Způsob platby"
                                }
                                p {
                                    @match &self.payment_method {
                                        PaymentMethod::Cash => {
                                            "Hotově"
                                        }
                                        PaymentMethod::Card(_) => {
                                            "Platební kartou"
                                        }
                                        PaymentMethod::BankTransfer(_) => {
                                            "Bankovním převodem"
                                        }
                                    }
                                }
                            }
                        }

                        div class="dates block-right" {
                            div class="space-between" {
                                p class="text-grayed" {
                                    "Datum vystavení"
                                }

                                p {
                                    (self.date.format(fmt));
                                }
                            }

                            div class="space-between" {
                                p class="text-grayed" {
                                    "Datum splatnosti"
                                }

                                p {
                                    (self.due_date.format(fmt));
                                }
                            }
                        }

                    }
                    table class="invoice-items line-below" {
                        thead class="line-below" {
                            tr {
                                th class="align-right no-wrap" { "" }
                                th { "" }
                                th class="align-right no-wrap" { "CENA ZA MJ" }
                                th class="align-right no-wrap" { "CELKEM" }
                            }
                        }
                        @for item in &self.items {
                            tr {
                                ({
                                    item.to_html(&ac)
                                });
                            }
                        }
                    }

                    div class="space-between block" {
                        div {
                            div class = "qr" {
                                @if let Some(qr_code) = qr_code {
                                    (PreEscaped(qr_code))
                                }
                            }
                        }

                        div class = "line-above-bold block-right border-black" {
                            p class = "text-bold text-big align-right" {
                                (ac.format_money(items_sum))
                            }
                        }
                    }

                    @if let Some(note) = &self.note {
                        div class = "note" {
                            p {
                                (note)
                            }
                        }
                    }
                }
            }
        )
    }

    pub fn to_pdf(
        &self,
        filename: &str,
    ) -> Option<()> {
        Command::new("weasyprint")
            .arg(
                format!(
                    "data:text/html,{}",
                    self.to_html()
                        .into_string()
                ),
            )
            .arg("-u")
            .arg(
                std::fs::canonicalize(".")
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
            .arg(filename)
            .output()
            .ok()
            .map(|_| ())
    }
}

pub trait ToBankAccountNumber {
    fn to_bank_account_number(&self) -> String;
}

impl ToBankAccountNumber for Iban {
    fn to_bank_account_number(&self) -> String {
        let value = self.to_string();
        let bank_code = value[5..9].to_string();
        let account_number = value[9..]
            .replace(
                ' ', "",
            )
            .trim_start_matches('0')
            .to_string();
        format!(
            "{}/{}",
            account_number, bank_code
        )
    }
}
