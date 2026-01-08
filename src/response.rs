//! Response modules
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use std::{fmt::Debug, sync::LazyLock};

use crate::{error::Error, Megalodon};

static NEXT: LazyLock<Option<String>> = LazyLock::new(|| Some("next".to_string()));

static PREV: LazyLock<Option<String>> = LazyLock::new(|| Some("prev".to_string()));

pub trait MegalodonResponse {
    fn json(&self) -> String;
    fn status(&self) -> u16;
    fn status_text(&self) -> String;
    fn headers(&self) -> HeaderMap;
}

/// Response struct for API response.
#[derive(Debug, Clone)]
pub struct Response<T> {
    /// Parsed json object.
    pub json: T,
    /// Status code of the response.
    pub status: u16,
    /// Status text of the response.
    pub status_text: String,
    /// Headers of the response.
    pub header: HeaderMap,
}

/// A struct to wrap the previous and next uris provided in paginated responses.
#[derive(Clone)]
pub struct LinkedResponse<T> {
    pub url: reqwest::Url,
    phantom: std::marker::PhantomData<T>,
}

impl<T> LinkedResponse<T> {
    pub(crate) fn new(url: reqwest::Url) -> Self {
        Self {
            url: url,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Debug for LinkedResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.url.fmt(f)
    }
}

impl<T> Response<T> {
    /// Create a new Response struct.
    pub fn new(json: T, status: u16, status_text: String, header: HeaderMap) -> Response<T> {
        Self {
            json,
            status,
            status_text,
            header,
        }
    }

    /// Create a new Response struct from reqwest::Response.
    pub async fn from_reqwest(response: reqwest::Response) -> Result<Response<T>, reqwest::Error>
    where
        T: DeserializeOwned + Debug,
    {
        let header = response.headers().clone();
        let status_code = response.status();
        let json = response.json::<T>().await?;

        Ok(Self {
            json,
            status: status_code.as_u16(),
            status_text: status_code.as_str().to_string(),
            header,
        })
    }

    /// Get json object.
    pub fn json(&self) -> T
    where
        T: Clone,
    {
        self.json.clone()
    }

    fn get_link(&self, rel: &Option<String>) -> Result<Option<LinkedResponse<T>>, Error> {
        let Some(value) = self.header.get("link") else {
            return Ok(None);
        };

        let value_str = match value.to_str() {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        let parsed = match parse_link_header::parse(value_str) {
            Ok(links) => links,
            Err(e) => return Err(e.into()),
        };

        let Some(link) = parsed.get(rel) else {
            return Ok(None);
        };

        Ok(Some(LinkedResponse::new(
            reqwest::Url::parse(&link.raw_uri).unwrap(),
        )))
    }

    pub fn next_uri(&self) -> Result<Option<LinkedResponse<T>>, Error> {
        self.get_link(&NEXT)
    }

    pub fn prev_uri(&self) -> Result<Option<LinkedResponse<T>>, Error> {
        self.get_link(&PREV)
    }
}

/// Response struct for API response.
#[derive(Debug, Clone)]
pub struct PaginatedResponse<'m, T, TRaw, M: Megalodon> {
    /// Parsed json object.
    pub response: Response<T>,
    /// Megalodon instance used to make the pagination requests.
    pub megalodon: &'m M,
    /// Function to convert from raw response type to parsed type.
    pub convert_response: fn(TRaw) -> T,
}
