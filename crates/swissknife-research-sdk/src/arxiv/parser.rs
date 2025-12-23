use crate::{Author, Error, Paper, Result};

pub fn parse_atom_feed(xml: &str) -> Result<Vec<Paper>> {
    let mut papers = Vec::new();

    let entries = extract_entries(xml);
    for entry in entries {
        if let Some(paper) = parse_entry(&entry) {
            papers.push(paper);
        }
    }

    Ok(papers)
}

pub fn parse_total_results(xml: &str) -> Option<u32> {
    extract_tag_content(xml, "opensearch:totalResults")
        .and_then(|s| s.parse().ok())
}

pub fn parse_start_index(xml: &str) -> u32 {
    extract_tag_content(xml, "opensearch:startIndex")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn parse_items_per_page(xml: &str) -> u32 {
    extract_tag_content(xml, "opensearch:itemsPerPage")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn extract_entries(xml: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut remaining = xml;

    while let Some(start_idx) = remaining.find("<entry>") {
        if let Some(end_idx) = remaining[start_idx..].find("</entry>") {
            let entry_end = start_idx + end_idx + "</entry>".len();
            entries.push(remaining[start_idx..entry_end].to_string());
            remaining = &remaining[entry_end..];
        } else {
            break;
        }
    }

    entries
}

fn parse_entry(entry: &str) -> Option<Paper> {
    let id = extract_tag_content(entry, "id")?;
    let arxiv_id = extract_arxiv_id(&id);

    let title = extract_tag_content(entry, "title")
        .map(|s| normalize_whitespace(&s))?;

    let summary = extract_tag_content(entry, "summary")
        .map(|s| normalize_whitespace(&s));

    let published = extract_tag_content(entry, "published");
    let updated = extract_tag_content(entry, "updated");

    let authors = extract_authors(entry);
    let categories = extract_categories(entry);
    let primary_category = extract_primary_category(entry);

    let pdf_url = extract_pdf_link(entry);
    let html_url = Some(format!("https://arxiv.org/abs/{}", arxiv_id));

    let doi = extract_tag_content(entry, "arxiv:doi");
    let journal_ref = extract_tag_content(entry, "arxiv:journal_ref");
    let comment = extract_tag_content(entry, "arxiv:comment");

    Some(Paper {
        id: arxiv_id,
        title,
        authors,
        summary,
        published,
        updated,
        categories,
        primary_category,
        pdf_url,
        html_url,
        doi,
        journal_ref,
        comment,
    })
}

fn extract_tag_content(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);

    if let Some(start_idx) = xml.find(&open_tag) {
        let after_open = &xml[start_idx + open_tag.len()..];
        if let Some(content_start) = after_open.find('>') {
            let content = &after_open[content_start + 1..];
            if let Some(end_idx) = content.find(&close_tag) {
                return Some(content[..end_idx].trim().to_string());
            }
        }
    }
    None
}

fn extract_arxiv_id(url: &str) -> String {
    url.rsplit('/')
        .next()
        .map(|s| s.replace("v", ".v"))
        .unwrap_or_else(|| url.to_string())
        .replace(".v", "v")
}

fn extract_authors(entry: &str) -> Vec<Author> {
    let mut authors = Vec::new();
    let mut remaining = entry;

    while let Some(start_idx) = remaining.find("<author>") {
        if let Some(end_idx) = remaining[start_idx..].find("</author>") {
            let author_end = start_idx + end_idx + "</author>".len();
            let author_xml = &remaining[start_idx..author_end];

            if let Some(name) = extract_tag_content(author_xml, "name") {
                let affiliation = extract_tag_content(author_xml, "arxiv:affiliation");
                authors.push(Author { name, affiliation });
            }

            remaining = &remaining[author_end..];
        } else {
            break;
        }
    }

    authors
}

fn extract_categories(entry: &str) -> Vec<String> {
    let mut categories = Vec::new();
    let mut remaining = entry;

    while let Some(start_idx) = remaining.find("<category") {
        if let Some(end_idx) = remaining[start_idx..].find("/>") {
            let cat_end = start_idx + end_idx + "/>".len();
            let cat_xml = &remaining[start_idx..cat_end];

            if let Some(term_start) = cat_xml.find("term=\"") {
                let term_content = &cat_xml[term_start + 6..];
                if let Some(term_end) = term_content.find('"') {
                    categories.push(term_content[..term_end].to_string());
                }
            }

            remaining = &remaining[cat_end..];
        } else {
            break;
        }
    }

    categories
}

fn extract_primary_category(entry: &str) -> Option<String> {
    if let Some(start_idx) = entry.find("<arxiv:primary_category") {
        if let Some(end_idx) = entry[start_idx..].find("/>") {
            let cat_xml = &entry[start_idx..start_idx + end_idx];
            if let Some(term_start) = cat_xml.find("term=\"") {
                let term_content = &cat_xml[term_start + 6..];
                if let Some(term_end) = term_content.find('"') {
                    return Some(term_content[..term_end].to_string());
                }
            }
        }
    }
    None
}

fn extract_pdf_link(entry: &str) -> Option<String> {
    let mut remaining = entry;

    while let Some(start_idx) = remaining.find("<link") {
        if let Some(end_idx) = remaining[start_idx..].find("/>").or_else(|| remaining[start_idx..].find('>')) {
            let link_end = start_idx + end_idx + if remaining[start_idx + end_idx..].starts_with("/>") { 2 } else { 1 };
            let link_xml = &remaining[start_idx..link_end];

            if link_xml.contains("title=\"pdf\"") {
                if let Some(href_start) = link_xml.find("href=\"") {
                    let href_content = &link_xml[href_start + 6..];
                    if let Some(href_end) = href_content.find('"') {
                        return Some(href_content[..href_end].to_string());
                    }
                }
            }

            remaining = &remaining[link_end..];
        } else {
            break;
        }
    }

    None
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
