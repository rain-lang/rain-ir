/*!
A simple parser, and AST for a textual representation of `rain` programs
*/
use nom::{
    branch::alt,
    bytes::complete::{is_a as is_a_c, is_not, tag},
    bytes::streaming::{is_a as is_a_s, take_until},
    character::streaming::{line_ending, not_line_ending},
    combinator::{map, opt},
    multi::separated_nonempty_list,
    sequence::{delimited, preceded},
    Err, IResult,
};
use smallvec::SmallVec;

pub mod ast;
use ast::*;
pub mod builder;

/// The `rain` special characters, including whitespace
pub const SPECIAL_CHARACTERS: &str = " \t\r\n#()[]|\"\':.;/";

/// The `rain` whitespace characters
pub const WHITESPACE: &str = " \t\r\n";

/// The `rain` path separator charactor
pub const PATH_SEP: &str = ".";

/// The delimiter for single-line `rain` comments
pub const SINGLE_COMMENT_START: &str = "//";

/// The opening delimiter for a multi-line `rain` comment
pub const MULTI_COMMENT_OPEN: &str = "/*";

/// The closing delimiter for a multi-line `rain` comment
pub const MULTI_COMMENT_CLOSE: &str = "*/";

/// The opening delimiter for a sexpr
pub const SEXPR_OPEN: &str = "(";

/// The closing delimiter for a sexpr
pub const SEXPR_CLOSE: &str = ")";

/// The opening delimiter for a tuple
pub const TUPLE_OPEN: &str = "[";

/// The closing delimiter for a tuple
pub const TUPLE_CLOSE: &str = "]";

/// The opening delimiter for a scope
pub const SCOPE_OPEN: &str = "{";

/// The closing delimiter for a scope
pub const SCOPE_CLOSE: &str = "}";

/**
Parse a single-line `rain` comment, returning the content as an `&str`.MULTI_COMMENT_END

Single-line comments begin with "//" and run until a line ending, which may be `\n` or `\r\n`, and may contain any character.
This is a streaming parser, so incomplete comments (i.e. cones without an ending newline) will return `Incomplete` instead of `Err`.

# Example
```rust
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
Parse a multi-line `rain` comment, returning the content as an `&str`.

Multi-line comments begin with `/*`, end with `*/`, and may contain any character. This is a streaming
parser, so incomplete comments (i.e. ones without the ending `*\/`) will return `Incomplete` instead of `Err`.

For now, nested comments are not supported, but this may change.

# Example
```rust
use rain_lang::parser::parse_multi_comment;
assert_eq!(
    parse_multi_comment("/*This is a multiline\ncomment*/\nThis is not").unwrap(),
    ("\nThis is not", "This is a multiline\ncomment")
);
assert_eq!(
    parse_multi_comment("/*This is a CRLF\r\nmultiline comment\n*/This still isn't").unwrap(),
    ("This still isn't", "This is a CRLF\r\nmultiline comment\n")
);
assert!(parse_multi_comment(concat!("/", "*This is an incomplete comment")).is_err());
assert!(parse_multi_comment("This is not a comment").is_err());
```
*/
pub fn parse_multi_comment(input: &str) -> IResult<&str, &str> {
    delimited(
        tag(MULTI_COMMENT_OPEN),
        take_until(MULTI_COMMENT_CLOSE),
        tag(MULTI_COMMENT_CLOSE),
    )(input)
}

/**
Parse whitespace (including comments). Returns nothing.

If `complete` is true, then it will consume potentially incomplete whitespace. If `complete` is false and the end of input is
reached and no non-whitespace character has been parsed, then `Incomplete` will be returned. `Incomplete` is always returned
for unfinished multi-line and single-line comments.

# Example
```rust
use rain_lang::parser::parse_ws;

// Whitespace parses as you would expect
assert_eq!(parse_ws(false, "    \t\r      \n\r\n \t   hello   \t").unwrap(), ("hello   \t", ()));
// Comments inside whitespace disappear
assert_eq!(parse_ws(false, r"
    // Hello, I'm a single line comment

    /*
    And I'm a multi-line comment!
    */

    // Look, another single line comment

    some.variable // Another single line comment").unwrap(),
    ("some.variable // Another single line comment", ())
);

// Non-comments and the empty string return an error
assert!(parse_ws(false, "This is not a comment").is_err());
assert!(parse_ws(false, "").is_err());

// Multiline comments work as before, but notice now that newlines after them are consumed
assert_eq!(
    parse_ws(false, "/*This is a multiline\ncomment*/\nThis is not").unwrap(),
    ("This is not", ())
);
assert_eq!(
    parse_ws(false, "/*This is a CRLF\r\nmultiline comment\n*/This still isn't").unwrap(),
    ("This still isn't", ())
);
assert!(parse_ws(false, concat!("/", "*This is an incomplete comment")).is_err());

// The same holds for single line comments
assert_eq!(
    parse_ws(false, "//This is a comment\nThis is not").unwrap(),
    ("This is not", ())
);
assert_eq!(
    parse_ws(false, "//This is a CRLF comment\r\nThis still isn't").unwrap(),
    ("This still isn't", ())
);
assert!(parse_ws(false, "//This is an incomplete comment").is_err());

// Unfinished whitespace in streaming mode is an error
assert!(parse_ws(false, "     ").is_err());

// But not in complete mode, to facilitate REPL construction
assert_eq!(parse_ws(true, "     ").unwrap(), ("", ()));
```
*/
pub fn parse_ws(complete: bool, mut input: &str) -> IResult<&str, ()> {
    let is_a = |input| {
        if complete {
            is_a_c(WHITESPACE)(input)
        } else {
            is_a_s(WHITESPACE)(input)
        }
    };
    input = alt((parse_single_comment, parse_multi_comment, is_a))(input)?.0;
    loop {
        input = match alt((parse_single_comment, parse_multi_comment, is_a))(input) {
            Ok((rest, _)) => rest,
            Err(Err::Incomplete(n)) => return Err(Err::Incomplete(n)),
            _ => return Ok((input, ())),
        }
    }
}

/// Parses whitespace (including comments). Streaming, returns nothing.
/// See `parse_ws`
pub fn ws(input: &str) -> IResult<&str, ()> {
    parse_ws(false, input)
}

/// Parses whitespace (including comments). Complete, returns nothing.
/// See `parse_ws`
pub fn cws(input: &str) -> IResult<&str, ()> {
    parse_ws(true, input)
}

/**
Parse a `rain` identifier, which is composed of a string of non-special, non-whitespace characters.

Numbers are parsed as `Ident`s as well, and only later resolved to their special numeric types.

# Example
```rust
use rain_lang::parser::{ident, ast::Ident};
use nom::IResult;
let process = |res: IResult<&'static str, Ident<'static>>| -> (&'static str, &'static str) {
    let (rest, id) = res.unwrap();
    (rest, id.get_str())
};

// Both special characters and whitespace separate idents
assert_eq!(process(ident("hello world")), (" world", "hello"));
assert_eq!(process(ident("hello.world")), (".world", "hello"));

// Numbers are allowed within an ident
assert_eq!(process(ident("h3110.w0r1d")), (".w0r1d", "h3110"));
assert_eq!(process(ident("1337")), ("", "1337"));

// Unicode is fine too, as are most mathematical operators
assert_eq!(process(ident("C++")), ("", "C++"));
let arabic = process(ident("الحروف العربية"));
let desired_arabic = (" العربية" ,"الحروف");
assert_eq!(arabic, desired_arabic);
assert_eq!(process(ident("汉字:")), (":", "汉字"));

// The empty string is not an ident
assert!(ident("").is_err());
// Whitespace is not an ident
assert!(ident(" ").is_err());
// Nor are special characters
assert!(ident(".").is_err());
assert!(ident("#").is_err());
```
 */
pub fn ident(input: &str) -> IResult<&str, Ident> {
    map(is_not(SPECIAL_CHARACTERS), Ident)(input)
}

/**
Parse a `rain` path, which is composed of a string of `Ident`s separated by periods.

A preceding period is optional and do not affect the resulting object. An empty path is one period:
an empty string is *not* a path, and a trailing period is forbidden.

TODO: make a trailing period with nothing after it return `Incomplete` for REPL purposes.

# Example
```rust
use rain_lang::parser::{path, ast::{Ident, Path}};
use smallvec::smallvec;
use std::convert::TryFrom;
let my_path = Path(smallvec![
    Ident::try_from("hello").unwrap(),
    Ident::try_from("world").unwrap()
]);

// Single periods before a valid path are ignored
assert_eq!(path("hello.world: T").unwrap(), (": T", my_path.clone()));
assert_eq!(path(".hello.world#").unwrap(), ("#", my_path.clone()));
// Periods after a valid path are *not* considered part of the path
assert_eq!(path(".hello.world.\"\"").unwrap(), (".\"\"", my_path.clone()));
assert_eq!(path("hello.world..").unwrap(), ("..", my_path));

// Single periods represent the empty path
assert_eq!(path(".   ").unwrap(), ("   ", Path::empty()));

// Groups of periods are not a valid path.
// Instead, the first period is just parsed as an empty path.
assert_eq!(path("..").unwrap(), (".", Path::empty()));
// Neither are special characters
assert!(path("#").is_err());
// Or the empty string, or whitespace
assert!(path("   ").is_err());
assert!(path("").is_err());
```

# Grammar
The grammar for a path can be represented by the following EBNF fragment:
```ebnf
Path ::= "." | "."? Ident ("." Ident)*
```
*/
pub fn path(input: &str) -> IResult<&str, Path> {
    alt((
        map(
            preceded(
                opt(tag(PATH_SEP)),
                separated_nonempty_list(tag(PATH_SEP), ident),
            ),
            |v| Path(SmallVec::from_vec(v)),
        ),
        map(tag(PATH_SEP), |_| Path::empty()),
    ))(input)
}

/**
Parse a list of `rain` atoms

If `complete` is `false`, the parser will return `Incomplete` in the case of trailing whitespace.
If `complete` is `true`, the parser will not do so, though it will still return `Incomplete` in the case of,  e.g., unfinished comments.
In either case, the parser returns `Incomplete` if given only whitespace or an empty string.

# Grammar
The grammar for a list of atoms can be represented by the following EBNF fragment:
```ebnf
Atoms ::= (WS Atom)+
```
*/
pub fn parse_atoms(complete: bool, input: &str) -> IResult<&str, Vec<Expr>> {
    preceded(
        ws,
        separated_nonempty_list(|input| parse_ws(complete, input), atom),
    )(input)
}

/**
Parse an S-expression
*/
pub fn sexpr(input: &str) -> IResult<&str, Sexpr> {
    map(
        delimited(
            tag(SEXPR_OPEN),
            opt(|input| parse_atoms(false, input)),
            preceded(opt(ws), tag(SEXPR_CLOSE)),
        ),
        |s| s.map(Sexpr).unwrap_or_default(),
    )(input)
}

/**
Parse a tuple
*/
pub fn tuple(input: &str) -> IResult<&str, Tuple> {
    map(
        delimited(
            tag(TUPLE_OPEN),
            opt(|input| parse_atoms(false, input)),
            preceded(opt(ws), tag(TUPLE_CLOSE)),
        ),
        |s| s.map(Tuple).unwrap_or_default(),
    )(input)
}

/**
Parse an atomic `rain` expression. Does *not* consume whitespace before the expression!
*/
pub fn atom(input: &str) -> IResult<&str, Expr> {
    alt((
        map(path, Expr::Path),   // Atom ::= Path
        map(sexpr, Expr::Sexpr), // Atom ::= Sexpr
        map(tuple, Expr::Tuple), // Atom ::= Tuple
    ))(input)
}
