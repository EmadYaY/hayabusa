use hashbrown::HashMap;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use lazy_static::lazy_static;
use pulldown_cmark::{html, Options, Parser};
use std::fs::{create_dir, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::RwLock;

lazy_static! {
    pub static ref HTML_REPORTER: RwLock<HtmlReporter> = RwLock::new(HtmlReporter::new());
}

#[derive(Clone)]
pub struct HtmlReporter {
    pub section_order: Vec<String>,
    pub md_datas: HashMap<String, Vec<String>>,
}

impl HtmlReporter {
    pub fn new() -> HtmlReporter {
        let (init_section_order, init_data) = get_init_md_data_map();
        HtmlReporter {
            section_order: init_section_order,
            md_datas: init_data,
        }
    }

    /// return converted String from md_data(markdown fmt string).
    pub fn create_html(self) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_FOOTNOTES);

        let mut md_data = vec![];
        for section_name in self.section_order {
            if let Some(v) = self.md_datas.get(&section_name) {
                md_data.push(format!("## {}\n", &section_name));
                if v.is_empty() {
                    md_data.push("not found data.\n".to_string());
                } else {
                    md_data.push(v.join("\n"));
                }
            }
        }
        let md_str = md_data.join("\n");
        let parser = Parser::new_ext(&md_str, options);

        let mut ret = String::new();
        html::push_html(&mut ret, parser);
        ret
    }
}

impl Default for HtmlReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// get html report section data in LinkedHashMap
fn get_init_md_data_map() -> (Vec<String>, HashMap<String, Vec<String>>) {
    let mut ret = HashMap::new();
    let section_order = vec![
        "General Overview {#general_overview}".to_string(),
        "Results Summary {#results_summary}".to_string(),
    ];
    for section in section_order.iter() {
        ret.insert(section.to_owned(), vec![]);
    }

    (section_order, ret)
}

pub fn add_md_data(section_name: String, data: Vec<String>) {
    let mut md_with_section_data = HTML_REPORTER.write().unwrap().md_datas.clone();
    for c in data {
        let entry = md_with_section_data
            .entry(section_name.clone())
            .or_insert(Vec::new());
        entry.push(c);
    }
    HTML_REPORTER.write().unwrap().md_datas = md_with_section_data;
}

/// create html file
pub fn create_html_file(input_html: String, path_str: String) {
    let path = Path::new(&path_str);
    if !path.parent().unwrap().exists() {
        create_dir(path.parent().unwrap()).ok();
    }

    let mut html_writer = BufWriter::new(File::create(path).unwrap());

    let html_data = format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    meta(charset="UTF-8");
                    link(rel="stylesheet", type="text/css", href="./hayabusa_report.css");
                    link(rel="icon", type="image/png", href="./favicon.png");
                }
                body {
                    section {
                        img(id="logo", src = "./logo.png");
                        : Raw(input_html.as_str());
                    }
                }

            }
        }
    );

    writeln!(html_writer, "{}", html_data).ok();
    println!(
        "HTML Report was generated. Please check {} for details.",
        path_str
    );
    println!();
}

#[cfg(test)]
mod tests {

    use crate::options::htmlreport::HtmlReporter;

    #[test]
    fn test_create_html() {
        let mut html_reporter = HtmlReporter::new();
        let general_data = vec![
            "- Analyzed event files: 581".to_string(),
            "- Total file size: 148.5 MB".to_string(),
            "- Excluded rules: 12".to_string(),
            "- Noisy rules: 5 (Disabled)".to_string(),
            "- Experimental rules: 1935 (65.97%)".to_string(),
            "- Stable rules: 215 (7.33%)".to_string(),
            "- Test rules: 783 (26.70%)".to_string(),
            "- Hayabusa rules: 138".to_string(),
            "- Sigma rules: 2795".to_string(),
            "- Total enabled detection rules: 2933".to_string(),
            "- Elapsed Time: 00:00:29.035".to_string(),
            "".to_string(),
        ];
        html_reporter.md_datas.insert(
            "General Overview {#general_overview}".to_string(),
            general_data.clone(),
        );
        let general_overview_str = format!(
            "<ul>\n<li>{}</li>\n</ul>",
            general_data[..general_data.len() - 1]
                .join("</li>\n<li>")
                .replace("- ", "")
        );
        let expect_str = format!(
            "<h2 id=\"general_overview\">General Overview</h2>\n{}\n<h2 id=\"results_summary\">Results Summary</h2>\n<p>not found data.</p>\n",
            general_overview_str
        );

        assert_eq!(html_reporter.create_html(), expect_str);
    }
}
