//! Integration tests: verify fight logic through the library's public API only.

use rust_funny::game::{FighterState, Game, Projectile, Side, StrikeKind, MAX_HP};

#[test]
fn round_flow_ko_then_reset() {
    let mut game = Game::new();
    assert_eq!(game.left.hp, MAX_HP);
    assert_eq!(game.right.hp, MAX_HP);
    assert_eq!(game.left_score, 0);
    assert_eq!(game.right_score, 0);
    assert!(!game.round_over);

    // Cause a KO: left side hits 0 HP, right side wins.
    game.left.hp = 0;
    game.tick();
    assert!(game.round_over);
    assert_eq!(game.right_score, 1);
    assert_eq!(game.ko_side, Some(Side::Right));
    assert_eq!(game.left.state, FighterState::Down);

    // After the ROUND_PAUSE (30 frames) it auto-resets.
    for _ in 0..30 {
        game.tick();
    }
    assert!(!game.round_over);
    assert_eq!(game.left.hp, MAX_HP);
    assert_eq!(game.right.hp, MAX_HP);
    assert_eq!(game.ko_side, None);
}

#[test]
fn long_run_produces_scores_and_valid_hp() {
    let mut game = Game::new();
    for _ in 0..50_000 {
        game.tick();
    }
    assert!(
        game.left_score + game.right_score > 0,
        "a long run should produce a score"
    );
    assert!((0..=MAX_HP).contains(&game.left.hp));
    assert!((0..=MAX_HP).contains(&game.right.hp));
}

#[test]
fn projectile_deals_damage_on_contact() {
    let mut game = Game::new();
    let before = game.right.hp;
    game.projectiles.push(Projectile {
        owner: Side::Left,
        kind: StrikeKind::Punch,
        progress: 0.99,
        dmg: 12,
    });

    game.tick();

    assert!(
        game.right.hp < before,
        "the projectile should reduce HP on contact"
    );
    assert!(
        game.projectiles.is_empty(),
        "the projectile should disappear on contact"
    );
}
