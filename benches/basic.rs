use criterion::{criterion_group, criterion_main, Criterion};
use rain_ir::function::lambda::Lambda;
use rain_ir::primitive::finite::Index;
use rain_ir::region::Region;
use rain_ir::typing::Typed;
use rain_ir::value::{expr::Sexpr, ValId};
use rand::{thread_rng, Rng};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("finite id", |b| {
        let mut rng = thread_rng();
        b.iter(|| {
            let ix: Index = rng.gen();
            let region = Region::with(vec![ix.ty().clone_ty()].into(), Region::default());
            let param = region.clone().param(0).unwrap();
            let id = Lambda::try_new(param.into(), region).unwrap();
            let ixv: ValId = ix.into();
            let sexpr: ValId = Sexpr::try_new(vec![id.into(), ixv.clone()]).unwrap().into();
            assert_eq!(sexpr, ixv);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
