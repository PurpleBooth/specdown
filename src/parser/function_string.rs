use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::streaming::{tag, take_until},
    character::streaming::{alpha1, alphanumeric1, space0},
    combinator::map,
    multi::{many0, separated_list},
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub arguments: HashMap<String, ArgumentValue>,
}

type Argument<'a> = (&'a str, ArgumentValue);

#[derive(Debug, PartialEq, Clone)]
pub enum ArgumentValue {
    String(String),
    Token(String),
}

pub fn parse(input: &str) -> IResult<&str, Function> {
    let p = tuple((alpha1, argument_list));
    map(p, |(name, arguments)| Function {
        name: name.to_string(),
        arguments,
    })(input)
}

fn argument_list(input: &str) -> IResult<&str, HashMap<String, ArgumentValue>> {
    let p = delimited(
        tag("("),
        separated_list(tuple((space0, tag(","), space0)), argument),
        tag(")"),
    );

    map(p, |args| list_of_args_to_hash_map(&args))(input)
}

fn list_of_args_to_hash_map(arguments: &[Argument]) -> HashMap<String, ArgumentValue> {
    let mut result = HashMap::new();

    for (name, value) in arguments {
        result.insert(String::from(*name), value.clone());
    }

    result
}

fn argument(input: &str) -> IResult<&str, Argument> {
    let p = tuple((
        argument_name,
        tuple((space0, tag("="), space0)),
        argument_value,
    ));
    map(p, |(name, _, value)| (name, value))(input)
}

fn argument_name(input: &str) -> IResult<&str, &str> {
    let p = tuple((alpha1, many0(alt((alphanumeric1, tag("_"))))));
    let (remainder, (start, parts)) = p(input)?;
    let length = start.len() + parts.join("").len();
    Ok((remainder, &input[0..length]))
}

fn argument_value(input: &str) -> IResult<&str, ArgumentValue> {
    alt((string_value, token_value))(input)
}

fn string_value(input: &str) -> IResult<&str, ArgumentValue> {
    let p = delimited(tag("\""), take_until("\""), tag("\""));
    map(p, |s: &str| ArgumentValue::String(s.to_string()))(input)
}

fn token_value(input: &str) -> IResult<&str, ArgumentValue> {
    map(alpha1, |token: &str| {
        ArgumentValue::Token(token.to_string())
    })(input)
}

// mod tests {
//     #[cfg(test)]
//     use super::*;

//     mod parse {
//         #[cfg(test)]
//         use super::*;

//         #[test]
//         fn succeeds_when_function_has_no_args() {
//             assert_eq!(
//                 parse("func(), more"),
//                 Ok((
//                     ", more",
//                     Function {
//                         name: "func",
//                         arguments: HashMap::new()
//                     }
//                 ))
//             )
//         }

//         #[test]
//         fn succeeds_when_function_has_args() {
//             assert_eq!(
//                 parse("funcy(arg=\"hi\")"),
//                 Ok((
//                     "",
//                     Function {
//                         name: "funcy",
//                         arguments: [("arg", ArgumentValue::String("hi"))]
//                             .iter()
//                             .cloned()
//                             .collect()
//                     }
//                 ))
//             )
//         }

//         #[test]
//         fn succeeds_when_no_arguments_are_provided() {
//             let expected_fn = Function {
//                 name: "fn",
//                 arguments: HashMap::new(),
//             };
//             assert_eq!(parse("fn()"), Ok(("", expected_fn)));
//         }

//         #[test]
//         fn succeeds_when_one_argument_is_provided() {
//             let expected_fn = Function {
//                 name: "fn",
//                 arguments: [("arg", ArgumentValue::String("abc"))]
//                     .iter()
//                     .cloned()
//                     .collect(),
//             };

//             assert_eq!(parse("fn(arg=\"abc\")"), Ok(("", expected_fn)));
//         }

//         #[test]
//         fn succeeds_when_multiple_arguments_are_provided() {
//             let expected_fn = Function {
//                 name: "fn",
//                 arguments: [
//                     ("arg1", ArgumentValue::Token("abc")),
//                     ("arg2", ArgumentValue::String("def")),
//                 ]
//                 .iter()
//                 .cloned()
//                 .collect(),
//             };

//             assert_eq!(parse("fn(arg1=abc,arg2=\"def\")"), Ok(("", expected_fn)));
//         }

//         #[test]
//         fn succeeds_when_multiple_arguments_are_provided_with_spaces() {
//             let expected_fn = Function {
//                 name: "fn",
//                 arguments: [
//                     ("arg1", ArgumentValue::Token("xxx")),
//                     ("arg2", ArgumentValue::String("123")),
//                 ]
//                 .iter()
//                 .cloned()
//                 .collect(),
//             };

//             assert_eq!(parse("fn(arg1=xxx , arg2=\"123\")"), Ok(("", expected_fn)));
//         }
//     }

//     mod argument_list {
//         #[cfg(test)]
//         use super::*;

//         #[test]
//         fn succeeds_when_no_arguments_are_provided() {
//             let expected_args: HashMap<&str, ArgumentValue> = [].iter().cloned().collect();
//             assert_eq!(argument_list("()"), Ok(("", expected_args)));
//         }

//         #[test]
//         fn succeeds_when_one_argument_is_provided() {
//             let expected_args: HashMap<&str, ArgumentValue> =
//                 [("arg", ArgumentValue::String("abc"))]
//                     .iter()
//                     .cloned()
//                     .collect();

//             assert_eq!(argument_list("(arg=\"abc\")"), Ok(("", expected_args)));
//         }

//         #[test]
//         fn succeeds_when_multiple_arguments_are_provided() {
//             let expected_args: HashMap<&str, ArgumentValue> = [
//                 ("arg1", ArgumentValue::Token("abc")),
//                 ("arg2", ArgumentValue::String("def")),
//             ]
//             .iter()
//             .cloned()
//             .collect();

//             assert_eq!(
//                 argument_list("(arg1=abc,arg2=\"def\")"),
//                 Ok(("", expected_args))
//             );
//         }

//         #[test]
//         fn succeeds_when_multiple_arguments_are_provided_with_spaces() {
//             let expected_args: HashMap<&str, ArgumentValue> = [
//                 ("arg1", ArgumentValue::Token("xxx")),
//                 ("arg2", ArgumentValue::String("123")),
//             ]
//             .iter()
//             .cloned()
//             .collect();
//             assert_eq!(
//                 argument_list("(arg1=xxx , arg2=\"123\")"),
//                 Ok(("", expected_args))
//             );
//         }
//     }

//     mod argument {
//         #[cfg(test)]
//         use super::*;

//         #[test]
//         fn fails_when_name_starts_with_a_digit() {
//             assert_eq!(
//                 argument("1arg=\"value\",more..."),
//                 Err(nom::Err::Error(nom::error_position!(
//                     "1arg=\"value\",more...",
//                     nom::error::ErrorKind::Alpha
//                 )))
//             );
//         }

//         #[test]
//         fn succeeds_when_single_alpha_char_name() {
//             assert_eq!(
//                 argument("a=\"value\",more..."),
//                 Ok((",more...", ("a", ArgumentValue::String("value"))))
//             );
//         }

//         #[test]
//         fn succeeds_when_alpha_numeric_name() {
//             assert_eq!(
//                 argument("arg1=\"value\",more..."),
//                 Ok((",more...", ("arg1", ArgumentValue::String("value"))))
//             );
//         }

//         #[test]
//         fn succeds_with_token_argument() {
//             assert_eq!(
//                 argument("arg=token,more..."),
//                 Ok((",more...", ("arg", ArgumentValue::Token("token"))))
//             );
//         }

//         #[test]
//         fn succeeds_when_arg_contains_underscore() {
//             assert_eq!(
//                 argument("arg_name=\"value\",more..."),
//                 Ok((",more...", ("arg_name", ArgumentValue::String("value"))))
//             );
//         }

//         #[test]
//         fn succeds_with_spaces_around_equals() {
//             assert_eq!(
//                 argument("arg  =  token rest"),
//                 Ok((" rest", ("arg", ArgumentValue::Token("token"))))
//             );
//         }
//     }

//     mod argument_value {
//         #[cfg(test)]
//         use super::*;

//         #[test]
//         fn succeeds_when_there_is_a_remainder() {
//             assert_eq!(
//                 argument_value("\"string\" rest"),
//                 Ok((" rest", ArgumentValue::String("string")))
//             );
//         }

//         // #[test]
//         // fn succeeds_when_there_is_no_remainder() {
//         //     let result = argument_value("token");
//         //     assert_eq!(result, Ok(("", ArgumentValue::Token("token"))));
//         // }

//         mod string_value {
//             #[cfg(test)]
//             use super::*;

//             #[test]
//             fn succeeds_when_there_is_a_remainder() {
//                 assert_eq!(
//                     argument_value("\"arg_value1\" leftovers"),
//                     Ok((" leftovers", ArgumentValue::String("arg_value1")))
//                 );
//             }

//             #[test]
//             fn succeeds_when_there_is_no_remainder() {
//                 assert_eq!(
//                     argument_value("\"arg_value2\""),
//                     Ok(("", ArgumentValue::String("arg_value2")))
//                 );
//             }

//             #[test]
//             fn fails_when_there_is_no_closing_quote() {
//                 assert_eq!(
//                     argument_value("\"arg_value2"),
//                     Err(nom::Err::Incomplete(nom::Needed::Size(1)))
//                 );
//             }
//         }

//         mod token_value {
//             #[cfg(test)]
//             use super::*;

//             #[test]
//             fn succeeds_when_there_is_a_remainder() {
//                 let result = argument_value("stdout leftovers");
//                 assert_eq!(result, Ok((" leftovers", ArgumentValue::Token("stdout"))));
//             }

//             // #[test]
//             // fn succeeds_when_there_is_no_remainder() {
//             //     let result = argument_value("stderr");
//             //     assert_eq!(result, Ok(("", ArgumentValue::Token("stderr"))));
//             // }
//         }
//     }
// }