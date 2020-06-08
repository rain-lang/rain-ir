/*!
The basic tokens making up the standard `rain` representation
*/

/// The `rain` special characters, including whitespace
pub const SPECIAL_CHARACTERS: &str = " \t\r\n#()[]{}|\"\':.;/=";

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

/// The `rain` keyword for judgemental equality
pub const KEYWORD_JEQ: &str = "#jeq";

/// The `rain` keyword for `false`
pub const KEYWORD_FALSE: &str = "#false";

/// The `rain` keyword for `typeof`
pub const KEYWORD_TYPEOF: &str = "#typeof";

/// The `rain` keyword for the unit type
pub const KEYWORD_UNIT: &str = "#unit";

/// The `rain` keyword for finite types
pub const KEYWORD_FINITE: &str = "#finite";

/// The `rain` keyword for indices into finite types
pub const KEYWORD_IX: &str = "#ix";

/// The delimiter for `rain` statements
pub const STATEMENT_DELIM: &str = ";";

/// The delimiter for special arguments
pub const SPECIAL_DELIM: &str = ",";

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

/// The keyword for gamma nodes
pub const KEYWORD_GAMMA: &str = "#gamma";

/// The keyword for a logical operation
pub const KEYWORD_LOGICAL: &str = "#logical";

/// The keyword for phi nodes
pub const KEYWORD_PHI: &str = "#phi";

/// The keyword for logical identity
pub const KEYWORD_LOGICAL_ID: &str = "#bool_id";

/// The keyword for logical not
pub const KEYWORD_NOT: &str = "#not";

/// The keyword for logical and
pub const KEYWORD_AND: &str = "#and";

/// The keyword for logical or
pub const KEYWORD_OR: &str = "#or";

/// The keyword for logical xor
pub const KEYWORD_XOR: &str = "#xor";

/// The keyword for logical nor
pub const KEYWORD_NOR: &str = "#nor";

/// The keyword for logical nand
pub const KEYWORD_NAND: &str = "#nand";

/// The keyword for logical equality (iff)
pub const KEYWORD_IFF: &str = "#iff";