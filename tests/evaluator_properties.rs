use poker_rs::cards::{Card, Rank, Suit};
use poker_rs::evaluator::{evaluate_five, evaluate_seven, Category};
use proptest::prelude::*;
use std::cmp::Ordering;

prop_compose! {
    fn any_rank()(v in 2u8..=14u8) -> Rank {
        match v {
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Jack,
            12 => Rank::Queen,
            13 => Rank::King,
            _ => Rank::Ace,
        }
    }
}

fn any_suit() -> impl Strategy<Value = Suit> {
    prop_oneof![Just(Suit::Clubs), Just(Suit::Diamonds), Just(Suit::Hearts), Just(Suit::Spades),]
}

fn any_card() -> impl Strategy<Value = Card> {
    (any_rank(), any_suit()).prop_map(|(r, s)| Card::new(r, s))
}

fn rank_from_val(v: u8) -> Rank {
    match v {
        2 => Rank::Two,
        3 => Rank::Three,
        4 => Rank::Four,
        5 => Rank::Five,
        6 => Rank::Six,
        7 => Rank::Seven,
        8 => Rank::Eight,
        9 => Rank::Nine,
        10 => Rank::Ten,
        11 => Rank::Jack,
        12 => Rank::Queen,
        13 => Rank::King,
        _ => Rank::Ace,
    }
}

fn straight_cards(top: u8) -> [Card; 5] {
    let ranks = if top == 5 {
        [Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five]
    } else {
        [
            rank_from_val(top - 4),
            rank_from_val(top - 3),
            rank_from_val(top - 2),
            rank_from_val(top - 1),
            rank_from_val(top),
        ]
    };
    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades, Suit::Clubs];
    [
        Card::new(ranks[0], suits[0]),
        Card::new(ranks[1], suits[1]),
        Card::new(ranks[2], suits[2]),
        Card::new(ranks[3], suits[3]),
        Card::new(ranks[4], suits[4]),
    ]
}

fn ranks_desc(ranks: &[Rank]) -> Vec<Rank> {
    let mut out = ranks.to_vec();
    out.sort_by(|a, b| b.cmp(a));
    out
}

fn compare_rank_lists(a: &[Rank], b: &[Rank]) -> Ordering {
    for i in 0..a.len().min(b.len()) {
        let ord = a[i].cmp(&b[i]);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

fn flush_rank_set() -> impl Strategy<Value = Vec<Rank>> {
    prop::collection::btree_set(2u8..=14u8, 5)
        .prop_filter("non-straight ranks", |set| {
            let mut vals: Vec<u8> = set.iter().copied().collect();
            vals.sort_unstable();
            let is_wheel = vals == vec![2, 3, 4, 5, 14];
            let is_straight = vals.windows(2).all(|w| w[1] == w[0] + 1);
            !(is_straight || is_wheel)
        })
        .prop_map(|set| set.into_iter().map(rank_from_val).collect())
}

proptest! {
    #[test]
    fn five_card_ordering_is_antisymmetric_and_transitive(a in prop::array::uniform5(any_card()), b in prop::array::uniform5(any_card()), c in prop::array::uniform5(any_card())) {
        let ea = evaluate_five(&a);
        let eb = evaluate_five(&b);
        let ec = evaluate_five(&c);

        // antisymmetric: if a >= b and b >= a then a == b
        if ea >= eb && eb >= ea { prop_assert_eq!(ea, eb); }

        // transitive: if a >= b and b >= c then a >= c
        if ea >= eb && eb >= ec { prop_assert!(ea >= ec); }
    }

    #[test]
    fn seven_card_best_is_at_least_as_good_as_any_five(cards in prop::array::uniform7(any_card())) {
        let best7 = evaluate_seven(&cards);
        // Check against each 5-subset deterministically
        for i in 0..3 { for j in (i+1)..4 { for k in (j+1)..5 { for l in (k+1)..6 { for m in (l+1)..7 {
            let five = [cards[i], cards[j], cards[k], cards[l], cards[m]];
            let e5 = evaluate_five(&five);
            prop_assert!(best7 >= e5);
        }}}}}
    }

    #[test]
    fn straight_ordering_respects_top_card(top_hi in 6u8..=14u8, top_lo in 5u8..=13u8) {
        prop_assume!(top_hi > top_lo);
        let hi = straight_cards(top_hi);
        let lo = straight_cards(top_lo);
        let e_hi = evaluate_five(&hi);
        let e_lo = evaluate_five(&lo);
        prop_assert!(matches!(e_hi.category, Category::Straight));
        prop_assert!(matches!(e_lo.category, Category::Straight));
        prop_assert!(e_hi > e_lo);
    }

    #[test]
    fn wheel_is_lowest_straight(top in 6u8..=14u8) {
        let wheel = straight_cards(5);
        let higher = straight_cards(top);
        let e_wheel = evaluate_five(&wheel);
        let e_high = evaluate_five(&higher);
        prop_assert!(matches!(e_wheel.category, Category::Straight));
        prop_assert!(matches!(e_high.category, Category::Straight));
        prop_assert!(e_high > e_wheel);
    }

    #[test]
    fn flush_kicker_ordering(a in flush_rank_set(), b in flush_rank_set()) {
        let suit = Suit::Hearts;
        let hand_a = [
            Card::new(a[0], suit),
            Card::new(a[1], suit),
            Card::new(a[2], suit),
            Card::new(a[3], suit),
            Card::new(a[4], suit),
        ];
        let hand_b = [
            Card::new(b[0], suit),
            Card::new(b[1], suit),
            Card::new(b[2], suit),
            Card::new(b[3], suit),
            Card::new(b[4], suit),
        ];
        let e_a = evaluate_five(&hand_a);
        let e_b = evaluate_five(&hand_b);
        prop_assert!(matches!(e_a.category, Category::Flush));
        prop_assert!(matches!(e_b.category, Category::Flush));

        let a_desc = ranks_desc(&a);
        let b_desc = ranks_desc(&b);
        match compare_rank_lists(&a_desc, &b_desc) {
            Ordering::Greater => prop_assert!(e_a > e_b),
            Ordering::Less => prop_assert!(e_a < e_b),
            Ordering::Equal => prop_assert_eq!(e_a, e_b),
        }
    }
}
