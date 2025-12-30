use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use poker_rs::cards::{Card, Rank, Suit};
use poker_rs::evaluator::{evaluate_five, evaluate_seven};

fn bench_evaluate_five(c: &mut Criterion) {
    let hi = [
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::King, Suit::Diamonds),
        Card::new(Rank::Seven, Suit::Spades),
        Card::new(Rank::Five, Suit::Clubs),
        Card::new(Rank::Two, Suit::Diamonds),
    ];
    let sf = [
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Ten, Suit::Spades),
    ];

    let mut g = c.benchmark_group("evaluate_five");
    g.bench_with_input(BenchmarkId::new("high_card", "A,K,7,5,2"), &hi, |b, input| {
        b.iter(|| evaluate_five(black_box(input)))
    });
    g.bench_with_input(BenchmarkId::new("straight_flush", "royal"), &sf, |b, input| {
        b.iter(|| evaluate_five(black_box(input)))
    });
    g.finish();
}

fn bench_evaluate_seven(c: &mut Criterion) {
    let seven = [
        Card::new(Rank::Ace, Suit::Spades),
        Card::new(Rank::Ace, Suit::Hearts),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Ten, Suit::Spades),
        Card::new(Rank::Nine, Suit::Spades),
    ];
    c.bench_function("evaluate_seven", |b| b.iter(|| evaluate_seven(black_box(&seven))));
}

criterion_group!(benches, bench_evaluate_five, bench_evaluate_seven);
criterion_main!(benches);
