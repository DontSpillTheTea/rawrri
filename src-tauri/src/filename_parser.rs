use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use regex::Regex;

use crate::models::VideoSide;

#[derive(Debug, Clone)]
pub struct ParsedFilename {
    pub raw_basename: String,
    pub side: VideoSide,
    pub raw_timestamp_string: String,
    pub timestamp: Option<NaiveDateTime>,
    pub sequence: Option<u32>,
    pub extension: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFilenameError {
    NoPatternMatch,
    InvalidDateTime,
    InvalidSide,
}

pub fn parse_k6_filename(filename: &str) -> Option<ParsedFilename> {
    parse_k6_filename_with_error(filename).ok()
}

pub fn parse_k6_filename_with_error(filename: &str) -> Result<ParsedFilename, ParseFilenameError> {
    let regex = Regex::new(r"(?i)^(\d{8})_(\d{6})_(\d+)_([FR])\.([A-Za-z0-9]+)$")
        .map_err(|_| ParseFilenameError::NoPatternMatch)?;
    let captures = regex
        .captures(filename)
        .ok_or(ParseFilenameError::NoPatternMatch)?;

    let date_raw = captures
        .get(1)
        .map(|m| m.as_str())
        .ok_or(ParseFilenameError::NoPatternMatch)?;
    let time_raw = captures
        .get(2)
        .map(|m| m.as_str())
        .ok_or(ParseFilenameError::NoPatternMatch)?;
    let sequence_raw = captures
        .get(3)
        .map(|m| m.as_str())
        .ok_or(ParseFilenameError::NoPatternMatch)?;
    let side_raw = captures
        .get(4)
        .map(|m| m.as_str())
        .ok_or(ParseFilenameError::NoPatternMatch)?
        .to_uppercase();
    let extension_raw = captures
        .get(5)
        .map(|m| m.as_str())
        .ok_or(ParseFilenameError::NoPatternMatch)?;

    if !extension_raw.eq_ignore_ascii_case("mp4") {
        return Err(ParseFilenameError::NoPatternMatch);
    }

    let side = match side_raw.as_str() {
        "F" => VideoSide::Front,
        "R" => VideoSide::Rear,
        _ => return Err(ParseFilenameError::InvalidSide),
    };

    let timestamp = parse_timestamp(date_raw, time_raw).ok_or(ParseFilenameError::InvalidDateTime)?;
    let sequence = sequence_raw.parse::<u32>().ok();

    Ok(ParsedFilename {
        raw_basename: filename.to_string(),
        side,
        raw_timestamp_string: format!("{date_raw}_{time_raw}"),
        timestamp: Some(timestamp),
        sequence,
        extension: "mp4".to_string(),
    })
}

fn parse_timestamp(date_raw: &str, time_raw: &str) -> Option<NaiveDateTime> {
    let date = NaiveDate::parse_from_str(date_raw, "%Y%m%d").ok()?;
    let time = NaiveTime::parse_from_str(time_raw, "%H%M%S").ok()?;
    Some(NaiveDateTime::new(date, time))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_front_file() {
        let parsed = parse_k6_filename("20260323_114324_000023_F.MP4").expect("should parse");
        assert_eq!(parsed.side, VideoSide::Front);
        assert_eq!(parsed.sequence, Some(23));
        assert_eq!(parsed.raw_timestamp_string, "20260323_114324".to_string());
        assert_eq!(
            parsed.timestamp.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            Some("2026-03-23 11:43:24".to_string())
        );
    }

    #[test]
    fn parses_rear_lowercase_extension() {
        let parsed = parse_k6_filename("20260323_114325_000024_r.mp4").expect("should parse");
        assert_eq!(parsed.side, VideoSide::Rear);
        assert_eq!(parsed.sequence, Some(24));
    }

    #[test]
    fn rejects_non_k6_name() {
        assert!(parse_k6_filename("video.mp4").is_none());
    }

    #[test]
    fn rejects_non_mp4_extensions() {
        assert!(parse_k6_filename("20260323_112727_000007_F.JPG").is_none());
    }

    #[test]
    fn parses_examples_from_fixture_set() {
        let sample_names = [
            "20260323_112520_000005_F.MP4",
            "20260323_112521_000006_R.MP4",
            "20260323_114324_000023_F.MP4",
            "20260323_114325_000024_R.MP4",
            "20000101_000319_000001_F.MP4",
            "20000101_000319_000002_R.MP4",
        ];

        for name in sample_names {
            assert!(parse_k6_filename(name).is_some(), "expected parser match for {}", name);
        }
    }

    #[test]
    fn reports_invalid_datetime_when_shape_matches() {
        let err = parse_k6_filename_with_error("20261323_250000_000005_F.MP4").expect_err("should fail");
        assert_eq!(err, ParseFilenameError::InvalidDateTime);
    }
}
