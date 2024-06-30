use accounting::Accounting;
use iso_currency::Currency;

pub fn create_accounting_from_currency(currency: Currency) -> Accounting {
    let mut ac = Accounting::new_from(&currency.symbol().to_string(), currency.exponent().unwrap_or(2) as usize);

    ac.set_format("{v} {s}");
    ac.set_format_positive("{v} {s}");
    ac.set_format_negative("-{v} {s}");
    ac.set_format_zero("{v} {s}");
    ac.set_decimal_separator(",");
    ac.set_thousand_separator(" ");

    match currency {
        Currency::CZK => {
        },
        Currency::EUR => {
            ac.set_format("{s}{v}");
            ac.set_format_positive("{s}{v}");
            ac.set_format_zero("{s}{v}");
            ac.set_format_negative("-{s}{v}");
            ac.set_decimal_separator(",");
            ac.set_thousand_separator(".");
        },
        Currency::USD => {
            ac.set_format("{s}{v}");
            ac.set_format_positive("{s}{v}");
            ac.set_format_zero("{s}{v}");
            ac.set_format_negative("-{s}{v}");
            ac.set_decimal_separator(".");
            ac.set_thousand_separator(",");
        },
        _ => {}
    }

    ac
}
