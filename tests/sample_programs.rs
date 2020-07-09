/*!
Test `rain` by compiling a variety of sample programs, and checking their output is correct.
*/
use rain_ir::builder::Builder;
use rain_ir::value::{expr::Sexpr, tuple::Tuple, Value};

/// Indexed projections from `(bool, bool)` pairs to a member.
///
/// Reported not to work in issue #35, added as a regression test
#[test]
fn boolean_pair_ix_projections() {
    let mut builder = Builder::<&str>::new();
    let (rest, pi0) = builder
        .parse_expr("|x: #product[#bool #bool]| (x #ix(2)[0])")
        .expect("Valid lambda");
    assert_eq!(rest, "");
    let (rest, pi1) = builder
        .parse_expr("|x: #product[#bool #bool]| (x #ix(2)[1])")
        .expect("Valid lambda");
    assert_eq!(rest, "");
    for l in [true, false].iter().copied() {
        for r in [true, false].iter().copied() {
            let input =
                Tuple::try_new(vec![l.into_val(), r.into_val()].into()).expect("Valid input tuple");
            assert_eq!(
                Sexpr::try_new(vec![pi0.clone(), input.clone().into()])
                    .expect("Valid S-expression")
                    .into_norm(),
                l.into_norm()
            );
            assert_eq!(
                Sexpr::try_new(vec![pi1.clone(), input.clone().into()])
                    .expect("Valid S-expression")
                    .into_norm(),
                r.into_norm()
            )
        }
    }
}

/// Member projections from `(bool, bool)` pairs to a member.
#[test]
fn boolean_pair_mem_projections() {
    let mut builder = Builder::<&str>::new();
    let (rest, pi0) = builder
        .parse_expr("|x: #product[#bool #bool]| x.0")
        .expect("Valid lambda");
    assert_eq!(rest, "");
    let (rest, pi1) = builder
        .parse_expr("|x: #product[#bool #bool]| x.1")
        .expect("Valid lambda");
    assert_eq!(rest, "");
    for l in [true, false].iter().copied() {
        for r in [true, false].iter().copied() {
            let input =
                Tuple::try_new(vec![l.into_val(), r.into_val()].into()).expect("Valid input tuple");
            assert_eq!(
                Sexpr::try_new(vec![pi0.clone(), input.clone().into()])
                    .expect("Valid S-expression")
                    .into_norm(),
                l.into_norm()
            );
            assert_eq!(
                Sexpr::try_new(vec![pi1.clone(), input.clone().into()])
                    .expect("Valid S-expression")
                    .into_norm(),
                r.into_norm()
            )
        }
    }
}