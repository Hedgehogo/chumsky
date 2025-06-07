use chumsky::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

mod utils;

fn bench_choice(c: &mut Criterion) {
    let alphabet_choice = choice((
        just::<_, &str, extra::Default>('A'),
        just('B'),
        just('C'),
        just('D'),
        just('E'),
        just('F'),
        just('G'),
        just('H'),
        just('I'),
        just('J'),
        just('K'),
        just('L'),
        just('M'),
        just('N'),
        just('O'),
        just('P'),
        just('Q'),
        just('R'),
        just('S'),
        just('T'),
        just('U'),
        just('V'),
        just('W'),
        just('X'),
        just('Y'),
        just('Z'),
    )).boxed();

    let mut group = c.benchmark_group("choice");

    group.bench_function(BenchmarkId::new("choice::<(A..Z)>", "A"), |b| {
        b.iter(|| {
            black_box(alphabet_choice.parse(black_box("A")))
                .into_result()
                .unwrap();
        })
    });

    group.bench_function(BenchmarkId::new("choice::<(A..Z)>", "Z"), |b| {
        b.iter(|| {
            black_box(alphabet_choice.parse(black_box("Z")))
                .into_result()
                .unwrap();
        })
    });

    group.bench_function(BenchmarkId::new("choice::<(A..Z)>", "0"), |b| {
        b.iter(|| {
            assert!(black_box(alphabet_choice.parse(black_box("0")))
                .into_result()
                .is_err());
        })
    });
}

fn bench_or(c: &mut Criterion) {
    let alphabet_or = just::<_, _, extra::Default>('A')
        .or(just('B')).boxed()
        .or(just('C')).boxed()
        .or(just('D')).boxed()
        .or(just('E')).boxed()
        .or(just('F')).boxed()
        .or(just('G')).boxed()
        .or(just('H')).boxed()
        .or(just('I')).boxed()
        .or(just('J')).boxed()
        .or(just('K')).boxed()
        .or(just('L')).boxed()
        .or(just('M')).boxed()
        .or(just('N')).boxed()
        .or(just('O')).boxed()
        .or(just('P')).boxed()
        .or(just('Q')).boxed()
        .or(just('R')).boxed()
        .or(just('S')).boxed()
        .or(just('T')).boxed()
        .or(just('U')).boxed()
        .or(just('V')).boxed()
        .or(just('W')).boxed()
        .or(just('X')).boxed()
        .or(just('Y')).boxed()
        .or(just('Z')).boxed();

    let mut group = c.benchmark_group("or");

    group.bench_function(BenchmarkId::new("A.or(B)...or(Z)", "A"), |b| {
        b.iter(|| {
            black_box(alphabet_or.parse(black_box("A")))
                .into_result()
                .unwrap();
        })
    });

    group.bench_function(BenchmarkId::new("A.or(B)...or(Z)", "Z"), |b| {
        b.iter(|| {
            black_box(alphabet_or.parse(black_box("Z")))
                .into_result()
                .unwrap();
        })
    });

    group.bench_function(BenchmarkId::new("A.or(B)...or(Z)", "0"), |b| {
        b.iter(|| {
            assert!(black_box(alphabet_or.parse(black_box("0")))
                .into_result()
                .is_err());
        })
    });
}

fn bench_group(c: &mut Criterion) {
    let alphabet_group = group((
        just::<_, &str, extra::Default>('A'),
        just('B'),
        just('C'),
        just('D'),
        just('E'),
        just('F'),
        just('G'),
        just('H'),
        just('I'),
        just('J'),
        just('K'),
        just('L'),
        just('M'),
        just('N'),
        just('O'),
        just('P'),
        just('Q'),
        just('R'),
        just('S'),
        just('T'),
        just('U'),
        just('V'),
        just('W'),
        just('X'),
        just('Y'),
        just('Z'),
    )).boxed();

    let mut group = c.benchmark_group("group");

    group.bench_function(
        BenchmarkId::new("group::<(A..Z)>", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        |b| {
            b.iter(|| {
                black_box(alphabet_group.parse(black_box("ABCDEFGHIJKLMNOPQRSTUVWXYZ")))
                    .into_result()
                    .unwrap();
            })
        },
    );

    group.bench_function(
        BenchmarkId::new("group::<(A..Z)>", "ABCDEFGHIJKLMNOPQRSTUVWXY0"),
        |b| {
            b.iter(|| {
                assert!(
                    black_box(alphabet_group.parse(black_box("ABCDEFGHIJKLMNOPQRSTUVWXY0")))
                        .into_result()
                        .is_err()
                );
            })
        },
    );

    group.bench_function(BenchmarkId::new("group::<(A..Z)>", "0"), |b| {
        b.iter(|| {
            assert!(black_box(alphabet_group.parse(black_box("0")))
                .into_result()
                .is_err());
        })
    });
}

fn bench_then(c: &mut Criterion) {
    let alphabet_then = just::<_, _, extra::Default>('A')
        .then(just('B')).boxed()
        .then(just('C')).boxed()
        .then(just('D')).boxed()
        .then(just('E')).boxed()
        .then(just('F')).boxed()
        .then(just('G')).boxed()
        .then(just('H')).boxed()
        .then(just('I')).boxed()
        .then(just('J')).boxed()
        .then(just('K')).boxed()
        .then(just('L')).boxed()
        .then(just('M')).boxed()
        .then(just('N')).boxed()
        .then(just('O')).boxed()
        .then(just('P')).boxed()
        .then(just('Q')).boxed()
        .then(just('R')).boxed()
        .then(just('S')).boxed()
        .then(just('T')).boxed()
        .then(just('U')).boxed()
        .then(just('V')).boxed()
        .then(just('W')).boxed()
        .then(just('X')).boxed()
        .then(just('Y')).boxed()
        .then(just('Z')).boxed();

    let mut group = c.benchmark_group("then");

    group.bench_function(
        BenchmarkId::new("A.then(B)...then(Z)", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        |b| {
            b.iter(|| {
                black_box(alphabet_then.parse(black_box("ABCDEFGHIJKLMNOPQRSTUVWXYZ")))
                    .into_result()
                    .unwrap();
            })
        },
    );

    group.bench_function(
        BenchmarkId::new("A.then(B)...then(Z)", "ABCDEFGHIJKLMNOPQRSTUVWXY0"),
        |b| {
            b.iter(|| {
                assert!(
                    black_box(alphabet_then.parse(black_box("ABCDEFGHIJKLMNOPQRSTUVWXY0")))
                        .into_result()
                        .is_err()
                );
            })
        },
    );

    group.bench_function(BenchmarkId::new("A.then(B)...then(Z)", "0"), |b| {
        b.iter(|| {
            assert!(black_box(alphabet_then.parse(black_box("0")))
                .into_result()
                .is_err());
        })
    });
}

#[cfg(feature = "regex")]
fn bench_regex(c: &mut Criterion) {
    let re_foo = regex::<_, extra::Default>("foo").boxed();
    let re_foo2 = regex::<_, extra::Default>("[fF]oo").boxed();
    let re_rep = regex::<_, extra::Default>("(?:abc){4}").boxed();

    let mut group = c.benchmark_group("regex");

    group.bench_function(BenchmarkId::new("foo", "foo"), |b| {
        b.iter(|| {
            black_box(re_foo.parse(black_box("foo")))
                .into_result()
                .unwrap();
        })
    });

    group.bench_function(BenchmarkId::new("foo", "barfoofoofoo"), |b| {
        b.iter(|| {
            black_box(re_foo.parse(black_box("barfoofoofoo")))
                .into_result()
                .unwrap_err();
        })
    });

    group.bench_function(BenchmarkId::new("[fF]oo", "foo"), |b| {
        b.iter(|| {
            black_box(re_foo2.parse(black_box("foo")))
                .into_result()
                .unwrap()
        })
    });

    group.bench_function(BenchmarkId::new("[fF]oo", "Foo"), |b| {
        b.iter(|| {
            black_box(re_foo2.parse(black_box("Foo")))
                .into_result()
                .unwrap()
        })
    });

    group.bench_function(BenchmarkId::new("[fF]oo", "barFoofoo"), |b| {
        b.iter(|| {
            black_box(re_foo2.parse(black_box("barFoofoo")))
                .into_result()
                .unwrap_err()
        })
    });

    group.bench_function(BenchmarkId::new("(?:abc){4}", "abcabcabcabc"), |b| {
        b.iter(|| {
            black_box(re_rep.parse(black_box("abcabcabcabc")))
                .into_result()
                .unwrap()
        })
    });
}

#[cfg(not(feature = "regex"))]
fn bench_regex(_: &mut Criterion) {}

criterion_group!(
    name = benches;
    config = utils::make_criterion();
    targets = bench_choice, bench_or, bench_group, bench_then, bench_regex,
);
criterion_main!(benches);
