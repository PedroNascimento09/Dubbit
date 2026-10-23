use std::path::Path;

use crate::error::Result;
use crate::files::atomic_write;
use crate::segments::Segment;

pub const MIN_CUE: f64 = 0.4;
pub const MAX_LINE: usize = 42;
pub const MAX_CUE_LINES: usize = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

pub fn timestamp(seconds: f64, separator: char) -> String {
    let total_ms = (seconds.max(0.0) * 1000.0).round() as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let whole_seconds = (total_ms % 60_000) / 1000;
    let milliseconds = total_ms % 1000;
    format!("{hours:02}:{minutes:02}:{whole_seconds:02}{separator}{milliseconds:03}")
}

pub fn flatten(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

pub fn wrap_text(text: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.chars().count() + 1 + word.chars().count() <= max {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(std::mem::take(&mut current));
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

fn sentences(text: &str) -> Vec<String> {
    let mut units = Vec::new();
    let mut current = String::new();
    let characters: Vec<char> = text.chars().collect();
    for (index, character) in characters.iter().enumerate() {
        if character.is_whitespace() && current.trim().is_empty() {
            continue;
        }
        current.push(*character);
        let ends_sentence = matches!(character, '.' | '!' | '?' | '\u{2026}');
        let next_is_space = characters
            .get(index + 1)
            .map(|next| next.is_whitespace())
            .unwrap_or(true);
        if ends_sentence && next_is_space {
            units.push(flatten(&current));
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        units.push(flatten(&current));
    }
    units
}

fn split_lines(lines: Vec<String>) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for line in lines {
        current.push(line);
        if current.len() == MAX_CUE_LINES {
            blocks.push(current.join("\n"));
            current.clear();
        }
    }
    if !current.is_empty() {
        blocks.push(current.join("\n"));
    }
    blocks
}

pub fn cues(segments: &[Segment], translated: bool) -> Vec<Cue> {
    let mut cues = Vec::new();
    for segment in segments {
        let text = flatten(segment.line(translated));
        if text.is_empty() {
            continue;
        }
        let mut blocks = Vec::new();
        for sentence in sentences(&text) {
            blocks.extend(split_lines(wrap_text(&sentence, MAX_LINE)));
        }
        let total_chars: usize = blocks
            .iter()
            .map(|block| block.chars().filter(|item| *item != '\n').count())
            .sum();
        if total_chars == 0 {
            continue;
        }
        let duration = segment.end - segment.start;
        let mut consumed = 0_usize;
        for block in blocks {
            let chars = block.chars().filter(|item| *item != '\n').count();
            let start = segment.start + duration * (consumed as f64 / total_chars as f64);
            consumed += chars;
            let end = segment.start + duration * (consumed as f64 / total_chars as f64);
            let end = end
                .max(start + MIN_CUE)
                .min(segment.end.max(start + MIN_CUE));
            cues.push(Cue {
                start,
                end,
                text: block,
            });
        }
    }
    cues
}

pub fn render_srt(segments: &[Segment], translated: bool) -> String {
    let mut blocks = Vec::new();
    for (number, cue) in cues(segments, translated).into_iter().enumerate() {
        blocks.push(format!(
            "{}\n{} --> {}\n{}\n",
            number + 1,
            timestamp(cue.start, ','),
            timestamp(cue.end, ','),
            cue.text
        ));
    }
    blocks.join("\n")
}

pub fn render_vtt(segments: &[Segment], translated: bool) -> String {
    let mut blocks = vec!["WEBVTT\n".to_string()];
    for cue in cues(segments, translated) {
        blocks.push(format!(
            "{} --> {}\n{}\n",
            timestamp(cue.start, '.'),
            timestamp(cue.end, '.'),
            cue.text
        ));
    }
    blocks.join("\n")
}

pub fn write_srt(path: &Path, segments: &[Segment], translated: bool) -> Result<()> {
    atomic_write(path, render_srt(segments, translated).as_bytes())
}

pub fn write_vtt(path: &Path, segments: &[Segment], translated: bool) -> Result<()> {
    atomic_write(path, render_vtt(segments, translated).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_timestamps_at_boundaries() {
        assert_eq!(timestamp(0.0, ','), "00:00:00,000");
        assert_eq!(timestamp(3661.5, ','), "01:01:01,500");
        assert_eq!(timestamp(59.9996, ','), "00:01:00,000");
        assert_eq!(timestamp(1.25, '.'), "00:00:01.250");
    }

    #[test]
    fn srt_skips_empty_lines() {
        let segments = vec![
            Segment {
                index: 0,
                start: 0.5,
                end: 2.0,
                text: "Hello".into(),
                translation: "Ola".into(),
            },
            Segment::new(1, 2.0, 3.0, "Skip"),
        ];
        let rendered = render_srt(&segments, true);
        assert!(rendered.contains("1\n00:00:00,500 --> 00:00:02,000\nOla\n"));
        assert!(!rendered.contains("Skip"));
    }

    #[test]
    fn vtt_starts_with_header() {
        let segments = vec![Segment {
            index: 0,
            start: 0.0,
            end: 1.2,
            text: "Hello".into(),
            translation: "Ola".into(),
        }];
        let rendered = render_vtt(&segments, true);
        assert!(rendered.starts_with("WEBVTT"));
        assert!(rendered.contains("00:00:00.000 --> 00:00:01.200"));
    }

    #[test]
    fn short_cues_are_extended() {
        let rendered = render_srt(&[Segment::new(0, 0.0, 0.1, "Hi")], false);
        assert!(rendered.contains("00:00:00,000 --> 00:00:00,400"));
    }

    #[test]
    fn long_speech_is_split_without_loss() {
        let text = "Hello there! This is a test of the subtitle pipeline. It should produce two subtitle files without dropping a single word from the transcript.";
        let segments = vec![Segment::new(0, 0.0, 9.0, text)];
        let cues = cues(&segments, false);
        assert!(cues.len() > 1);
        let joined = cues
            .iter()
            .map(|cue| flatten(&cue.text))
            .collect::<Vec<String>>()
            .join(" ");
        assert_eq!(joined, flatten(text));
        for cue in &cues {
            assert!(cue.end > cue.start);
            assert!(cue.end <= segments[0].end);
            assert!(cue
                .text
                .lines()
                .all(|line| line.chars().count() <= MAX_LINE));
            assert!(cue.text.lines().count() <= MAX_CUE_LINES);
        }
    }

    #[test]
    fn wrapping_respects_word_boundaries() {
        let lines = wrap_text("one two three four five six seven", 10);
        assert_eq!(lines, vec!["one two", "three four", "five six", "seven"]);
    }
}
