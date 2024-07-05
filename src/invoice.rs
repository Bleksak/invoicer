use std::fmt::Display;

use crate::{
    accounting,
    entity::{Entity, EntityType},
    payment_method::PaymentMethod,
    pdf::{calculate_text_width, PdfData, PdfFont},
    time::Time,
};
use azul_text_layout::text_shaping::get_font_metrics_freetype;
use chrono::NaiveDate;
use fast_qr::{
    convert::{svg::SvgBuilder, Builder, Shape},
    qr,
};
use iban::{Iban, IbanLike};
use iso_currency::Currency;
use printpdf::{
    Color, Line, Mm, PdfDocument, PdfDocumentReference, Point, Px, Rgb, SvgTransform, TextRenderingMode
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use serde::Serialize;
use spayd::Spayd;

#[derive(Debug, Serialize)]
pub enum InvoiceItemType {
    Hours(Time),
    Quantity(u32),
    Other(String),
}

impl Display for InvoiceItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceItemType::Hours(time) => {
                let result = write!(f, "{} hod", time.hours());

                if time.minutes() > 0 {
                    return write!(f, "{} min", time.minutes());
                }

                result
            }
            InvoiceItemType::Quantity(quantity) => write!(f, "{} ks", quantity),
            InvoiceItemType::Other(other) => write!(f, "{}", other),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InvoiceItem {
    item_type: InvoiceItemType,
    description: String,
    price_per_unit: Decimal,
}

impl InvoiceItem {
    pub fn new(item_type: InvoiceItemType, description: String, price_per_unit: Decimal) -> Self {
        Self {
            item_type,
            description,
            price_per_unit,
        }
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

#[derive(Debug)]
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
        note: Option<String>,
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
            note,
        }
    }
}

impl Invoice {
    pub fn pdf_draw_heading_text(
        number: &Decimal,
        pdf: &PdfData,
        bound_x: Mm,
        position: (Mm, Mm),
    ) -> (Mm, Mm) {
        let line = Line {
            points: vec![
                (Point::new(position.0, position.1), false),
                (
                    Point::new((position.0 + Mm(84.0)).max(bound_x), position.1),
                    false,
                ),
            ],
            is_closed: true,
        };

        pdf.layer.set_outline_thickness(Mm(2.25).0);
        pdf.layer.set_outline_color(pdf.gray.clone());
        pdf.layer.add_line(line);

        pdf.layer.begin_text_section();

        pdf.layer.set_font(&pdf.font_bold.font, Mm(17.5).0);
        pdf.layer.set_text_cursor(position.0, position.1 - Mm(8.0));
        pdf.layer.set_fill_color(pdf.black.clone());
        pdf.layer.set_text_rendering_mode(TextRenderingMode::Fill);
        pdf.layer.write_text("Faktura ", &pdf.font_bold.font);

        pdf.layer.set_fill_color(pdf.gray.clone());

        pdf.layer.write_text(number.to_string(), &pdf.font.font);

        pdf.layer.end_text_section();

        (Mm(84.0), Mm(18.0))
    }

    pub fn pdf_draw_entity_info(
        entity: &Entity,
        r#type: EntityType,
        pdf: &PdfData,
        max_position: Mm,
        position: (Mm, Mm),
    ) -> (Mm, Mm) {
        let line = Line {
            points: vec![
                (Point::new(position.0, position.1), false),
                (Point::new(position.0 + Mm(4.0), position.1), false),
            ],
            is_closed: true,
        };

        pdf.layer.set_outline_thickness(0.0);

        pdf.layer.add_line(line);
        pdf.layer.set_text_rendering_mode(TextRenderingMode::Fill);

        pdf.layer.set_fill_color(pdf.gray.clone());
        pdf.layer.use_text(
            r#type.to_string(),
            10.0,
            position.0,
            position.1 - Mm(5.0),
            &pdf.font.font,
        );

        pdf.layer.set_fill_color(pdf.black.clone());

        pdf.layer.use_text(
            &entity.name,
            Mm(10.0).0,
            position.0,
            position.1 - (Mm(5.0) + pdf.line_height * 2.0),
            &pdf.font_bold.font,
        );

        pdf.layer.set_fill_color(pdf.gray.clone());
        pdf.layer.use_text(
            entity.address.get_first_line(),
            10.0,
            position.0,
            position.1 - (Mm(6.0) + pdf.line_height * 3.0),
            &pdf.font.font,
        );
        pdf.layer.use_text(
            entity.address.get_second_line(),
            Mm(10.0).0,
            position.0,
            position.1 - (Mm(6.0) + pdf.line_height * 4.0),
            &pdf.font.font,
        );

        pdf.layer.use_text(
            "IČO",
            Mm(10.0).0,
            position.0,
            position.1 - (Mm(6.0) + pdf.line_height * 6.0),
            &pdf.font.font,
        );

        pdf.layer.set_fill_color(pdf.black.clone());

        let identifier = entity.identifier.to_string();

        pdf.layer.use_text(
            &identifier,
            Mm(10.0).0,
            max_position - calculate_text_width(&identifier, pdf, 10.0),
            position.1 - (Mm(6.0) + pdf.line_height * 6.0),
            &pdf.font.font,
        );

        if let Some(vat) = &entity.vat_number {
            pdf.layer.set_fill_color(pdf.gray.clone());

            pdf.layer.use_text(
                "DIČ",
                10.0,
                position.0,
                position.1 - (Mm(6.0) + pdf.line_height * 7.0),
                &pdf.font.font,
            );

            pdf.layer.set_fill_color(pdf.black.clone());

            pdf.layer.use_text(
                vat,
                10.0,
                max_position - calculate_text_width(vat, pdf, 10.0),
                position.1 - (Mm(6.0) + pdf.line_height * 7.0),
                &pdf.font.font,
            );
        } else {
            pdf.layer.set_fill_color(pdf.black.clone());
            pdf.layer.use_text(
                "Neplátce DPH",
                Mm(10.0).0,
                position.0,
                position.1 - (Mm(6.0) + pdf.line_height * 7.0),
                &pdf.font.font,
            );
        }

        pdf.layer.set_fill_color(pdf.gray.clone());

        (max_position, Mm(6.0) + pdf.line_height * 8.0)
    }

    pub fn pdf_draw_payment_method(
        iban: &Iban,
        payment_method: &PaymentMethod,
        pdf: &PdfData,
        max_position: Mm,
        position: (Mm, Mm),
    ) -> (Mm, Mm) {
        pdf.layer.set_fill_color(pdf.gray.clone());

        match payment_method {
            PaymentMethod::Cash => todo!(),
            PaymentMethod::Card(_) => todo!(),
            PaymentMethod::BankTransfer(var_symbol) => {
                let bank_account = iban.to_bank_account_number();
                let r#type = "Převodem";

                pdf.layer.use_text(
                    "Bankovní účet",
                    10.0,
                    position.0,
                    position.1 - pdf.line_height,
                    &pdf.font.font,
                );
                pdf.layer.use_text(
                    "Variabilní symbol",
                    10.0,
                    position.0,
                    position.1 - pdf.line_height * 2.0,
                    &pdf.font.font,
                );
                pdf.layer.use_text(
                    "Způsob platby",
                    10.0,
                    position.0,
                    position.1 - pdf.line_height * 3.0,
                    &pdf.font.font,
                );

                pdf.layer.set_fill_color(pdf.black.clone());

                pdf.layer.use_text(
                    &bank_account,
                    10.0,
                    max_position - calculate_text_width(&bank_account, pdf, 10.0),
                    position.1 - pdf.line_height,
                    &pdf.font.font,
                );
                pdf.layer.use_text(
                    var_symbol,
                    10.0,
                    max_position - calculate_text_width(var_symbol, pdf, 10.0),
                    position.1 - pdf.line_height * 2.0,
                    &pdf.font.font,
                );
                pdf.layer.use_text(
                    r#type,
                    10.0,
                    max_position - calculate_text_width(r#type, pdf, 10.0),
                    position.1 - pdf.line_height * 3.0,
                    &pdf.font.font,
                );
            }
        }

        (max_position, pdf.line_height * 3.0)
    }

    pub fn pdf_draw_dates(
        invoice_date: &NaiveDate,
        due_date: &NaiveDate,
        pdf: &PdfData,
        max_position: Mm,
        position: (Mm, Mm),
    ) -> (Mm, Mm) {
        pdf.layer.set_fill_color(pdf.gray.clone());

        pdf.layer.use_text(
            "Datum vystavení",
            10.0,
            position.0,
            position.1 - pdf.line_height,
            &pdf.font.font,
        );

        pdf.layer.use_text(
            "Datum splatnosti",
            10.0,
            position.0,
            position.1 - pdf.line_height * 2.0,
            &pdf.font.font,
        );

        pdf.layer.set_fill_color(pdf.black.clone());

        let fmt = "%d. %m. %Y";
        let invoice_date = invoice_date.format(fmt).to_string();
        let due_date = due_date.format(fmt).to_string();

        pdf.layer.use_text(
            &invoice_date,
            10.0,
            max_position - calculate_text_width(&invoice_date, pdf, 10.0),
            position.1 - pdf.line_height,
            &pdf.font.font,
        );

        pdf.layer.use_text(
            &due_date,
            10.0,
            max_position - calculate_text_width(&due_date, pdf, 10.0),
            position.1 - pdf.line_height * 2.0,
            &pdf.font.font,
        );

        (max_position, pdf.line_height * 2.0)
    }

    pub fn pdf_draw_items(
        items: &Vec<InvoiceItem>,
        currency: Currency,
        price_total: Decimal,
        pdf: &PdfData,
        max_position: Mm,
        position: (Mm, Mm),
    ) -> (Mm, Mm) {
        let padding_y = pdf.line_height * 4.0;

        let line = Line {
            points: vec![
                (Point::new(position.0, position.1 - padding_y), false),
                (Point::new(max_position, position.1 - padding_y), false),
            ],
            is_closed: true,
        };

        pdf.layer.set_fill_color(pdf.light_gray.clone());
        pdf.layer.set_outline_color(pdf.light_gray.clone());

        pdf.layer.set_outline_thickness(0.0);

        pdf.layer.add_line(line);

        let mut max_price = dec!(0.0);
        let mut max_price_per_unit = dec!(0.0);

        let mut longest_count = String::new();

        for item in items {
            let price_per_unit = item.price_per_unit;
            let price = item.price();

            max_price = max_price.max(price);
            max_price_per_unit = max_price_per_unit.max(price_per_unit);

            match &item.item_type {
                InvoiceItemType::Hours(x) => {
                    let count = x.hour_multiplicator().to_string();
                    if count.len() > longest_count.len() {
                        longest_count = count;
                    }
                }
                InvoiceItemType::Quantity(x) => {
                    let count = x.to_string();
                    if count.len() > longest_count.len() {
                        longest_count = count;
                    }
                }
                _ => {}
            };
        }

        let ac = accounting::create_accounting_from_currency(currency);

        let max_price = ac.format_money(max_price);

        // let max_price_per_unit = ac.format_money(max_price_per_unit);

        let table_font_size = 9.2;

        let max_price_width = calculate_text_width(&max_price, pdf, table_font_size);
        // let max_price_per_unit_width =
        //     calculate_text_width(&max_price_per_unit, pdf, table_font_size);

        let gap = Mm(5.0);

        let total_price_string = "CELKEM";
        let price_per_unit_string = "CENA ZA MJ";

        let total_price_width = calculate_text_width(&total_price_string, pdf, table_font_size);
        let price_per_unit_width =
            calculate_text_width(&price_per_unit_string, pdf, table_font_size);

        pdf.layer.set_fill_color(pdf.gray.clone());
        pdf.layer.set_outline_color(pdf.gray.clone());

        pdf.layer.use_text(
            total_price_string,
            table_font_size,
            max_position - total_price_width,
            position.1 - padding_y + pdf.line_height * 0.5,
            &pdf.font.font,
        );

        pdf.layer.use_text(
            price_per_unit_string,
            table_font_size,
            max_position - total_price_width - max_price_width - price_per_unit_width - gap,
            position.1 - padding_y + pdf.line_height * 0.5,
            &pdf.font.font,
        );

        pdf.layer.set_fill_color(pdf.black.clone());
        pdf.layer.set_outline_color(pdf.black.clone());

        let count_offset = calculate_text_width(&longest_count, pdf, table_font_size);

        let mut y_offset = pdf.line_height;

        for item in items {
            match &item.item_type {
                InvoiceItemType::Hours(time) => {
                    let time = time.hour_multiplicator();
                    let label = "hod";

                    pdf.layer.use_text(
                        &time.to_string(),
                        table_font_size,
                        position.0,
                        position.1 - padding_y - y_offset,
                        &pdf.font.font,
                    );

                    pdf.layer.use_text(
                        label,
                        table_font_size,
                        position.0 + count_offset + gap,
                        position.1 - padding_y - y_offset,
                        &pdf.font.font,
                    );
                }
                InvoiceItemType::Quantity(quantity) => {
                    pdf.layer.use_text(
                        &quantity.to_string(),
                        table_font_size,
                        position.0,
                        position.1 - padding_y - y_offset,
                        &pdf.font.font,
                    );

                    pdf.layer.use_text(
                        "ks",
                        table_font_size,
                        position.0 + count_offset + gap,
                        position.1 - padding_y - y_offset,
                        &pdf.font.font,
                    );
                }
                InvoiceItemType::Other(other) => {
                    pdf.layer.use_text(
                        other,
                        table_font_size,
                        position.0,
                        position.1 - padding_y - y_offset,
                        &pdf.font.font,
                    );
                }
            }

            let description_length = calculate_text_width(&item.description, pdf, table_font_size);

            pdf.layer.begin_text_section();

            pdf.layer.set_font(&pdf.font.font, table_font_size);
            pdf.layer.set_line_height(12.0);

            pdf.layer.set_text_cursor(
                position.0 + count_offset + gap * 3.0,
                position.1 - padding_y - y_offset,
            );

            let start = position.0 + count_offset + gap * 3.0;
            let end = max_position
                - total_price_width
                - max_price_width
                - price_per_unit_width
                - gap * 4.0;

            let threshold = end - start;

            let mut add_offsets = Mm(0.0);

            if description_length > threshold {
                let mut new_description = String::new();

                for word in item.description.split(' ') {
                    new_description.push_str(word);
                    new_description.push(' ');

                    let accumulated_length =
                        calculate_text_width(&new_description, pdf, table_font_size);

                    if accumulated_length >= threshold {
                        pdf.layer.write_text(&new_description, &pdf.font.font);
                        pdf.layer.add_line_break();
                        add_offsets += pdf.line_height;

                        new_description.clear();
                    }
                }
                if !new_description.is_empty() {
                    pdf.layer.write_text(&new_description, &pdf.font.font);
                }
            } else {
                pdf.layer.write_text(&item.description, &pdf.font.font);
            }

            pdf.layer.end_text_section();

            let price_per_unit = ac.format_money(item.price_per_unit);

            let price_per_unit_width = calculate_text_width(&price_per_unit, pdf, table_font_size);

            pdf.layer.use_text(
                price_per_unit,
                table_font_size,
                max_position - total_price_width - max_price_width - price_per_unit_width - gap,
                position.1 - padding_y - y_offset,
                &pdf.font.font,
            );

            let price = ac.format_money(item.price());

            let price_width = calculate_text_width(&price, pdf, table_font_size);

            pdf.layer.use_text(
                price,
                table_font_size,
                max_position - price_width,
                position.1 - padding_y - y_offset,
                &pdf.font.font,
            );

            y_offset += pdf.line_height * 1.25 + add_offsets;
        }

        let line = Line {
            points: vec![
                (
                    Point::new(
                        position.0,
                        position.1 - padding_y - y_offset + pdf.line_height,
                    ),
                    false,
                ),
                (
                    Point::new(
                        max_position,
                        position.1 - padding_y - y_offset + pdf.line_height,
                    ),
                    false,
                ),
            ],
            is_closed: false,
        };

        pdf.layer.set_line_height(pdf.line_height.0);

        pdf.layer.set_fill_color(pdf.light_gray.clone());
        pdf.layer.set_outline_color(pdf.light_gray.clone());

        pdf.layer.set_outline_thickness(0.0);

        pdf.layer.add_line(line);

        let line = Line {
            points: vec![
                (
                    Point::new(
                        (max_position - position.0) / 2.0,
                        position.1 - padding_y - y_offset - pdf.line_height * 1.0,
                    ),
                    false,
                ),
                (
                    Point::new(
                        max_position,
                        position.1 - padding_y - y_offset - pdf.line_height * 1.0,
                    ),
                    false,
                ),
            ],
            is_closed: false,
        };

        pdf.layer.set_fill_color(pdf.black.clone());
        pdf.layer.set_outline_color(pdf.black.clone());

        pdf.layer.set_outline_thickness(1.5);

        pdf.layer.add_line(line);

        let total_price = ac.format_money(price_total);

        pdf.layer.use_text(
            &total_price,
            16.0,
            max_position - calculate_text_width(&total_price, pdf, 16.0),
            position.1 - padding_y - y_offset - pdf.line_height * 2.4,
            &pdf.font_bold.font,
        );

        (
            max_position,
            position.1 - padding_y - y_offset - pdf.line_height * 2.4,
        )
    }

    pub fn pdf_draw_spayd(qr_code: String, pdf: &PdfData, position: (Mm, Mm)) {
        let mut image = printpdf::Svg::parse(&qr_code).unwrap();
        image.width = Px(800);
        image.height = Px(800);

        let y_offset = position.1.into_pt() - image.height.into_pt(300.0) / 3.0 * 2.0;

        image.add_to_layer(&pdf.layer, SvgTransform {
            translate_x: Some(position.0.into_pt()),
            translate_y: Some(y_offset),
            rotate: None,
            scale_x: None,
            scale_y: None,
            dpi: Some(300.0),
        });
    }
}

impl From<Invoice> for PdfDocumentReference {
    fn from(value: Invoice) -> Self {
        let price_total = value
            .items
            .iter()
            .fold(dec!(0.0), |acc, item| acc + item.price());

        let (document, page1, layer1) =
            PdfDocument::new("Faktura", Mm(210.0), Mm(297.0), "Layer 1");

        let noto_bytes = include_bytes!("../assets/NotoSans-Regular.ttf");
        let noto_bold_bytes = include_bytes!("../assets/NotoSans-Bold.ttf");

        let noto_metrics = get_font_metrics_freetype(noto_bytes, 0);
        let noto_bold_metrics = get_font_metrics_freetype(noto_bold_bytes, 0);

        let noto = document.add_external_font(noto_bytes.as_ref()).unwrap();

        let noto_bold = document
            .add_external_font(noto_bold_bytes.as_ref())
            .unwrap();

        let pdf_font_noto = PdfFont {
            font: noto,
            metrics: noto_metrics,
            index: 0,
            bytes: noto_bytes,
        };

        let pdf_font_noto_bold = PdfFont {
            font: noto_bold,
            metrics: noto_bold_metrics,
            index: 0,
            bytes: noto_bold_bytes,
        };

        let layer = document.get_page(page1).get_layer(layer1);

        let black = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
        let gray = Color::Rgb(Rgb::new(90.0 / 256.0, 90.0 / 256.0, 90.0 / 256.0, None));
        let light_gray = Color::Rgb(Rgb::new(200.0 / 256.0, 200.0 / 256.0, 200.0 / 256.0, None));

        let pdf_data = PdfData {
            document: &document,
            layer: &layer,
            black: &black,
            gray: &gray,
            light_gray: &light_gray,
            font: &pdf_font_noto,
            font_bold: &pdf_font_noto_bold,
            line_height: Mm(5.0),
        };

        let margin_x = 10.0;

        let left_half = Mm(margin_x);
        let left_half_max = Mm(110.0 - 6.25);

        let right_half = Mm(110.0 + 6.25);
        let right_half_max = Mm(210.0 - margin_x);

        let mut y = Mm(297.0 - 10.5);

        let (_, height) = Invoice::pdf_draw_heading_text(
            &value.number,
            &pdf_data,
            right_half_max,
            (right_half, y),
        );

        y -= height;

        let contractor_rect = Invoice::pdf_draw_entity_info(
            &value.contractor,
            EntityType::Contractor,
            &pdf_data,
            left_half_max,
            (left_half, y),
        );

        let client_rect = Invoice::pdf_draw_entity_info(
            &value.client,
            EntityType::Client,
            &pdf_data,
            right_half_max,
            (right_half, y),
        );

        y -= contractor_rect.1.max(client_rect.1) + pdf_data.line_height;

        let (_, payment_method_h) = Invoice::pdf_draw_payment_method(
            &value.iban,
            &value.payment_method,
            &pdf_data,
            left_half_max,
            (left_half, y),
        );

        let (_, dates_h) = Invoice::pdf_draw_dates(
            &value.date,
            &value.due_date,
            &pdf_data,
            right_half_max,
            (right_half, y),
        );

        y -= dates_h.max(payment_method_h);

        let (_, items_h) = Invoice::pdf_draw_items(
            &value.items,
            value.currency,
            price_total,
            &pdf_data,
            right_half_max,
            (left_half, y),
        );

        y -= items_h;

        if let PaymentMethod::BankTransfer(symbol) = &value.payment_method {
            let spayd = Spayd::new_v1_0([
                (spayd::fields::ACCOUNT, &value.iban.electronic_str().to_string()),
                (spayd::fields::AMOUNT, &price_total.to_string()),
                (spayd::fields::CURRENCY, &value.currency.code().to_string()),
                ("X-VS", symbol),
            ]);

            if let Ok(qr) = qr::QRBuilder::new(spayd.to_string()).build() {
                let img = SvgBuilder::default()
                    .shape(Shape::RoundedSquare)
                    .background_color([255, 255, 255, 0])
                    .to_str(&qr);

                Invoice::pdf_draw_spayd(img, &pdf_data, (left_half, y));
            }

            if let Some(note) = value.note {
                pdf_data.layer.use_text(note, 8.0, left_half, Mm(6.0), &pdf_data.font.font);
            }
        }

        document
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
            .replace(' ', "")
            .trim_start_matches('0')
            .to_string();
        format!("{}/{}", account_number, bank_code)
    }
}
