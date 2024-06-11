use std::{env, fs::File, io::BufWriter};

use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use entity::Entity;
use iban::Iban;
use invoice::{Invoice, InvoiceItem, InvoiceItemType};
use printpdf::PdfDocumentReference;
use registration_number::RegistrationNumber;
use rust_decimal_macros::dec;
use time::Time;

mod address;
mod ares;
mod entity;
mod invoice;
mod payment_method;
mod pdf;
mod registration_number;
mod time;

fn main() {
    dotenvy::dotenv().unwrap();

    let iban = env::var("IBAN").unwrap();
    let contractor = env::var("CONTRACTOR").unwrap();
    let due_days = env::var("DUE_DAYS").unwrap().parse().unwrap();

    let contractor: RegistrationNumber = contractor.parse().unwrap();
    let client: RegistrationNumber = "29210372".parse().unwrap();
    // let client: RegistrationNumber = "18038140".parse().unwrap();

    let contractor: Entity = contractor.try_into().unwrap();
    let client: Entity = client.try_into().unwrap();

    let iban: Iban = iban.parse().unwrap();

    let today = Utc::now().with_timezone(&Local).date_naive();

    let due_date = today + Duration::days(due_days);

    // let date: NaiveDate = "2024-10-10".parse().unwrap();

    let invoice = Invoice::new(
        "202403".parse().unwrap(),
        contractor,
        client,
        payment_method::PaymentMethod::BankTransfer(iban, "202403".to_string()),
        vec![InvoiceItem::new(
            InvoiceItemType::Hours(Time::new(1, 30)),
            "Programování SmartEmalingu".to_string(),
            dec!(350.0),
        )],
        today,
        due_date,
    );

    let pdf: PdfDocumentReference = invoice.into();
    let file = File::create("example.pdf").unwrap();
    let mut writer = BufWriter::new(file);
    pdf.save(&mut writer).unwrap();
}
