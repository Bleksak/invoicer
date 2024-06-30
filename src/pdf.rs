use azul_text_layout::text_layout::FontMetrics;
use azul_text_layout::text_layout::{split_text_into_words, words_to_scaled_words};
use printpdf::{Color, IndirectFontRef, Mm, PdfDocumentReference, PdfLayerReference, Pt};

pub struct PdfFont<'bytes> {
    pub font: IndirectFontRef,
    pub metrics: FontMetrics,
    pub index: u32,
    pub bytes: &'bytes [u8],
}

pub struct PdfData<'pdf, 'black, 'gray, 'light_gray> {
    pub document: &'pdf PdfDocumentReference,
    pub layer: &'pdf PdfLayerReference,
    pub black: &'black Color,
    pub gray: &'gray Color,
    pub light_gray: &'light_gray Color,
    pub font: &'pdf PdfFont<'pdf>,
    pub font_bold: &'pdf PdfFont<'pdf>,
    pub line_height: Mm,
}

pub fn calculate_text_width(text: &str, pdf_data: &PdfData, font_size: f32) -> Mm {
    let words = split_text_into_words(text);
    let scaled_words = words_to_scaled_words(
        &words,
        pdf_data.font.bytes,
        pdf_data.font.index,
        pdf_data.font.metrics,
        font_size,
    );

    let total_width: f32 = (scaled_words.items.len() - 1) as f32 * scaled_words.space_advance_px
        + scaled_words.items.iter().map(|i| i.word_width).sum::<f32>();

    Mm::from(Pt(total_width))
}
