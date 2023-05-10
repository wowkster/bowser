use std::{convert::Infallible, str::FromStr, time::Duration};

use lazy_static::lazy_static;
use reqwest::blocking::{Client, ClientBuilder};

pub use reqwest::blocking::*;
pub use reqwest::StatusCode;

lazy_static! {
    pub static ref HTTP_CLIENT: Client = ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .connection_verbose(true)
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client");
}

#[derive(Debug)]
pub struct ContentType {
    media_type: MediaType,
    charset: Option<Charset>,
}

impl FromStr for ContentType {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(';').map(|s| s.trim()).collect();

        assert!((1..=2).contains(&parts.len()));

        let (media_type, parameters) = if parts.len() == 2 {
            (parts[0], Some(parts[1]))
        } else {
            (parts[0], None)
        };

        // Match case insensitively
        let media_type = media_type.parse::<MediaType>().unwrap();

        let charset = parameters.and_then(|s| Some(s.parse::<Charset>().unwrap()));

        Ok(ContentType {
            media_type,
            charset,
        })
    }
}

impl ContentType {
    pub fn media_type(&self) -> &MediaType {
        &self.media_type
    }

    pub fn charset(&self) -> Option<&Charset> {
        self.charset.as_ref()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub enum MediaType {
    TextHTML, // text/html
    #[default]
    ApplicationOctetStream, // application/octet-stream
    Other(String),
}

impl FromStr for MediaType {
    type Err = Infallible;

    /// Parse a media type from a string case insensitively
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('/').map(|p| p.to_ascii_lowercase()).collect();

        assert_eq!(parts.len(), 2);

        let [super_type, sub_type]: [String; 2] = parts.try_into().unwrap();

        match (super_type.as_str(), sub_type.as_str()) {
            ("text", "html") => Ok(Self::TextHTML),
            ("application", "octet-stream") => Ok(Self::ApplicationOctetStream),
            _ => Ok(Self::Other(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum Charset {
    UTF8, // charset=utf-8
    Other(String),
}

impl FromStr for Charset {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('=').map(|p| p.to_ascii_lowercase()).collect();

        assert_eq!(parts.len(), 2);

        let [name, value]: [String; 2] = parts.try_into().unwrap();

        assert_eq!(name, "charset");

        // Remove quotes if present
        let value = if value.starts_with('"') && value.ends_with('"') {
            &value[1..value.len() - 1]
        } else {
            &value
        };

        match value {
            "utf-8" => Ok(Self::UTF8),
            other => Ok(Self::Other(other.to_string())),
        }
    }
}

pub trait ResponseContentType {
    fn content_type(&self) -> Option<ContentType>;
}

impl ResponseContentType for Response {
    fn content_type(&self) -> Option<ContentType> {
        let header = self.headers().get("content-type")?;

        let header = header.to_str().ok()?;

        Some(header.parse().unwrap())
    }
}
