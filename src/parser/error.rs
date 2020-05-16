use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    RootMustBeDocument,
    StringEncodingFailed(String),
    ParserFailed(String),
    UnknownFunction(String),
    MissingArgument(String, String),
    IncorrectArgumentType { expected: String, got: String },
    InvalidArgumentValue { got: String, expected: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RootMustBeDocument => {
                write!(f, "RootMustBeDocument :: This error should never occur")
            }
            Self::StringEncodingFailed(msg) => {
                write!(f, "Failed to encode string. Got error: {}", msg)
            }
            Self::ParserFailed(msg) => write!(f, "The parser failed: {}", msg),
            Self::UnknownFunction(name) => write!(f, "Unknown function: {}", name),
            Self::MissingArgument(func, arg) => {
                write!(f, "Function {} requires argument {}", func, arg)
            }
            Self::IncorrectArgumentType { expected, got } => write!(
                f,
                "Invalid argument type. Expected {}, got {}",
                expected, got
            ),
            Self::InvalidArgumentValue { got, expected } => write!(
                f,
                "Invalid argument value. Expected {}, got {}",
                expected, got
            ),
        }
    }
}

mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    fn display_root_must_be_document() {
        assert_eq!(
            format!("{}", Error::RootMustBeDocument),
            "RootMustBeDocument :: This error should never occur"
        )
    }

    #[test]
    fn display_string_encoding_failed() {
        assert_eq!(
            format!("{}", Error::StringEncodingFailed("message".to_string())),
            "Failed to encode string. Got error: message"
        )
    }

    #[test]
    fn display_parser_failed() {
        assert_eq!(
            format!("{}", Error::ParserFailed("reason".to_string())),
            "The parser failed: reason"
        )
    }

    #[test]
    fn display_unknown_function() {
        assert_eq!(
            format!("{}", Error::UnknownFunction("funcy".to_string())),
            "Unknown function: funcy"
        )
    }

    #[test]
    fn display_missing_argument() {
        assert_eq!(
            format!(
                "{}",
                Error::MissingArgument("funcy".to_string(), "argy".to_string())
            ),
            "Function funcy requires argument argy"
        )
    }

    #[test]
    fn display_incorrect_argument_type() {
        assert_eq!(
            format!(
                "{}",
                Error::IncorrectArgumentType {
                    expected: "token".to_string(),
                    got: "string".to_string()
                }
            ),
            "Invalid argument type. Expected token, got string"
        )
    }

    #[test]
    fn display_invalid_argument_value() {
        assert_eq!(
            format!(
                "{}",
                Error::InvalidArgumentValue {
                    expected: "true or false".to_string(),
                    got: "maybe".to_string()
                }
            ),
            "Invalid argument value. Expected true or false, got maybe"
        )
    }
}