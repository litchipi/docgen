use crate::{config::ConfigStore, data::Transaction, lang::LangDict};

pub fn sanitize(data: &str) -> String {
    data.replace('@', "\\@").replace('#', "\\#")
}

pub fn write_page_settings(buffer: &mut String, footer: &str) {
    *buffer += format!(
        "#set page(
        paper: paper_type(),
        margin: (top: margin_top(), x: margin_x(), bottom: margin_bottom()),
        footer: [
            #text(footer_font_size())[#h(1fr) {footer} #h(1fr)]
        ])\n"
    )
    .as_str();

    *buffer += "#set text(font: font_name(), font_size())\n";
    *buffer += "#let sep_par() = 28pt\n";
}

pub fn generate_header(cfg: &ConfigStore, source: &mut String) {
    let logo_path = cfg.get_company("logo_path");
    let logo = if logo_path.is_empty() {
        "".to_string()
    } else {
        format!("#image(\"{logo_path}\", width: logo_width())")
    };

    let writing_logo_path = cfg.get_company("logo_writing");
    let writing_logo = if writing_logo_path.is_empty() {
        format!(
            "#text(company_name_font_size())[*{}*]",
            cfg.get_company("name")
        )
    } else {
        format!("#image(\"{writing_logo_path}\", width: logo_width())")
    };

    *source += format!(
        "#grid(
        columns: (1fr, auto),
        align(left)[{writing_logo}],
        align(right)[{logo}]
    )\n"
    )
    .as_str();

    *source += format!(
        "#align(left)[
        {} \\ {} \\ {} \\ {} \\ SIRET: {}
    ]\n",
        sanitize(&cfg.get_company("person_name")),
        sanitize(&cfg.get_company("address")),
        sanitize(&cfg.get_company("email")),
        sanitize(&cfg.get_company("legal_status")),
        sanitize(&cfg.get_company("siret_number")),
    )
    .as_str();
    *source += "\n";
}

pub fn generate_transaction_table(source: &mut String, tx: &[Transaction], lang: &LangDict) -> f64 {
    let word_desc = lang.get_doctype_word("general", "tx_item_description");
    let word_units = lang.get_doctype_word("general", "tx_units");
    let word_ppu = lang.get_doctype_word("general", "tx_price_per_unit");
    let word_total = lang.get_doctype_word("general", "total_price_no_tax");
    let curr_sym = lang.get_doctype_word("general", "currency_symbol");
    *source += format!(
        "#table(
        stroke: table_color(),
        columns: (tx_descr_width(), 1fr, 1fr, 1fr),
        [*{word_desc}*], [*{word_units}*], [*{word_ppu}*], [*{word_total}*],
    "
    )
    .as_str();

    let mut total_price = 0.0;
    for (descr, units, ppu) in tx.iter() {
        let total = units * ppu;
        *source += format!(
            "
            \"{descr}\", \"{units}\", \"{ppu:.2} {curr_sym}\", \"{total:.2} {curr_sym}\",
        "
        )
        .as_str();
        total_price += total;
    }

    *source += ")\n";
    total_price
}

pub fn generate_summary_table(
    source: &mut String,
    total_price: f64,
    lang: &LangDict,
    cfg: &ConfigStore,
) {
    let curr_sym = lang.get_doctype_word("general", "currency_symbol");
    let (tax_fmt, tax_amnt) = if cfg.get_bool("taxes", "tax_applicable") {
        let tax_rate: f64 = cfg.get_float("taxes", "tax_rate");
        let amnt = total_price * tax_rate;
        (
            format!(
                "[*{} {:.2}%*], [{:.2} {curr_sym}]",
                lang.get_doctype_word("general", "tax_name"),
                tax_rate * 100.0,
                amnt,
            ),
            amnt,
        )
    } else {
        (
            format!(
                "[*{}*], []",
                lang.get_doctype_word("general", "tax_not_applicable")
            ),
            0.0,
        )
    };

    *source += format!(
        "#table(
        stroke: table_color(),
        columns: (auto, auto),
        [*{}*], [{total_price:.2} {curr_sym}],
        {tax_fmt},
        [*{}*], [{:.2} {curr_sym}],
    )",
        lang.get_doctype_word("general", "total_price_no_tax"),
        lang.get_doctype_word("general", "total_price_with_tax"),
        total_price + tax_amnt
    )
    .as_str();
    *source += "\n";
}

pub fn generate_iban(source: &mut String, lang: &LangDict, cfg: &ConfigStore) {
    *source += format!(
        "
        === {}

        #table(
            stroke: table_color(),
            columns: (auto, auto),
            [*{}*], [{}],
            [*IBAN*], [{}],
            [*BIC*], [{}],
        )",
        lang.get_doctype_word("general", "iban_title"),
        lang.get_doctype_word("general", "iban_bank"),
        cfg.get_str("bank", "name"),
        cfg.get_str("bank", "iban"),
        cfg.get_str("bank", "bic"),
    )
    .as_str();
    *source += "\n";
}
