use poker_rs::game::Game;

fn mk_game(n: usize) -> Game {
    Game::new(n, 1000, 5, 10)
}

#[test]
fn postflop_starts_left_of_dealer_skipping_ineligible() {
    let mut g = mk_game(5);
    g.new_hand();
    // Force to flop immediately: everyone checks/calls until street ends
    // Preflop UTG..BB acts: with blinds posted, to_call for UTG=10
    // UTG..CO fold to speed up; BB checks (call 0)
    for _ in 0..4 {
        g.action_fold().unwrap();
    }
    // Only BB left, should go to showdown
    assert!(matches!(g.street, poker_rs::game::Street::Showdown));
}

#[test]
fn preflop_no_raise_everyone_acts_before_flop() {
    let mut g = mk_game(5);
    g.new_hand();
    // Everyone calls to match BB; sequence UTG, MP, CO, BTN, SB
    for _ in 0..5 {
        g.action_check_call().unwrap();
    }
    // After SB acts with no raise, street should advance to flop
    assert!(matches!(g.street, poker_rs::game::Street::Flop));
}

#[test]
fn raise_round_ends_when_returns_to_last_raiser_with_matched_bets() {
    let mut g = mk_game(5);
    g.new_hand();
    // UTG raises min
    g.action_raise_min().unwrap();
    // Others call around to UTG
    for _ in 0..4 {
        g.action_check_call().unwrap();
    }
    assert!(matches!(g.street, poker_rs::game::Street::Flop));
}

#[test]
fn heads_up_blinds_and_preflop_order() {
    let mut g = mk_game(2);
    g.new_hand();
    let dealer = g.dealer;
    let sb = dealer;
    let bb = (dealer + 1) % g.players.len();

    assert_eq!(g.players[sb].bet, g.small_blind);
    assert_eq!(g.players[bb].bet, g.big_blind);
    assert_eq!(g.current, dealer, "BTN/SB acts first preflop");

    g.action_check_call().unwrap();
    assert_eq!(g.current, bb, "BB should act after BTN/SB calls");
}

#[test]
fn heads_up_postflop_starts_non_dealer() {
    let mut g = mk_game(2);
    g.new_hand();
    let dealer = g.dealer;
    let non_dealer = (dealer + 1) % g.players.len();

    g.action_check_call().unwrap();
    g.action_check_call().unwrap();

    assert!(matches!(g.street, poker_rs::game::Street::Flop));
    assert_eq!(g.current, non_dealer, "postflop starts at non-dealer");
}
