# Invoicer

## Create CZ/SK invoices using Rust.

Make sure to use Decimal::new() to create Decimal values instead of dec!() macro.

```rust
use invoicer::Invoice;

let contractor = "contractor registration number (ičo)";
let iban = "your IBAN";
let due_days = 14;

let contractor: RegistrationNumber = contractor.parse().unwrap();
let client: RegistrationNumber = "client registration number".parse().unwrap();

let contractor: Entity = contractor.try_into().unwrap();
let client: Entity = client.try_into().unwrap();

let iban: Iban = iban.parse().unwrap();
let today = Utc::now().with_timezone(&Local).date_naive();
let due_date = today + Duration::days(due_days);

let invoice = Invoice::new(
    "202403".parse().unwrap(),
    contractor,
    client,
    iban,
    payment_method::PaymentMethod::BankTransfer("202403".to_string()),
    vec![
        InvoiceItem::new(
            InvoiceItemType::Hours(Time::new(1, 30)),
            "Položka faktury 1",
            Decimal::new(1234.0),
        ),
    ],
    today,
    due_date,
    Currency::CZK,
    Some("Fyzická osoba zapsaná v živnostenském rejstříku.")
);

```
