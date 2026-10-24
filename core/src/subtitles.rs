use std::path::Path;

use crate::error::Result;
use crate::files::atomic_write;
use crate::segments::Segment;

pub const MIN_CUE: f64 = 1.0;
pub const MAX_LINE: usize = 42;
pub const MAX_CUE_LINES: usize = 2;
pub const READ_RATE: f64 = 17.0;

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

pub fn reading_speed(text: &str, start: f64, end: f64) -> f64 {
    let characters = text.chars().filter(|item| *item != '\n').count() as f64;
    let duration = (end - start).max(0.0);
    if duration <= 0.0 {
        return f64::INFINITY;
    }
    characters / duration
}

fn block_chars(text: &str) -> f64 {
    text.chars().filter(|item| *item != '\n').count() as f64
}

fn allotted(blocks: &[String], duration: f64) -> Vec<f64> {
    let total: f64 = blocks.iter().map(|block| block_chars(block)).sum();
    if total <= 0.0 {
        return Vec::new();
    }
    let mut times: Vec<f64> = blocks
        .iter()
        .map(|block| duration * block_chars(block) / total)
        .collect();
    let shortest = times.iter().cloned().fold(f64::INFINITY, f64::min);
    if shortest < MIN_CUE {
        let share = duration / blocks.len() as f64;
        return vec![share; blocks.len()];
    }
    for time in times.iter_mut() {
        *time = time.max(MIN_CUE);
    }
    let summed: f64 = times.iter().sum();
    if summed > duration {
        let share = duration / blocks.len() as f64;
        return vec![share; blocks.len()];
    }
    let spare = duration - summed;
    let readable: Vec<f64> = blocks
        .iter()
        .map(|block| block_chars(block) / READ_RATE)
        .collect();
    let hungry: f64 = readable
        .iter()
        .zip(times.iter())
        .map(|(need, have)| (need - have).max(0.0))
        .sum();
    if hungry <= 0.0 {
        if let Some(last) = times.last_mut() {
            *last += spare;
        }
        return times;
    }
    for (time, need) in times.iter_mut().zip(readable.iter()) {
        let gap = (need - *time).max(0.0);
        *time += spare * gap / hungry;
    }
    times
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
        let duration = segment.end - segment.start;
        let times = allotted(&blocks, duration);
        if times.is_empty() {
            continue;
        }
        let mut cursor = segment.start;
        for (block, time) in blocks.into_iter().zip(times) {
            let start = cursor;
            let end = (start + time).min(segment.end);
            cursor = end;
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
    fn segment_shorter_than_the_floor_keeps_its_time() {
        let rendered = render_srt(&[Segment::new(0, 0.0, 0.1, "Hi")], false);
        assert!(rendered.contains("00:00:00,000 --> 00:00:00,100"));
    }

    #[test]
    fn slow_segment_stays_in_one_cue() {
        let segments = vec![Segment::new(0, 0.0, 4.0, "Hello there")];
        let cues = cues(&segments, false);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].start, 0.0);
        assert!((cues[0].end - 4.0).abs() < 1e-9);
    }

    #[test]
    fn dense_segment_shares_its_time_between_cues() {
        let text = "This sentence is long enough to need several cues on screen at once, and the speaker says it far too quickly for anyone to read every word of it comfortably.";
        let segments = vec![Segment::new(0, 0.0, 3.0, text)];
        let cues = cues(&segments, false);
        assert!(cues.len() > 1);
        assert!((cues[0].start).abs() < 1e-9);
        assert!((cues[cues.len() - 1].end - 3.0).abs() < 1e-9);
        for pair in cues.windows(2) {
            assert!((pair[0].end - pair[1].start).abs() < 1e-9);
        }
        for cue in &cues {
            assert!(cue.end > cue.start);
            assert!(cue.end <= 3.0 + 1e-9);
        }
        let joined = cues
            .iter()
            .map(|cue| flatten(&cue.text))
            .collect::<Vec<String>>()
            .join(" ");
        assert_eq!(joined, flatten(text));
    }

    #[test]
    fn no_cue_exceeds_the_reading_rate_when_the_segment_allows() {
        let text = "Hello there! This is a test of the subtitle pipeline. It should produce two subtitle files without dropping a single word from the transcript.";
        let segments = vec![Segment::new(0, 0.0, 14.0, text)];
        for cue in cues(&segments, false) {
            assert!(reading_speed(&cue.text, cue.start, cue.end) <= READ_RATE + 1e-6);
            assert!(cue.end - cue.start >= MIN_CUE - 1e-6);
        }
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
