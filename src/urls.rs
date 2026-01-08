use reqwest::Url;

pub enum RelativeOrAbsoluteUrl {
    Relative(String),
    Absolute(Url),
}

impl RelativeOrAbsoluteUrl {
    pub fn to_url(self, base: &str) -> Result<Url, url::ParseError> {
        match self {
            RelativeOrAbsoluteUrl::Relative(mut s) => {
                s.insert_str(0, base);
                s.parse()
            }
            RelativeOrAbsoluteUrl::Absolute(u) => Ok(u),
        }
    }
}

impl From<String> for RelativeOrAbsoluteUrl {
    fn from(s: String) -> Self {
        RelativeOrAbsoluteUrl::Relative(s)
    }
}

impl From<&str> for RelativeOrAbsoluteUrl {
    fn from(s: &str) -> Self {
        RelativeOrAbsoluteUrl::Relative(s.to_string())
    }
}

impl From<Url> for RelativeOrAbsoluteUrl {
    fn from(u: Url) -> Self {
        RelativeOrAbsoluteUrl::Absolute(u)
    }
}
