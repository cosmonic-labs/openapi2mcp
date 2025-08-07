use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
    Generation(String),
    Network(String),
    Serialization(String),
    Template(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Parse(msg) => write!(f, "Parse error: {}", msg),
            Error::Validation(msg) => write!(f, "Validation error: {}", msg),
            Error::Generation(msg) => write!(f, "Generation error: {}", msg),
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Template(msg) => write!(f, "Template error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Parse(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Parse(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::Parse(format!("URL parse error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let io_error = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(io_error.to_string().contains("IO error"));

        let parse_error = Error::Parse("invalid syntax".to_string());
        assert_eq!(parse_error.to_string(), "Parse error: invalid syntax");

        let validation_error = Error::Validation("missing field".to_string());
        assert_eq!(validation_error.to_string(), "Validation error: missing field");

        let generation_error = Error::Generation("template error".to_string());
        assert_eq!(generation_error.to_string(), "Generation error: template error");
    }

    #[test]
    fn test_error_from_conversions() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let error: Error = io_err.into();
        matches!(error, Error::Io(_));

        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error: Error = json_err.into();
        matches!(error, Error::Parse(_));
    }
}