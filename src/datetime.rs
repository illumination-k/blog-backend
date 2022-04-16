use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderBy {
    CreatedAt,
    UpdatedAt,
}

pub const SUPPORT_DATETIME_FORMAT: [&str; 2] = ["%Y/%m/%d %H:%M:%S", "%Y-%m-%d %H:%M:%S"];
pub const SUPPORT_DATE_FORMAT: [&str; 2] = ["%Y/%m/%d", "%Y-%m-%d"];

#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeFormat {
    RFC3339,
    RFC2822,
    Custom(String),
}

impl From<&str> for DateTimeFormat {
    fn from(s: &str) -> Self {
        if s == "RFC3329" {
            DateTimeFormat::RFC3339
        } else if s == "RFC2822" {
            DateTimeFormat::RFC2822
        } else {
            DateTimeFormat::Custom(s.to_string())
        }
    }
}

impl ToString for DateTimeFormat {
    fn to_string(&self) -> String {
        match self {
            DateTimeFormat::RFC3339 => "RFC3329".to_string(),
            DateTimeFormat::RFC2822 => "RFC2822".to_string(),
            DateTimeFormat::Custom(s) => s.to_owned(),
        }
    }
}

impl DateTimeFormat {
    pub fn format(&self, datetime: DateTime<Utc>) -> String {
        match self {
            DateTimeFormat::RFC3339 => datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            DateTimeFormat::RFC2822 => datetime.to_rfc2822(),
            DateTimeFormat::Custom(fmt) => datetime.format(fmt).to_string(),
        }
    }
}

pub fn parse_rfc(s: &str) -> Result<(DateTimeFormat, DateTime<Utc>)> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(s) {
        return Ok((DateTimeFormat::RFC3339, datetime.into()));
    }

    if let Ok(datetime) = DateTime::parse_from_rfc2822(s) {
        return Ok((DateTimeFormat::RFC2822, datetime.into()));
    }

    Err(anyhow!(format!(
        "Cannot Parse {} by RFC3339 and RFC2822",
        s
    )))
}

pub fn parse_datetime(
    s: &str,
    custom_fmt: Option<String>,
) -> Result<(DateTimeFormat, DateTime<Utc>)> {
    if let Some(fmt) = custom_fmt {
        if let Ok(datetime) = Utc.datetime_from_str(s, &fmt) {
            return Ok((DateTimeFormat::Custom(fmt), datetime));
        }
    }

    if let Ok(res) = parse_rfc(s) {
        return Ok(res);
    }

    for fmt in SUPPORT_DATETIME_FORMAT.into_iter() {
        if let Ok(datetime) = Utc.datetime_from_str(s, fmt) {
            return Ok((DateTimeFormat::Custom(fmt.to_string()), datetime));
        }
    }

    for fmt in SUPPORT_DATE_FORMAT.into_iter() {
        if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
            let datetime = DateTime::<Utc>::from_utc(date.and_hms(0, 0, 0), Utc);
            return Ok((DateTimeFormat::Custom(fmt.to_string()), datetime));
        }
    }

    Err(anyhow!(format!("Cannot Parse {}", s)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateTimeWithFormat {
    datetime: DateTime<Utc>,
    format: DateTimeFormat,
}

impl DateTimeWithFormat {
    pub fn new(datetime: DateTime<Utc>, format: DateTimeFormat) -> Self {
        Self { datetime, format }
    }

    pub fn now(format: &DateTimeFormat) -> Self {
        Self::new(Utc::now(), format.to_owned())
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    pub fn format(&self) -> DateTimeFormat {
        self.format.clone()
    }

    pub fn from_str(date_string: &str) -> Result<Self> {
        match parse_datetime(date_string, None) {
            Ok((format, datetime)) => Ok(Self::new(datetime, format)),
            Err(e) => Err(e),
        }
    }
}

impl ToString for DateTimeWithFormat {
    fn to_string(&self) -> String {
        self.format.format(self.datetime)
    }
}

impl Default for DateTimeWithFormat {
    fn default() -> Self {
        Self::now(&DateTimeFormat::RFC3339)
    }
}

#[cfg(test)]
mod test_datetime {
    use super::*;

    const RFC3329: &str = "2022-01-11T19:08:09+00:00";
    const RFC2822: &str = "Tue, 11 Jan 2022 19:09:09 +0000";
    const SUPPORT_DATETIME_0: &str = "2022/01/11 19:22:50";
    const SUPPORT_DATE_0: &str = "2022/01/11";

    #[test]
    fn test_parse_rfc() {
        let (fmt, _) = parse_rfc(RFC3329).unwrap();
        assert_eq!(fmt, DateTimeFormat::RFC3339);
        let (fmt, _) = parse_rfc(RFC2822).unwrap();
        assert_eq!(fmt, DateTimeFormat::RFC2822);
    }

    #[test]
    fn test_parse_datetime() {
        let (fmt, datetime) = parse_datetime(RFC3329, None).unwrap();
        assert_eq!(fmt, DateTimeFormat::RFC3339);
        assert_eq!(fmt.format(datetime), RFC3329);
        let (fmt, datetime) = parse_datetime(RFC2822, None).unwrap();
        assert_eq!(fmt, DateTimeFormat::RFC2822);
        assert_eq!(fmt.format(datetime), RFC2822);
        let (fmt, datetime) = parse_datetime(SUPPORT_DATETIME_0, None).unwrap();
        assert_eq!(
            fmt,
            DateTimeFormat::Custom(SUPPORT_DATETIME_FORMAT[0].to_string())
        );
        assert_eq!(fmt.format(datetime), SUPPORT_DATETIME_0)
        // let (fmt, _) = parse_datetime(SUPPORT2, None).unwrap();
        // assert_eq!(fmt, DateTimeFormat::Custom(SUPPORT_DATETIME_FORMAT[1].to_string()));
    }

    #[test]
    fn test_parse_date() {
        let (fmt, datetime) = parse_datetime(SUPPORT_DATE_0, None).unwrap();
        assert_eq!(
            fmt,
            DateTimeFormat::Custom(SUPPORT_DATE_FORMAT[0].to_string())
        );
        assert_eq!(fmt.format(datetime), SUPPORT_DATE_0)
    }
}
