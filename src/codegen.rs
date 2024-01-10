pub fn sanitize(data: &str) -> String {
    data.replace('@', "\\@").replace('#', "\\#")
}

pub fn write_page_settings(buffer: &mut String, footer: String) {
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
}
