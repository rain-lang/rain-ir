/*!
Recursor nodes, describing primitive recursion and control flow on `n`-ary sum types.

# Implementation Notes
The difference between a `switch` statement and a `rec` statement is that the former is implemented with run-length encoding, while
the later is implemented with a `ValArr`. "Compression normalization" is in effect: a `switch` statement which is larger than a
corresponding `rec` statement will normalize to the latter, and vice versa. We assume here that a pointer is 64 bits.
*/
