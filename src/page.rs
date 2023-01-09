use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref TITLE: Regex = Regex::new(r"#\s+(.+)").unwrap();
    static ref LINK: Regex = Regex::new(r"\[(.*?)\]\(([bu]):(.+?)\)").unwrap();
    static ref SUMMARY: Regex = RegexBuilder::new(r"\s*(.+?)$")
                                    .multi_line(true).build().unwrap();
}

#[derive(Debug, PartialEq, Eq)]
pub struct Page {
    pub text: String,
    pub summary: Option<String>,
    pub links: Vec<Link>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
    pub target: String,
    pub bidirectional: bool,
    pub text: String,
}

impl Page {
    pub fn from(text: &str) -> Self {
        let summary = SUMMARY.captures(text)
            .map(|c| c[1].trim().to_string());
        let links: Vec<Link> = LINK.captures_iter(text)
            .map(|c| Link {
                text: if !c[1].trim().is_empty() {
                    c[1].trim().to_string()
                } else {
                    c[3].trim().to_string()
                },
                target: c[3].trim().to_string(),
                bidirectional: c[2].trim() == "b",
            }).collect();
        Page { text: text.to_string(), summary, links }
    }

    /// Only keep links with unique source-target pairs. Link text is of any of the links with same
    /// source-target pair, no guarantees about which one.
    ///
    /// Determine direction for each pair: if a source page has both uni- and bidirectional links to
    /// the same target page, keep one bidirectional link.
    pub fn unique(mut page: Page) -> Self {
        if page.links.len() < 2 {
            return page;
        }
        // TODO: probably there is a more elegant way
        page.links.sort_by(|a, b| b.cmp(a));
        let mut links = Vec::with_capacity(page.links.len());
        let mut prev_target = "".to_string();
        for link in page.links {
            if link.target != prev_target {
                prev_target = link.target.to_string();
                links.push(link);
            }
        }
        Page {
            text: page.text,
            summary: page.summary,
            links,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::page::{Link, Page};

    #[test]
    fn parse_page() {
        let markdown = r#"
        This is a test page.

        Some more text with a [Unidirectional Link](u:Target Title) and a
        [Bidirectional Link](b:Target Title).

        [](u:Test page)
        "#;
        let page = Page::from(markdown);

        assert_eq!("This is a test page.", page.summary.unwrap());
        assert_eq!(Link {
            text: "Unidirectional Link".to_string(),
            target: "Target Title".to_string(),
            bidirectional: false,
        }, page.links[0]);
        assert_eq!(Link {
            text: "Bidirectional Link".to_string(),
            target: "Target Title".to_string(),
            bidirectional: true,
        }, page.links[1]);
        assert_eq!(Link {
            text: "Test page".to_string(),
            target: "Test page".to_string(),
            bidirectional: false,
        }, page.links[2]);
    }

    #[test]
    fn filter_unique_links() {
        let markdown = r#"
        [uA](u:A) [bA](b:A) [bzA](b:A) [uA](u:A)
        [uB](u:B) [uB](u:B)
        "#;

        let page = Page::unique(Page::from(markdown));

        assert_eq!(Link {
            text: "uB".to_string(),
            target: "B".to_string(),
            bidirectional: false,
        }, page.links[0]);

        assert_eq!(Link {
            text: "bzA".to_string(),
            target: "A".to_string(),
            bidirectional: true,
        }, page.links[1]);
    }
}