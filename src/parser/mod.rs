/*!
A simple parser, and AST for a textual representation of `rain` programs
*/
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::streaming::{line_ending, not_line_ending},
    combinator::{map, opt},
    multi::separated_nonempty_list,
    sequence::{delimited, preceded},
    IResult,
};
use smallvec::SmallVec;

pub mod ast;
use ast::*;
pub mod builder;

/// The `rain` special characters
const SPECIAL_CHARACTERS: &str = " \t\r\n#()[]|\"\':.;/";

/// The `rain` path separator charactor
const PATH_SEP: &str = ".";

/// The delimiter for single-line `rain` comments
const SINGLE_COMMENT_START: &str = "//";

/**
Parse a single-line `rain` comment, which begins with "//" and runs until a line ending, which may be `\n` or `\r\n`. 
Comments may contain any character. The content of the comment, not including the newline, is returned as an `&str`.
If the comment is not terminated by a newline, `Incomplete` is returned.
```
use rain_lang::parser::parse_single_comment;
assert_eq!(
    parse_single_comment("//This is a comment\nThis is not").unwrap(), 
    ("This is not", "This is a comment")
);
assert_eq!(
    parse_single_comment("//This is a CRLF comment\r\nThis still isn't").unwrap(), 
    ("This still isn't", "This is a CRLF comment")
);
assert!(parse_single_comment("//This is an incomplete comment").is_err());
assert!(parse_single_comment("This is not a comment").is_err());
```
*/
pub fn parse_single_comment(input: &str) -> IResult<&str, &str> {
    delimited(tag(SINGLE_COMMENT_START), not_line_ending, line_ending)(input)
}

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

/**
Parse a `rain` path, which is composed of a string of `Ident`s separated by periods.
A preceding period is optional and do not affect the resulting object. An empty path is one period: 
an empty string is *not* a path, and a trailing period is forbidden.

TODO: make a trailing period with nothing after it return `Incomplete` for REPL purposes.
```
use rain_lang::parser::{parse_path, ast::{Ident, Path}};
use smallvec::smallvec;
use std::convert::TryFrom;
let my_path = Path(smallvec![
    Ident::try_from("hello").unwrap(),
    Ident::try_from("world").unwrap()
]);

// Single periods before a valid path are ignored
assert_eq!(parse_path("hello.world: T").unwrap(), (": T", my_path.clone()));
assert_eq!(parse_path(".hello.world#").unwrap(), ("#", my_path.clone()));
// Periods after a valid path are *not* considered part of the path
assert_eq!(parse_path(".hello.world.\"\"").unwrap(), (".\"\"", my_path.clone()));
assert_eq!(parse_path("hello.world..").unwrap(), ("..", my_path));

// Single periods represent the empty path
assert_eq!(parse_path(".   ").unwrap(), ("   ", Path::empty()));

// Groups of periods are not a valid path, but instead just parse the first period as an empty path.
assert_eq!(parse_path("..").unwrap(), (".", Path::empty()));
// Neither are special characters
assert!(parse_path("#").is_err());
// Or the empty string, or whitespace
assert!(parse_path("   ").is_err());
assert!(parse_path("").is_err());
```

The grammar for a path can be represented by the following EBNF fragment:
```ebnf
Path ::= "." | "."? Ident ("." Ident)*
```
*/
pub fn parse_path(input: &str) -> IResult<&str, Path> {
    alt((
        map(
            preceded(
                opt(tag(PATH_SEP)),
                separated_nonempty_list(tag(PATH_SEP), parse_ident),
            ),
            |v| Path(SmallVec::from_vec(v)),
        ),
        map(tag(PATH_SEP), |_| Path::empty()),
    ))(input)
}