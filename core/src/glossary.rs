use std::fs;
use std::path::Path;

use csv::ReaderBuilder;

use crate::error::{invalid_option, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Default)]
pub struct Glossary {
    terms: Vec<Term>,
}

fn same_char(left: char, right: char) -> bool {
    left.to_lowercase().eq(right.to_lowercase())
}

fn replace_ci(haystack: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return haystack.to_string();
    }
    let needle_chars: Vec<char> = needle.chars().collect();
    let haystack_chars: Vec<char> = haystack.chars().collect();
    let mut output = String::with_capacity(haystack.len());
    let mut index = 0;
    while index < haystack_chars.len() {
        let end = index + needle_chars.len();
        let matches = end <= haystack_chars.len()
            && haystack_chars[index..end]
                .iter()
                .zip(needle_chars.iter())
                .all(|(left, right)| same_char(*left, *right));
        if matches {
            output.push_str(replacement);
            index = end;
        } else {
            output.push(haystack_chars[index]);
            index += 1;
        }
    }
    output
}

impl Glossary {
    pub fn new(mut terms: Vec<Term>) -> Self {
        terms.sort_by(|left, right| {
            right
                .source
                .chars()
                .count()
                .cmp(&left.source.chars().count())
        });
        Self { terms }
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn protect(&self, text: &str) -> (String, Vec<(String, String)>) {
        let mut result = text.to_string();
        let mut used = Vec::new();
        for (position, term) in self.terms.iter().enumerate() {
            if term.source.is_empty() {
                continue;
            }
            let token = format!("ZQX{position}QXZ");
            let replaced = replace_ci(&result, &term.source, &token);
            if replaced != result {
                used.push((token, term.target.clone()));
                result = replaced;
            }
        }
        (result, used)
    }

    pub fn restore(&self, text: &str, used: &[(String, String)]) -> String {
        let mut result = text.to_string();
        for (token, target) in used {
            result = replace_ci(&result, token, target);
        }
        self.apply(&result)
    }

    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();
        for term in &self.terms {
            if term.source.is_empty() {
                continue;
            }
            result = replace_ci(&result, &term.source, &term.target);
        }
        result
    }
}

pub fn load(path: &Path) -> Result<Glossary> {
    let text = fs::read_to_string(path).map_err(|error| {
        invalid_option(
            "glossary",
            &format!("{} ({error})", path.display()),
            "a readable CSV file",
        )
    })?;
    let text = text.trim_start_matches('\u{feff}');
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut terms = Vec::new();
    let mut first_data_row = true;
    for (offset, record) in reader.records().enumerate() {
        let line = offset + 1;
        let record = record.map_err(|error| {
            invalid_option(
                "glossary",
                &format!("line {line}"),
                "two columns: source term, target term",
            )
            .detail(error.to_string())
        })?;
        let source = record.get(0).unwrap_or("").trim();
        if source.is_empty() {
            continue;
        }
        if source.starts_with('#') {
            continue;
        }
        if first_data_row {
            first_data_row = false;
            let lowered = source.to_lowercase();
            if matches!(lowered.as_str(), "source" | "term" | "from" | "original") {
                continue;
            }
        }
        let target = record.get(1).unwrap_or("").trim();
        if target.is_empty() {
            return Err(invalid_option(
                "glossary",
                &format!("line {line}"),
                "two columns: source term, target term",
            ));
        }
        terms.push(Term {
            source: source.to_string(),
            target: target.to_string(),
        });
    }
    Ok(Glossary::new(terms))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(source: &str, target: &str) -> Term {
        Term {
            source: source.to_string(),
            target: target.to_string(),
        }
    }

    #[test]
    fn protection_is_case_insensitive() {
        let glossary = Glossary::new(vec![term("cloud", "nuvem"), term("Aurora", "Aurora")]);
        let (protected, used) = glossary.protect("The CLOUD belongs to Aurora");
        assert!(!protected.contains("CLOUD"));
        assert!(!protected.contains("Aurora"));
        assert_eq!(
            glossary.restore(&protected, &used),
            "The nuvem belongs to Aurora"
        );
    }

    #[test]
    fn longest_term_wins() {
        let glossary = Glossary::new(vec![
            term("New York", "Nova Iorque"),
            term("York", "Iorque"),
        ]);
        let (protected, used) = glossary.protect("New York");
        assert_eq!(protected, "ZQX0QXZ");
        assert_eq!(glossary.restore(&protected, &used), "Nova Iorque");
    }

    #[test]
    fn restore_handles_leftover_terms() {
        let glossary = Glossary::new(vec![term("cloud", "nuvem")]);
        let (protected, used) = glossary.protect("cloud and cloud");
        let restored = glossary.restore(&format!("{protected} cloud"), &used);
        assert_eq!(restored, "nuvem and nuvem nuvem");
    }

    #[test]
    fn protection_is_utf8_safe() {
        let glossary = Glossary::new(vec![term("ação", "action")]);
        let (protected, used) = glossary.protect("AÇÃO final");
        assert_eq!(protected, "ZQX0QXZ final");
        assert_eq!(glossary.restore(&protected, &used), "action final");
    }

    #[test]
    fn load_skips_header_and_comments() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("terms.csv");
        fs::write(&path, "source,target\n# note\ncloud,nuvem\nAurora,Aurora\n").unwrap();
        let glossary = load(&path).unwrap();
        let (protected, used) = glossary.protect("The cloud over Aurora");
        assert_eq!(glossary.restore(&protected, &used), "The nuvem over Aurora");
    }

    #[test]
    fn load_accepts_utf8_bom_and_quotes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("terms.csv");
        fs::write(
            &path,
            "\u{feff}Aurora,Nova Aurora\n\"two words\",duas palavras\n",
        )
        .unwrap();
        let glossary = load(&path).unwrap();
        assert_eq!(glossary.apply("two words"), "duas palavras");
        assert_eq!(glossary.apply("Aurora"), "Nova Aurora");
    }

    #[test]
    fn invalid_row_is_reported() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("terms.csv");
        fs::write(&path, "cloud\n").unwrap();
        let error = load(&path).unwrap_err();
        assert_eq!(error.code, "DUB-0001");
        assert!(error.detail.unwrap().contains("line 1"));
    }

    #[test]
    fn missing_file_is_reported() {
        let directory = tempfile::tempdir().unwrap();
        let error = load(&directory.path().join("missing.csv")).unwrap_err();
        assert_eq!(error.code, "DUB-0001");
    }

    #[test]
    fn empty_glossary_keeps_text() {
        let glossary = Glossary::default();
        assert!(glossary.is_empty());
        assert_eq!(glossary.apply("unchanged"), "unchanged");
    }
}
