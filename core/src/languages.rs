use crate::error::{unsupported_language, Result};

pub const LANGUAGES: &[(&str, &str)] = &[
    ("af", "Afrikaans"),
    ("ar", "Arabic"),
    ("az", "Azerbaijani"),
    ("be", "Belarusian"),
    ("bg", "Bulgarian"),
    ("bn", "Bengali"),
    ("bs", "Bosnian"),
    ("ca", "Catalan"),
    ("cs", "Czech"),
    ("cy", "Welsh"),
    ("da", "Danish"),
    ("de", "German"),
    ("el", "Greek"),
    ("en", "English"),
    ("es", "Spanish"),
    ("et", "Estonian"),
    ("eu", "Basque"),
    ("fa", "Persian"),
    ("fi", "Finnish"),
    ("fil", "Filipino"),
    ("fr", "French"),
    ("gl", "Galician"),
    ("he", "Hebrew"),
    ("hi", "Hindi"),
    ("hr", "Croatian"),
    ("hu", "Hungarian"),
    ("hy", "Armenian"),
    ("id", "Indonesian"),
    ("is", "Icelandic"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ka", "Georgian"),
    ("kk", "Kazakh"),
    ("kn", "Kannada"),
    ("ko", "Korean"),
    ("lt", "Lithuanian"),
    ("lv", "Latvian"),
    ("mk", "Macedonian"),
    ("ml", "Malayalam"),
    ("mn", "Mongolian"),
    ("mr", "Marathi"),
    ("ms", "Malay"),
    ("mt", "Maltese"),
    ("ne", "Nepali"),
    ("nl", "Dutch"),
    ("no", "Norwegian"),
    ("pa", "Punjabi"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("sq", "Albanian"),
    ("sr", "Serbian"),
    ("sv", "Swedish"),
    ("sw", "Swahili"),
    ("ta", "Tamil"),
    ("te", "Telugu"),
    ("th", "Thai"),
    ("tr", "Turkish"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("vi", "Vietnamese"),
    ("zh", "Chinese"),
];

fn clean(code: &str) -> String {
    let lowered = code.trim().to_lowercase();
    let mut result = String::new();
    let mut last_dash = false;
    for character in lowered.chars() {
        if character == '_' || character.is_whitespace() || character == '-' {
            if !result.is_empty() && !last_dash {
                result.push('-');
                last_dash = true;
            }
        } else {
            result.push(character);
            last_dash = false;
        }
    }
    while result.ends_with('-') {
        result.pop();
    }
    result
}

pub fn base_code(code: &str) -> &str {
    code.split('-').next().unwrap_or(code)
}

pub fn normalize(code: &str) -> Result<String> {
    let cleaned = clean(code);
    if cleaned.is_empty() || !LANGUAGES.iter().any(|(key, _)| *key == base_code(&cleaned)) {
        return Err(unsupported_language(code));
    }
    Ok(cleaned)
}

pub fn display_name(code: &str) -> Result<&'static str> {
    let normalized = normalize(code)?;
    LANGUAGES
        .iter()
        .find(|(key, _)| *key == base_code(&normalized))
        .map(|(_, name)| *name)
        .ok_or_else(|| unsupported_language(code))
}

pub fn matches(left: &str, right: &str) -> bool {
    let left = clean(left);
    let right = clean(right);
    if left == "und" || right == "und" {
        return true;
    }
    base_code(&left) == base_code(&right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_regions_and_separators() {
        assert_eq!(normalize("PT_br").unwrap(), "pt-br");
        assert_eq!(normalize(" en ").unwrap(), "en");
    }

    #[test]
    fn resolves_display_names() {
        assert_eq!(display_name("pt-BR").unwrap(), "Portuguese");
        assert_eq!(display_name("de").unwrap(), "German");
    }

    #[test]
    fn match_ignores_region() {
        assert!(matches("en-US", "en"));
        assert!(matches("pt-br", "pt-PT"));
        assert!(!matches("es-ES", "pt"));
        assert!(matches("und", "pt"));
    }

    #[test]
    fn rejects_unknown_codes() {
        let error = normalize("xx").unwrap_err();
        assert_eq!(error.code, "DUB-2001");
        assert!(normalize("  ").is_err());
    }
}
