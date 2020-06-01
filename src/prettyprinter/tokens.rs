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

/// The standard representation for the unit value
pub const UNIT_VALUE: &str = "()";

/// The standard representation for the unit type
pub const UNIT_TYPE: &str = "#unit";