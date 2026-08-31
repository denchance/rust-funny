//! Integration tests: verify layout, scaling, and elements through the public rendering API.

use rust_funny::game::{FighterState, Game, Projectile, Side, StrikeKind};
use rust_funny::render::compose_plain;

/// Extracts the arena width from the top border row (counts the horizontal `─` glyphs plus the two corners).
/// Note: don't subtract `str::find` results directly — border characters are multi-byte UTF-8, so those offsets are byte offsets.
fn arena_width(s: &str) -> Option<usize> {
    let line = s.lines().find(|l| l.contains('┌'))?;
    Some(line.chars().filter(|&c| c == '─').count() + 2)
}

#[test]
fn arena_is_centered_one_third_of_terminal() {
    let game = Game::new();
    for (w, h) in [(120, 45), (180, 60), (90, 30), (100, 30)] {
        let s = compose_plain(&game, w, h);
        assert_eq!(
            arena_width(&s),
            Some((w / 3) as usize),
            "terminal width w={w}"
        );
    }
}

#[test]
fn output_has_exact_terminal_dimensions() {
    let game = Game::new();
    let (w, h) = (160u16, 48u16);
    let s = compose_plain(&game, w, h);
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(
        lines.len(),
        h as usize,
        "line count should equal the terminal height"
    );
    for line in &lines {
        assert_eq!(
            line.chars().count(),
            w as usize,
            "each line's width should equal the terminal width"
        );
    }
}

#[test]
fn renders_score_and_hp_bars() {
    let game = Game::new();
    let s = compose_plain(&game, 160, 48);
    assert!(s.contains("P1"));
    assert!(s.contains("P2"));
    assert!(s.contains(" : "), "should show the score separator");
    assert!(s.contains('['), "should show an HP bar");
    assert!(s.contains(']'));
}

#[test]
fn renders_ko_and_winner() {
    let mut game = Game::new();
    game.left.hp = 0;
    game.left.state = FighterState::Down;
    game.right_score = 1;
    game.ko_side = Some(Side::Right);
    game.round_over = true;

    let s = compose_plain(&game, 160, 48);
    assert!(s.contains("K.O."));
    assert!(s.contains("P2 WINS!"));
}

#[test]
fn small_terminal_shows_too_small_message() {
    let game = Game::new();
    let s = compose_plain(&game, 30, 9);
    assert!(s.contains("too small"));
}

#[test]
fn scales_with_terminal_size() {
    let game = Game::new();
    let small = compose_plain(&game, 100, 30);
    let big = compose_plain(&game, 240, 72);
    assert!(
        big.lines().count() > small.lines().count(),
        "a larger terminal should have more lines"
    );
    assert!(
        arena_width(&big).unwrap() > arena_width(&small).unwrap(),
        "a larger terminal should have a wider arena"
    );
}

#[test]
fn projectile_renders_as_dot() {
    let mut game = Game::new();
    game.projectiles.push(Projectile {
        owner: Side::Left,
        kind: StrikeKind::Punch,
        progress: 0.5,
        dmg: 10,
    });
    let s = compose_plain(&game, 160, 48);
    assert!(s.contains('●'), "the projectile should render as a dot");
}
