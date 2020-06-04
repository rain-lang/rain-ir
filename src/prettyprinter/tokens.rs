/*!
The basic tokens making up the standard `rain` representation
*/

/// The `rain` special characters, including whitespace
pub const SPECIAL_CHARACTERS: &str = " \t\r\n#()[]|\"\':.;/";

/// The `rain` whitespace characters
pub const WHITESPACE: &str = " \t\r\n";

/// The `rain` path separator charactor
pub const PATH_SEP: &str = ".";

/// The `rain` typing judgement character
pub const JUDGE_TYPE: &str = ":";

/// The `rain` assignment character
pub const ASSIGN: &str = "=";

/// The `rain` keyword for `let`-statements
pub const KEYWORD_LET: &str = "#let";

/// The `rain` keyword for the boolean type
pub const KEYWORD_BOOL: &str = "#bool";

/// The `rain` keyword for `true`
pub const KEYWORD_TRUE: &str = "#true";

/// The `rain` keyword for `false`
pub const KEYWORD_FALSE: &str = "#false";

/// The `rain` keyword for `typeof`
pub const KEYWORD_TYPEOF: &str = "#typeof";

/// The `rain` keyword for finite types
pub const KEYWORD_FINITE: &str = "#finite";

/// The `rain` keyword for indices into finite types
pub const KEYWORD_IX: &str = "#ix";

/// The delimiter for `rain` statements
pub const STATEMENT_DELIM: &str = ";";

/// The null `rain` symbol
pub const NULL_SYMBOL: &str = "_";

/// The delimiter for single-line `rain` comments
pub const SINGLE_COMMENT_START: &str = "//";

/// The opening delimiter for a multi-line `rain` comment
pub const MULTI_COMMENT_OPEN: &str = "/*";

/// The closing delimiter for a multi-line `rain` comment
pub const MULTI_COMMENT_CLOSE: &str = "*/";

/// The opening delimiter for a parse_sexpr
pub const SEXPR_OPEN: &str = "(";

/// The closing delimiter for a parse_sexpr
pub const SEXPR_CLOSE: &str = ")";

/// The opening delimiter for a parse_tuple
pub const TUPLE_OPEN: &str = "[";

/// The closing delimiter for a parse_tuple
pub const TUPLE_CLOSE: &str = "]";

/// The opening delimiter for a scope
pub const SCOPE_OPEN: &str = "{";

/// The closing delimiter for a scope
pub const SCOPE_CLOSE: &str = "}";

/// The opening delimiter for a parameter list
pub const PARAM_OPEN: &str = "|";

/// The closing delimiter for a parameter list
pub const PARAM_CLOSE: &str = "|";

/// The standard representation for the unit value
pub const UNIT_VALUE: &str = "()";

/// The standard representation for the unit type
pub const UNIT_TYPE: &str = "#unit";

/// The keyword for product types
pub const KEYWORD_PROD: &str = "#product";

/// The keyword for pi types
pub const KEYWORD_PI: &str = "#pi";