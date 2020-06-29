/*!
Test `rain` by compiling a variety of sample programs, and checking their output is correct.
*/
use rain_ir::parser::builder::Builder;

/// Projections from `(bool, bool)` pairs to a member.
/// 
/// Reported not to work in issue #35, added as a regression test
#[test]
fn boolean_pair_projections() {
    let mut builder = Builder::<&str>::new();
    let (rest, pi0) = builder .parse_expr("|x: #product[#bool #bool]| (x #ix(2)[0])") .expect("Valid lambda");
    assert_eq!(rest, "");
    unimplemented!("Rest of the test: {}", pi0);
}