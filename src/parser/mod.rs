/*!
A simple parser, and AST for a textual representation of `rain` programs
*/
use nom::{bytes::complete::is_not, combinator::map, IResult};

pub mod ast;
use ast::*;
pub mod builder;

/// The `rain` special characters
const SPECIAL_CHARACTERS: &str = " \t\r\n#()[]|\"\':.;";

/**
Parse a `rain` identifier, which is composed of a string of non-special, non-whitespace characters.
Numbers are parsed as `Ident`s as well, and only later resolved to their special numeric types.
```
use rain_lang::parser::{parse_ident, ast::Ident};
use nom::IResult;
let process = |res: IResult<&'static str, Ident<'static>>| -> (&'static str, &'static str) {
    let (rest, id) = res.unwrap();
    (rest, id.get_str())
};

// Both special characters and whitespace separate idents
assert_eq!(process(parse_ident("hello world")), (" world", "hello"));
assert_eq!(process(parse_ident("hello.world")), (".world", "hello"));

// Numbers are allowed within an ident
assert_eq!(process(parse_ident("h3110.w0r1d")), (".w0r1d", "h3110"));
assert_eq!(process(parse_ident("1337")), ("", "1337"));

// Unicode is fine too, as are most mathematical operators
assert_eq!(process(parse_ident("C++")), ("", "C++"));
let arabic = process(parse_ident("الحروف العربية"));
let desired_arabic = (" العربية" ,"الحروف");
assert_eq!(arabic, desired_arabic);
assert_eq!(process(parse_ident("汉字:")), (":", "汉字"));

// The empty string is not an ident
assert!(parse_ident("").is_err());
// Whitespace is not an ident
assert!(parse_ident(" ").is_err());
// Nor are special characters
assert!(parse_ident(".").is_err());
assert!(parse_ident("#").is_err());
```
 */
pub fn parse_ident(input: &str) -> IResult<&str, Ident> {
    map(is_not(SPECIAL_CHARACTERS), Ident)(input)
}
