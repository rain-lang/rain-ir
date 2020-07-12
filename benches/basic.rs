use criterion::{criterion_group, criterion_main, Criterion};
use rain_ir::function::lambda::Lambda;
use rain_ir::primitive::{
    finite::Finite,
    logical::{And, Bool, Not, Or},
};
use rain_ir::region::Region;
use rain_ir::typing::Typed;
use rain_ir::value::{expr::Sexpr, ValId};
use rand::{thread_rng, Rng};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("finite id", |b| {
        let mut rng = thread_rng();
        let finite = Finite(rng.gen::<u128>().max(1));
        let ix: u128 = rng.gen_range(0, finite.0);
        b.iter(|| {
            let ix = finite.ix(ix).unwrap();
            let region = Region::with(vec![ix.ty().clone_ty()].into(), None);
            let param = region.clone().param(0).unwrap();
            let id = Lambda::try_new(param.into(), region).unwrap();
            let ixv: ValId = ix.into();
            let sexpr: ValId = Sexpr::try_new(vec![id.into(), ixv.clone()]).unwrap().into();
            assert_eq!(sexpr, ixv);
        })
    });

    c.bench_function("finite id eval", |b| {
        let mut rng = thread_rng();
        let finite = Finite(rng.gen::<u128>().max(1));
        let ix: u128 = rng.gen_range(0, finite.0);
        let ix = finite.ix(ix).unwrap();
        let region = Region::with(vec![ix.ty().clone_ty()].into(), None);
        let param = region.clone().param(0).unwrap();
        let id: ValId = Lambda::try_new(param.into(), region).unwrap().into();
        let ixv: ValId = ix.into();
        b.iter(|| {
            let sexpr: ValId = Sexpr::try_new(vec![id.clone(), ixv.clone()])
                .unwrap()
                .into();
            assert_eq!(sexpr, ixv);
        })
    });

    c.bench_function("boolean mux", |b| {
        let mut rng = thread_rng();
        let in_high: bool = rng.gen();
        let in_low: bool = rng.gen();
        let in_sel: bool = rng.gen();
        b.iter(|| {
            let region = Region::with(
                vec![Bool.into(), Bool.into(), Bool.into()].into(),
                None,
            );
            let select: ValId = region.clone().param(0).unwrap().into();
            let high = region.clone().param(1).unwrap();
            let low = region.clone().param(2).unwrap();
            let sel_high = Sexpr::try_new(vec![And.into(), select.clone(), high.into()])
                .unwrap()
                .into();
            let not_sel = Sexpr::try_new(vec![Not.into(), select.clone()])
                .unwrap()
                .into();
            let sel_low = Sexpr::try_new(vec![And.into(), not_sel, low.into()])
                .unwrap()
                .into();
            let result = Sexpr::try_new(vec![Or.into(), sel_high, sel_low])
                .unwrap()
                .into();
            let mux = Lambda::try_new(result, region).unwrap().into();
            let sexpr: ValId =
                Sexpr::try_new(vec![mux, in_sel.into(), in_high.into(), in_low.into()])
                    .unwrap()
                    .into();
            let result: ValId = if in_sel { in_high } else { in_low }.into();
            assert_eq!(sexpr, result);
        })
    });

    c.bench_function("boolean mux eval", |b| {
        let mut rng = thread_rng();
        let in_sel: bool = rng.gen();
        let in_high: bool = rng.gen();
        let in_low: bool = rng.gen();
        let in_high: ValId = in_high.into();
        let in_low: ValId = in_low.into();
        let res = if in_sel {
            in_high.clone()
        } else {
            in_low.clone()
        };
        let in_sel: ValId = in_sel.into();
        let region = Region::with(
            vec![Bool.into(), Bool.into(), Bool.into()].into(),
            None,
        );
        let select: ValId = region.clone().param(0).unwrap().into();
        let high = region.clone().param(1).unwrap();
        let low = region.clone().param(2).unwrap();
        let sel_high = Sexpr::try_new(vec![And.into(), select.clone(), high.into()])
            .unwrap()
            .into();
        let not_sel = Sexpr::try_new(vec![Not.into(), select.clone()])
            .unwrap()
            .into();
        let sel_low = Sexpr::try_new(vec![And.into(), not_sel, low.into()])
            .unwrap()
            .into();
        let result = Sexpr::try_new(vec![Or.into(), sel_high, sel_low])
            .unwrap()
            .into();
        let mux: ValId = Lambda::try_new(result, region).unwrap().into();
        b.iter(|| {
            let sexpr: ValId = Sexpr::try_new(vec![
                mux.clone(),
                in_sel.clone(),
                in_high.clone(),
                in_low.clone(),
            ])
            .unwrap()
            .into();
            assert_eq!(sexpr, res);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
