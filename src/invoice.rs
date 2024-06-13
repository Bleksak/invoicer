use std::fmt::Display;

use crate::{
    entity::{Entity, EntityType},
    payment_method::PaymentMethod,
    pdf::{calculate_text_width, PdfData, PdfFont},
    time::Time,
};
use azul_text_layout::text_shaping::get_font_metrics_freetype;
use chrono::NaiveDate;
use iban::Iban;
use printpdf::{Color, Line, Mm, PdfDocument, PdfDocumentReference, Point, Rgb, TextRenderingMode};
use rust_decimal::Decimal;
use serde::Serialize;

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
}

#[derive(Debug)]
pub struct Invoice {
    number: Decimal,
    contractor: Entity,
    client: Entity,
    payment_method: PaymentMethod,
    items: Vec<InvoiceItem>,
    date: NaiveDate,
    due_date: NaiveDate,
}

impl Invoice {
    pub fn new(
        number: Decimal,
        contractor: Entity,
        client: Entity,
        payment_method: PaymentMethod,
        items: Vec<InvoiceItem>,
        date: NaiveDate,
        due_date: NaiveDate,
    ) -> Self {
        Self {
            number,
            contractor,
            client,
            payment_method,
            items,
            date,
            due_date,
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
            max_position - calculate_text_width(&identifier, pdf),
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
                max_position - calculate_text_width(vat, pdf),
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
        payment_method: &PaymentMethod,
        pdf: &PdfData,
        max_position: Mm,
        position: (Mm, Mm),
    ) -> (Mm, Mm) {
        pdf.layer.set_fill_color(pdf.gray.clone());

        match payment_method {
            PaymentMethod::Cash => todo!(),
            PaymentMethod::Card(_) => todo!(),
            PaymentMethod::BankTransfer(iban, var_symbol) => {
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
                    max_position - calculate_text_width(&bank_account, pdf),
                    position.1 - pdf.line_height,
                    &pdf.font.font,
                );
                pdf.layer.use_text(
                    var_symbol,
                    10.0,
                    max_position - calculate_text_width(var_symbol, pdf),
                    position.1 - pdf.line_height * 2.0,
                    &pdf.font.font,
                );
                pdf.layer.use_text(
                    r#type,
                    10.0,
                    max_position - calculate_text_width(r#type, pdf),
                    position.1 - pdf.line_height * 3.0,
                    &pdf.font.font,
                );
            }
        }

        (Mm(0.0), Mm(0.0))
    }
}

impl From<Invoice> for PdfDocumentReference {
    fn from(value: Invoice) -> Self {
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

        let pdf_data = PdfData {
            document: &document,
            layer: &layer,
            black: &black,
            gray: &gray,
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

        Invoice::pdf_draw_payment_method(
            &value.payment_method,
            &pdf_data,
            left_half_max,
            (left_half, y),
        );

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
