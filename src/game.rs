//! Auto-battle logic: each fighter independently winds up -> punches/kicks -> recovers,
//! an attack's hit frame spawns a projectile that deals damage on contact; HP hitting zero triggers a KO, increments the score, and auto-starts the next round.

use rand::rngs::ThreadRng;
use rand::Rng;

pub const MAX_HP: i32 = 100;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FighterState {
    Idle,    // standing by
    Windup,  // winding up
    Punch,   // punching (attacking)
    Kick,    // kicking (attacking)
    Hit,     // taking a hit (staggering back)
    Recover, // recovering
    Down,    // knocked down
}

pub struct Fighter {
    pub hp: i32,
    pub state: FighterState,
    timer: u32,          // frames remaining in the current state
    cooldown: u32,       // cooldown frames until the next attack
    strike_elapsed: u32, // frames elapsed in the current attack (locates the hit frame)
    hit_done: bool,      // whether this attack has already dealt its damage
    pending_kick: bool,  // whether the windup resolves into a kick (vs. a punch)
}

impl Fighter {
    fn new() -> Self {
        Fighter {
            hp: MAX_HP,
            state: FighterState::Idle,
            timer: 0,
            cooldown: 10,
            strike_elapsed: 0,
            hit_done: false,
            pending_kick: false,
        }
    }
}

/// Attack kind — punch or kick — determines the projectile's height and start/end joints (geometry in render.rs's pose_*).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StrikeKind {
    Punch,
    Kick,
}

/// A flying projectile that drifts in a straight line from the attacker's leading joint
/// (punch = right fist, kick = right foot) toward the opponent's near side, dealing damage on contact.
/// Position is expressed as "progress" (0 = attacker's joint, 1 = opponent's near side), derived from pose_* in render.rs.
pub struct Projectile {
    pub owner: Side,      // the attacker
    pub kind: StrikeKind, // punch / kick
    pub progress: f32,    // 0..1; 0 = attacker's joint, 1 = opponent's near side
    pub dmg: i32,         // damage dealt on contact
}

const WINDUP_FRAMES: u32 = 3;
const STRIKE_FRAMES: u32 = 5;
const HIT_FRAME: u32 = 2;
const RECOVER_FRAMES: u32 = 2;
const HIT_FRAMES: u32 = 4;
const ROUND_PAUSE: u32 = 30;

// Projectiles: speed is measured in progress/frame (~8 frames to cross the arena). Start/end points and height come from render.rs's pose_*.
const PROJ_SPEED: f32 = 0.12;

pub struct Game {
    pub left: Fighter,
    pub right: Fighter,
    pub projectiles: Vec<Projectile>,
    pub left_score: u32,
    pub right_score: u32,
    pub round_over: bool,
    pub ko_side: Option<Side>,
    round_timer: u32,
    rng: ThreadRng,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            left: Fighter::new(),
            right: Fighter::new(),
            projectiles: Vec::new(),
            left_score: 0,
            right_score: 0,
            round_over: false,
            ko_side: None,
            round_timer: 0,
            rng: rand::rng(),
        }
    }

    /// Advances one frame: updates both state machines, advances projectiles, settles contact damage, and checks for a KO.
    pub fn tick(&mut self) {
        if self.round_over {
            if self.round_timer > 0 {
                self.round_timer -= 1;
            }
            if self.round_timer == 0 {
                self.reset_round();
            }
            return;
        }

        if let Some(p) = Self::update_fighter(&mut self.left, Side::Left, &mut self.rng) {
            self.projectiles.push(p);
        }
        if let Some(p) = Self::update_fighter(&mut self.right, Side::Right, &mut self.rng) {
            self.projectiles.push(p);
        }

        self.update_projectiles();
        self.check_ko();
    }

    fn update_fighter(f: &mut Fighter, side: Side, rng: &mut ThreadRng) -> Option<Projectile> {
        let mut result = None;

        if f.cooldown > 0 {
            f.cooldown -= 1;
        }
        if f.timer > 0 {
            f.timer -= 1;
        }

        match f.state {
            FighterState::Idle => {
                if f.cooldown == 0 {
                    f.pending_kick = rng.random_bool(0.4);
                    f.hit_done = false;
                    f.strike_elapsed = 0;
                    f.state = FighterState::Windup;
                    f.timer = WINDUP_FRAMES;
                }
            }
            FighterState::Windup => {
                if f.timer == 0 {
                    f.state = if f.pending_kick {
                        FighterState::Kick
                    } else {
                        FighterState::Punch
                    };
                    f.timer = STRIKE_FRAMES;
                }
            }
            FighterState::Punch | FighterState::Kick => {
                f.strike_elapsed += 1;
                if !f.hit_done && f.strike_elapsed == HIT_FRAME {
                    f.hit_done = true;
                    let (dmg, kind) = if f.state == FighterState::Kick {
                        (rng.random_range(14..22), StrikeKind::Kick)
                    } else {
                        (rng.random_range(8..16), StrikeKind::Punch)
                    };
                    result = Some(Projectile {
                        owner: side,
                        kind,
                        progress: 0.0,
                        dmg,
                    });
                }
                if f.timer == 0 {
                    f.state = FighterState::Recover;
                    f.timer = RECOVER_FRAMES;
                }
            }
            FighterState::Hit => {
                if f.timer == 0 {
                    f.state = FighterState::Idle;
                    f.cooldown = rng.random_range(6..20);
                }
            }
            FighterState::Recover => {
                if f.timer == 0 {
                    f.state = FighterState::Idle;
                    f.cooldown = rng.random_range(6..20);
                }
            }
            FighterState::Down => {}
        }

        result
    }

    fn update_projectiles(&mut self) {
        let mut i = 0;
        while i < self.projectiles.len() {
            self.projectiles[i].progress += PROJ_SPEED;
            if self.projectiles[i].progress >= 1.0 {
                let p = self.projectiles.remove(i);
                match p.owner {
                    Side::Left => Self::take_hit(&mut self.right, p.dmg),
                    Side::Right => Self::take_hit(&mut self.left, p.dmg),
                }
            } else {
                i += 1;
            }
        }
    }

    fn take_hit(f: &mut Fighter, dmg: i32) {
        if f.state == FighterState::Down || f.hp <= 0 {
            return;
        }
        f.hp = (f.hp - dmg).max(0);
        f.hit_done = true;
        f.pending_kick = false;
        if f.hp <= 0 {
            f.state = FighterState::Down;
        } else {
            f.state = FighterState::Hit;
            f.timer = HIT_FRAMES;
        }
    }

    fn check_ko(&mut self) {
        if self.left.hp <= 0 && !self.round_over {
            self.left.state = FighterState::Down;
            self.right_score += 1;
            self.ko_side = Some(Side::Right);
            self.round_over = true;
            self.round_timer = ROUND_PAUSE;
        } else if self.right.hp <= 0 && !self.round_over {
            self.right.state = FighterState::Down;
            self.left_score += 1;
            self.ko_side = Some(Side::Left);
            self.round_over = true;
            self.round_timer = ROUND_PAUSE;
        }
    }

    fn reset_round(&mut self) {
        self.left = Fighter::new();
        self.right = Fighter::new();
        self.projectiles.clear();
        self.round_over = false;
        self.ko_side = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Fighter state machine (tests may call private methods for determinism) ----

    #[test]
    fn fighter_starts_at_full_hp() {
        let f = Fighter::new();
        assert_eq!(f.hp, MAX_HP);
        assert_eq!(f.state, FighterState::Idle);
    }

    #[test]
    fn take_hit_reduces_hp_and_flinches() {
        let mut f = Fighter::new();
        Game::take_hit(&mut f, 30);
        assert_eq!(f.hp, MAX_HP - 30);
        assert_eq!(f.state, FighterState::Hit);
    }

    #[test]
    fn take_hit_kills_when_hp_drops_to_zero() {
        let mut f = Fighter::new();
        Game::take_hit(&mut f, MAX_HP + 10);
        assert_eq!(f.hp, 0);
        assert_eq!(f.state, FighterState::Down);
    }

    #[test]
    fn take_hit_ignores_down_fighter() {
        let mut f = Fighter::new();
        f.hp = 10;
        f.state = FighterState::Down;
        Game::take_hit(&mut f, 5);
        assert_eq!(f.hp, 10, "a downed fighter should not take more damage");
    }

    #[test]
    fn idle_with_zero_cooldown_starts_windup() {
        let mut f = Fighter::new();
        f.cooldown = 0;
        let mut rng = rand::rng();
        let hit = Game::update_fighter(&mut f, Side::Left, &mut rng);
        assert_eq!(f.state, FighterState::Windup);
        assert!(hit.is_none(), "no projectile should spawn during windup");
    }

    #[test]
    fn strike_spawns_projectile_at_hit_frame() {
        let mut f = Fighter::new();
        f.cooldown = 0;
        let mut rng = rand::rng();

        // Enter windup.
        Game::update_fighter(&mut f, Side::Left, &mut rng);
        assert_eq!(f.state, FighterState::Windup);

        // Run a full windup + punch cycle; the hit frame must spawn a projectile.
        let mut spawned = None;
        for _ in 0..(WINDUP_FRAMES + STRIKE_FRAMES) {
            if let Some(p) = Game::update_fighter(&mut f, Side::Left, &mut rng) {
                spawned = Some(p);
            }
        }
        let p = spawned.expect("an attack should spawn a projectile at its hit frame");
        assert_eq!(p.owner, Side::Left);
        assert_eq!(
            p.progress, 0.0,
            "the projectile should start at the attacker's joint"
        );
        assert!(matches!(p.kind, StrikeKind::Punch | StrikeKind::Kick));
        assert!(p.dmg > 0, "the projectile should carry positive damage");
    }

    #[test]
    fn projectile_touches_opponent_and_reduces_hp() {
        let mut game = Game::new();
        let before = game.right.hp;
        game.projectiles.push(Projectile {
            owner: Side::Left,
            kind: StrikeKind::Punch,
            progress: 0.99,
            dmg: 10,
        });

        game.tick();

        assert!(
            game.right.hp < before,
            "the right side's HP should drop after the projectile lands"
        );
        assert!(
            game.projectiles.is_empty(),
            "the projectile should be removed on contact"
        );
    }

    // ---- Rounds and scoring ----

    #[test]
    fn check_ko_awards_score_exactly_once() {
        let mut game = Game::new();
        game.left.hp = 0;
        game.check_ko();
        assert!(game.round_over);
        assert_eq!(game.right_score, 1);
        assert_eq!(game.ko_side, Some(Side::Right));

        game.check_ko();
        assert_eq!(game.right_score, 1, "the same KO should not score twice");
    }

    #[test]
    fn reset_round_restores_state() {
        let mut game = Game::new();
        game.left.hp = 0;
        game.check_ko();
        game.round_timer = 1; // so the next frame resets immediately
        game.tick();
        assert!(!game.round_over);
        assert_eq!(game.left.hp, MAX_HP);
        assert_eq!(game.right.hp, MAX_HP);
        assert_eq!(game.ko_side, None);
    }

    #[test]
    fn long_run_stays_in_bounds_and_scores() {
        let mut game = Game::new();
        for _ in 0..50_000 {
            game.tick();
        }
        assert!(
            game.left_score + game.right_score > 0,
            "some rounds should have been won"
        );
        assert!((0..=MAX_HP).contains(&game.left.hp));
        assert!((0..=MAX_HP).contains(&game.right.hp));
    }
}
