use ammonia::Builder;
use comrak::{markdown_to_html, Options};

pub fn render_markdown(input: &str) -> String {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.header_ids = Some(String::new());
    options.render.unsafe_ = false;

    markdown_to_html(input, &options)
}

pub fn calculate_reading_time(text: &str) -> u32 {
    let words = text.split_whitespace().count();
    let time = (words as f64 / 200.0).ceil() as u32;
    if time == 0 {
        1
    } else {
        time
    }
}

pub fn extract_first_paragraph(markdown: &str) -> String {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('!') && !trimmed.starts_with('[') {
            return trimmed.to_string();
        }
    }
    String::new()
}

pub fn sanitize_html(html: &str) -> String {
    let mut builder = Builder::default();
    
    let tags = ["p", "br", "strong", "em", "a", "code", "pre", "ul", "ol", "li"];
    builder.tags(tags.iter().cloned().collect());
    
    builder.link_rel(Some("nofollow"));
    
    builder.clean(html).to_string()
}
