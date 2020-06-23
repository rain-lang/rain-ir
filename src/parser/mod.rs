/*!
A simple parser, and AST for a textual representation of `rain` programs
*/
use crate::prettyprinter::tokens::*;
use crate::primitive::{
    finite::Finite,
    logical::{And, Bool, Id as LogicalId, Iff, Logical, Nand, Nor, Not, Or, Xor},
};
use nom::{
    branch::alt,
    bytes::complete::{is_a as is_a_c, is_not, tag},
    bytes::streaming::{is_a as is_a_s, take_until},
    character::complete::{digit1, hex_digit1, oct_digit1},
    character::streaming::{line_ending, not_line_ending},
    combinator::{map, map_res, opt},
    multi::{many0, many1, separated_list, separated_nonempty_list},
    sequence::{delimited, preceded, separated_pair, tuple},
    Err, IResult,
};
use smallvec::SmallVec;
use std::convert::TryFrom;

pub mod ast;
use ast::*;
pub mod builder;

/**
Parse a single-line `rain` comment, returning the content as an `&str`.MULTI_COMMENT_END

Single-line comments begin with "//" and run until a line ending, which may be `\n` or `\r\n`, and may contain any character.
This is a streaming parser, so incomplete comments (i.e. cones without an ending newline) will return `Incomplete` instead of `Err`.

# Example
```rust
use rain_ir::parser::parse_single_comment;
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
use rain_ir::parser::parse_multi_comment;
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
use rain_ir::parser::parse_ws;

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
Parse a `bool`, i.e. `#true` or `#false`

# Example
```rust
use rain_ir::parser::parse_bool;
assert_eq!(parse_bool("#true something_else"), Ok((" something_else", true)));
assert_eq!(parse_bool("#false #true"), Ok((" #true", false)));
assert!(parse_bool("#7rue").is_err())
```
*/
pub fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag(KEYWORD_TRUE), |_| true),
        map(tag(KEYWORD_FALSE), |_| false),
    ))(input)
}

/**
Parse the boolean type
*/
pub fn parse_bool_ty(input: &str) -> IResult<&str, Bool> {
    map(tag(KEYWORD_BOOL), |_| Bool)(input)
}

/**
Parse a `rain` identifier, which is composed of a string of non-special, non-whitespace characters.

Numbers are parsed as `Ident`s as well, and only later resolved to their special numeric types.

# Example
```rust
use rain_ir::parser::{parse_ident, ast::Ident};
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
Parse a `rain` path, which is composed of a string of `Ident`s preceded by periods.

TODO: make a trailing period with nothing after it return `Incomplete` for REPL purposes?

# Example
```rust
use rain_ir::parser::{parse_path, ast::{Ident, Path}};
use smallvec::smallvec;
use std::convert::TryFrom;
let my_path = Path(smallvec![
    Ident::try_from("hello").unwrap(),
    Ident::try_from("world").unwrap()
]);

// Single periods before a valid path are ignored
assert_eq!(parse_path(".hello.world#").unwrap(), ("#", my_path.clone()));
// Periods after a valid path are *not* considered part of the path
assert_eq!(parse_path(".hello.world.\"\"").unwrap(), (".\"\"", my_path.clone()));
assert_eq!(parse_path(".hello.world..").unwrap(), ("..", my_path));

// Periods alone are not a valid path. (TODO: consider)
assert!(parse_path(".").is_err());
assert!(parse_path("..").is_err());
// Neither are special characters
assert!(parse_path("#").is_err());
// Or the empty string, or whitespace
assert!(parse_path("   ").is_err());
assert!(parse_path("").is_err());
```

# Grammar
The grammar for a path can be represented by the following EBNF fragment:
```ebnf
Path ::= ("." Ident)*
```
*/
pub fn parse_path(input: &str) -> IResult<&str, Path> {
    map(many1(preceded(tag(PATH_SEP), parse_ident)), |v| {
        Path(SmallVec::from_vec(v))
    })(input)
}

/**
Parse a list of compound `rain` expressions

If `complete` is `false`, the parser will return `Incomplete` in the case of trailing whitespace.
If `complete` is `true`, the parser will not do so, though it will still return `Incomplete` in the case of,  e.g., unfinished comments.
In either case, the parser returns `Incomplete` if given only whitespace or an empty string.

# Example
```rust
use rain_ir::parser::{parse_expr_list, ast::{Expr, Sexpr, Tuple}};
let (rest, list_of_units) = parse_expr_list(true, "() () [] [] ()").expect("Valid");
assert_eq!(rest, "");
//assert!(parse_expr_list(false, "(), (), [] [] ()").is_err()); // Incomplete
let (paren_rest, paren_list_of_units) = parse_expr_list(false, "() () [] [] ())").expect("Valid");
assert_eq!(paren_rest, ")");
assert_eq!(list_of_units, paren_list_of_units);
```

# Grammar
The grammar for a list of compound expressions can be represented by the following EBNF fragment:
```ebnf
Compound ::= (WS Compound)+
```
*/
pub fn parse_expr_list(complete: bool, input: &str) -> IResult<&str, Vec<Expr>> {
    preceded(
        opt(ws),
        separated_nonempty_list(|input| parse_ws(complete, input), parse_compound),
    )(input)
}

/**
Parse an S-expression
*/
pub fn parse_sexpr(input: &str) -> IResult<&str, Sexpr> {
    map(
        delimited(
            tag(SEXPR_OPEN),
            opt(|input| parse_expr_list(false, input)),
            preceded(opt(ws), tag(SEXPR_CLOSE)),
        ),
        |s| s.map(Sexpr).unwrap_or_default(),
    )(input)
}

/**
Parse a tuple
*/
pub fn parse_tuple(input: &str) -> IResult<&str, Tuple> {
    map(
        delimited(
            tag(TUPLE_OPEN),
            opt(|input| parse_expr_list(false, input)),
            preceded(opt(ws), tag(TUPLE_CLOSE)),
        ),
        |s| s.map(Tuple).unwrap_or_default(),
    )(input)
}

/**
Parse a product type
*/
pub fn parse_product(input: &str) -> IResult<&str, Product> {
    map(
        delimited(
            preceded(tag(KEYWORD_PROD), tag(TUPLE_OPEN)),
            opt(|input| parse_expr_list(false, input)),
            preceded(opt(ws), tag(TUPLE_CLOSE)),
        ),
        |s| s.map(Product).unwrap_or_default(),
    )(input)
}

/**
Parse a typeof expression
*/
pub fn parse_typeof(input: &str) -> IResult<&str, TypeOf> {
    map(
        delimited(
            preceded(tag(KEYWORD_TYPEOF), tag("(")),
            preceded(opt(ws), parse_expr),
            preceded(opt(ws), tag(")")),
        ),
        |expr| TypeOf(Box::new(expr)),
    )(input)
}

/**
Parse a judgemental equality check
*/
pub fn parse_jeq(input: &str) -> IResult<&str, Jeq> {
    map(
        delimited(
            preceded(tag(KEYWORD_JEQ), tag(TUPLE_OPEN)),
            opt(|input| parse_expr_list(false, input)),
            preceded(opt(ws), tag(TUPLE_CLOSE)),
        ),
        |s| s.map(Jeq).unwrap_or_default(),
    )(input)
}

/**
Parse a 128-bit integer
*/
pub fn parse_u128(input: &str) -> IResult<&str, u128> {
    alt((
        map_res(preceded(tag("0x"), hex_digit1), |input| {
            u128::from_str_radix(input, 16)
        }),
        map_res(preceded(tag("0o"), oct_digit1), |input| {
            u128::from_str_radix(input, 8)
        }),
        map_res(preceded(tag("0b"), is_a_c("01")), |input| {
            u128::from_str_radix(input, 2)
        }),
        map_res(digit1, |input| u128::from_str_radix(input, 10)),
    ))(input)
}

/**
Parse a `usize`
*/
pub fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(parse_u128, TryFrom::try_from)(input)
}

/**
Parse a `u8`
*/
pub fn parse_u8(input: &str) -> IResult<&str, u8> {
    map_res(parse_u128, TryFrom::try_from)(input)
}

/**
Parse a finite type
*/
pub fn parse_finite(input: &str) -> IResult<&str, Finite> {
    delimited(
        preceded(tag(KEYWORD_FINITE), tag("(")),
        preceded(opt(ws), map(parse_u128, Finite)),
        preceded(opt(ws), tag(")")),
    )(input)
}

/**
Parse an index into a finite type
*/
pub fn parse_ix(input: &str) -> IResult<&str, Index> {
    preceded(
        tag(KEYWORD_IX),
        map(
            tuple((
                opt(delimited(
                    preceded(tag("("), opt(ws)),
                    map(parse_u128, Finite),
                    preceded(opt(ws), tag(")")),
                )),
                delimited(
                    preceded(tag("["), opt(ws)),
                    parse_u128,
                    preceded(opt(ws), tag("]")),
                ),
            )),
            |(ty, ix)| Index { ty, ix },
        ),
    )(input)
}

/**
Parse an inner scope
*/
pub fn parse_inner_scope(input: &str) -> IResult<&str, Scope> {
    map(
        tuple((
            preceded(opt(ws), many0(parse_statement)),
            preceded(opt(ws), opt(parse_expr)),
        )),
        |(statements, expr)| Scope {
            statements,
            retv: expr.map(Box::new),
        },
    )(input)
}

/**
Parse a scope
*/
pub fn parse_scope(input: &str) -> IResult<&str, Scope> {
    delimited(
        preceded(tag(SCOPE_OPEN), opt(ws)),
        parse_inner_scope,
        preceded(tag(SCOPE_CLOSE), opt(ws)),
    )(input)
}

/**
Parse a raw logical operation
*/
pub fn parse_raw_logical(input: &str) -> IResult<&str, Logical> {
    map_res(
        delimited(
            preceded(tag(KEYWORD_LOGICAL), tag(SEXPR_OPEN)),
            preceded(
                opt(ws),
                separated_pair(
                    parse_u8,
                    delimited(opt(ws), tag(SPECIAL_DELIM), opt(ws)),
                    parse_u128,
                ),
            ),
            preceded(opt(ws), tag(SEXPR_CLOSE)),
        ),
        |(arity, data)| Logical::try_new(arity, data),
    )(input)
}

/**
Parse a logical operation
*/
pub fn parse_logical(input: &str) -> IResult<&str, Logical> {
    alt((
        parse_raw_logical,
        map(tag(KEYWORD_AND), |_| And.into()),
        map(tag(KEYWORD_OR), |_| Or.into()),
        map(tag(KEYWORD_XOR), |_| Xor.into()),
        map(tag(KEYWORD_NOT), |_| Not.into()),
        map(tag(KEYWORD_LOGICAL_ID), |_| LogicalId.into()),
        map(tag(KEYWORD_NAND), |_| Nand.into()),
        map(tag(KEYWORD_NOR), |_| Nor.into()),
        map(tag(KEYWORD_IFF), |_| Iff.into()),
    ))(input)
}

/**
Parse an atomic `rain` expression. Does *not* consume whitespace before the expression!

These expressions can have paths attached
*/
pub fn parse_atom(input: &str) -> IResult<&str, Expr> {
    map(
        tuple((
            alt((
                map(parse_sexpr, Expr::Sexpr),
                map(parse_tuple, Expr::Tuple),
                map(parse_scope, Expr::Scope),
                map(parse_ident, Expr::Ident),
                map(parse_bool, Expr::Bool),
                map(parse_bool_ty, Expr::BoolTy),
                map(parse_typeof, Expr::TypeOf),
                map(parse_finite, Expr::Finite),
                map(parse_ix, Expr::Index),
                map(parse_product, Expr::Product),
                map(parse_jeq, Expr::Jeq),
                map(parse_logical, Expr::Logical),
                map(tag(KEYWORD_UNIT), |_| Expr::Unit),
            )),
            opt(parse_path),
        )),
        |(atom, path): (Expr, Option<Path>)| {
            if let Some(path) = path {
                let base = Box::new(atom);
                Expr::Member(Member { base, path })
            } else {
                atom
            }
        },
    )(input)
}

/**
Parse an argument to a parametrized expression
*/
pub fn parse_param_arg(input: &str) -> IResult<&str, (Ident, Expr)> {
    map(
        tuple((parse_ident, opt(ws), tag(JUDGE_TYPE), opt(ws), parse_atom)),
        |(id, _, _, _, atom)| (id, atom),
    )(input)
}

/**
Parse the arguments of a parametrized expression
*/
pub fn parse_param_args(input: &str) -> IResult<&str, ParamArgs> {
    delimited(
        preceded(tag(PARAM_OPEN), opt(ws)),
        map(separated_nonempty_list(ws, parse_param_arg), ParamArgs),
        preceded(opt(ws), tag(PARAM_CLOSE)),
    )(input)
}

/**
Parse a parametrized expression
*/
pub fn parse_parametrized(input: &str) -> IResult<&str, Parametrized> {
    map(
        tuple((parse_param_args, opt(ws), parse_expr)),
        |(args, _, expr)| Parametrized {
            args,
            result: Box::new(expr),
        },
    )(input)
}

/**
Parse a lambda function
*/
pub fn parse_lambda(input: &str) -> IResult<&str, Lambda> {
    map(parse_parametrized, Lambda)(input)
}

/**
Parse a pi type
*/
pub fn parse_pi(input: &str) -> IResult<&str, Pi> {
    map(
        preceded(preceded(tag(KEYWORD_PI), opt(ws)), parse_parametrized),
        Pi,
    )(input)
}

/**
Parse a compound `rain` expression. Does *not* consume whitespace before the expression!
*/
pub fn parse_compound(input: &str) -> IResult<&str, Expr> {
    alt((
        map(parse_pi, Expr::Pi),
        map(parse_lambda, Expr::Lambda),
        parse_atom,
    ))(input)
}

/// Parse a simple assignment
pub fn parse_simple_assign(input: &str) -> IResult<&str, Simple> {
    map(
        tuple((
            parse_ident,
            opt(preceded(
                delimited(opt(ws), tag(JUDGE_TYPE), opt(ws)),
                parse_compound,
            )),
        )),
        |(var, ty)| Simple { var, ty },
    )(input)
}

/// Parse a tuple destructure pattern
pub fn parse_detuple(input: &str) -> IResult<&str, Detuple> {
    map(
        delimited(
            preceded(tag(TUPLE_OPEN), opt(ws)),
            separated_list(ws, parse_pattern),
            preceded(opt(ws), tag(TUPLE_CLOSE)),
        ),
        |elems| Detuple(elems),
    )(input)
}

/// Parse a pattern
pub fn parse_pattern(input: &str) -> IResult<&str, Pattern> {
    alt((
        map(parse_simple_assign, Pattern::Simple),
        map(parse_detuple, Pattern::Detuple),
    ))(input)
}

/// Parse a let-statement
pub fn parse_let(input: &str) -> IResult<&str, Let> {
    map(
        tuple((
            tag(KEYWORD_LET),
            ws,
            parse_pattern,
            opt(ws),
            tag(ASSIGN),
            opt(ws),
            parse_expr,
            opt(ws),
            tag(STATEMENT_DELIM),
        )),
        |(_, _, lhs, _, _, _, rhs, _, _)| Let { lhs, rhs },
    )(input)
}

/// Parse a statement
pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
    map(parse_let, Statement::Let)(input)
}

/// Parse a standalone `rain` expression
pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    map(
        |input| parse_expr_list(true, input),
        |e| Expr::Sexpr(Sexpr(e)),
    )(input)
}
