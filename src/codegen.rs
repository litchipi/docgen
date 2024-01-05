pub fn sanitize(data: &String) -> String {
    data.replace("@", "\\@").replace("#", "\\#")
}

pub fn write_page_settings(buffer: &mut String) {
    *buffer += "#set page(paper: paper_type(), margin: margin_size())\n";
    *buffer += "#set text(font: font_name(), font_size())\n";
}
