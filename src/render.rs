//! Rendering: draws the fight into a character grid (with colors), then writes it to the terminal.
//!
//! The stick figures use a "skeleton + line drawing" approach: joints live in a design space
//! (x 0..8, y 0..10, facing +x) that is scaled and rounded by a terminal-derived factor, so the scene resizes smoothly with the window.

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, Print, SetForegroundColor};

use crate::game::{FighterState, Game, Side, StrikeKind, MAX_HP};

type Pt = (f32, f32);

// Joint indices
const HEAD: usize = 0;
const NECK: usize = 1;
const LSHO: usize = 2;
const RSHO: usize = 3;
const LELB: usize = 4;
const RELB: usize = 5;
const LHAND: usize = 6;
const RHAND: usize = 7;
const HIP: usize = 8;
const LKNEE: usize = 9;
const RKNEE: usize = 10;
const LFOOT: usize = 11;
const RFOOT: usize = 12;

// Bone connections (drawn as limbs and torso)
const BONES: [(usize, usize); 11] = [
    (NECK, LSHO),
    (LSHO, LELB),
    (LELB, LHAND),
    (NECK, RSHO),
    (RSHO, RELB),
    (RELB, RHAND),
    (NECK, HIP),
    (HIP, LKNEE),
    (LKNEE, LFOOT),
    (HIP, RKNEE),
    (RKNEE, RFOOT),
];

type Pose = [Pt; 13];

// Design space: body midline at x=4, mirrored about x=8
const MIRROR_X: f32 = 8.0;
const DESIGN_W: f32 = 8.0; // figure bounding-box width (units)
const DESIGN_H: f32 = 10.0; // figure bounding-box height (units)

fn pose_idle() -> Pose {
    [
        (4.0, 1.4), // HEAD
        (4.0, 2.7), // NECK
        (3.5, 3.0), // LSHO
        (4.5, 3.0), // RSHO
        (3.0, 4.3), // LELB
        (5.0, 4.3), // RELB
        (2.5, 5.7), // LHAND
        (5.5, 5.7), // RHAND
        (4.0, 5.3), // HIP
        (3.5, 7.3), // LKNEE
        (4.5, 7.3), // RKNEE
        (3.1, 9.4), // LFOOT
        (4.9, 9.4), // RFOOT
    ]
}

fn pose_windup() -> Pose {
    let mut p = pose_idle();
    p[HEAD] = (3.9, 1.4);
    p[NECK] = (3.9, 2.7);
    p[RSHO] = (4.4, 3.0);
    p[RELB] = (4.4, 4.2);
    p[RHAND] = (3.9, 4.9);
    p
}

fn pose_punch() -> Pose {
    let mut p = pose_idle();
    p[HEAD] = (4.1, 1.4);
    p[NECK] = (4.1, 2.7);
    p[RSHO] = (4.6, 2.9);
    p[RELB] = (6.0, 2.9);
    p[RHAND] = (7.4, 2.9);
    p
}

fn pose_kick() -> Pose {
    let mut p = pose_idle();
    p[HEAD] = (3.7, 1.5);
    p[NECK] = (3.8, 2.8);
    p[HIP] = (3.8, 5.3);
    p[LSHO] = (3.4, 3.1);
    p[LELB] = (2.9, 4.2);
    p[LHAND] = (2.4, 4.6);
    p[RSHO] = (4.4, 3.1);
    p[RELB] = (5.0, 4.0);
    p[RHAND] = (5.4, 4.6);
    p[RKNEE] = (5.4, 5.4);
    p[RFOOT] = (7.2, 5.2);
    p
}

fn pose_hit() -> Pose {
    let mut p = pose_idle();
    p[HEAD] = (3.4, 1.6);
    p[NECK] = (3.5, 2.9);
    p[HIP] = (3.5, 5.4);
    p[LSHO] = (3.1, 3.2);
    p[LELB] = (2.5, 4.0);
    p[LHAND] = (2.1, 4.6);
    p[RSHO] = (4.0, 3.2);
    p[RELB] = (3.6, 4.2);
    p[RHAND] = (3.2, 4.8);
    p[RKNEE] = (4.3, 7.4);
    p[RFOOT] = (4.7, 9.4);
    p
}

fn pose_for(state: FighterState) -> Pose {
    match state {
        FighterState::Punch => pose_punch(),
        FighterState::Kick => pose_kick(),
        FighterState::Hit | FighterState::Down => pose_hit(),
        FighterState::Windup => pose_windup(),
        _ => pose_idle(),
    }
}

fn mirror(mut p: Pose) -> Pose {
    for j in p.iter_mut() {
        j.0 = MIRROR_X - j.0;
    }
    p
}

// ---- Colors ----
const C_LEFT: Color = Color::Cyan;
const C_RIGHT: Color = Color::Magenta;
const C_BORDER: Color = Color::DarkGrey;
const C_TEXT: Color = Color::White;
const C_KO: Color = Color::Yellow;

fn hp_color(hp: i32) -> Color {
    if hp > 50 {
        Color::Green
    } else if hp > 25 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn projectile_color(owner: Side) -> Color {
    match owner {
        Side::Left => C_LEFT,
        Side::Right => C_RIGHT,
    }
}

// ---- Character grid ----
#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    color: Color,
}

struct Grid {
    w: usize,
    h: usize,
    cells: Vec<Cell>,
}

impl Grid {
    fn new(w: usize, h: usize) -> Self {
        Grid {
            w,
            h,
            cells: vec![
                Cell {
                    ch: ' ',
                    color: Color::Reset
                };
                w * h
            ],
        }
    }

    fn set(&mut self, x: i32, y: i32, ch: char, color: Color) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        self.cells[y as usize * self.w + x as usize] = Cell { ch, color };
    }

    fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.w + x]
    }

    fn to_plain_string(&self) -> String {
        let mut s = String::with_capacity((self.w + 1) * self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                s.push(self.cells[y * self.w + x].ch);
            }
            if y + 1 < self.h {
                s.push('\n');
            }
        }
        s
    }
}

fn draw_line(g: &mut Grid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        g.set(x, y, '█', color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_text(g: &mut Grid, x: i32, y: i32, s: &str, color: Color) {
    for (i, c) in s.chars().enumerate() {
        g.set(x + i as i32, y, c, color);
    }
}

// ---- Layout ----
struct Layout {
    ax: i32, // arena top-left x
    ay: i32,
    aw: usize, // arena width (columns)
    ah: usize, // arena height (rows)
    scale: f32,
}

fn compute_layout(w: usize, h: usize) -> Layout {
    let aw = (w / 3).max(2);
    let ah = (h / 3).max(2);
    let ax = ((w as i32 - aw as i32) / 2).max(0);
    let ay = ((h as i32 - ah as i32) / 2).max(0);

    let scale_w = (aw.saturating_sub(2)) as f32 / 18.0;
    // The figure spans ~8.6 design units vertically (head center 1.4 to feet 9.4),
    // so leave 5 rows of headroom for the score line, HP bars, and top/bottom borders.
    let scale_h = (ah.saturating_sub(5)) as f32 / 9.0;
    let scale = scale_w.min(scale_h).max(0.01);

    Layout {
        ax,
        ay,
        aw,
        ah,
        scale,
    }
}

fn draw_border(g: &mut Grid, l: &Layout) {
    for x in 0..l.aw as i32 {
        g.set(l.ax + x, l.ay, '─', C_BORDER);
        g.set(l.ax + x, l.ay + l.ah as i32 - 1, '─', C_BORDER);
    }
    for y in 1..l.ah as i32 - 1 {
        g.set(l.ax, l.ay + y, '│', C_BORDER);
        g.set(l.ax + l.aw as i32 - 1, l.ay + y, '│', C_BORDER);
    }
    g.set(l.ax, l.ay, '┌', C_BORDER);
    g.set(l.ax + l.aw as i32 - 1, l.ay, '┐', C_BORDER);
    g.set(l.ax, l.ay + l.ah as i32 - 1, '└', C_BORDER);
    g.set(
        l.ax + l.aw as i32 - 1,
        l.ay + l.ah as i32 - 1,
        '┘',
        C_BORDER,
    );
}

fn draw_score(g: &mut Grid, l: &Layout, game: &Game) {
    let left = game.left_score.to_string();
    let right = game.right_score.to_string();
    let s1 = "P1 ";
    let s3 = " : ";
    let s5 = " P2";
    let total = s1.len() + left.len() + s3.len() + right.len() + s5.len();
    let cx = l.ax + ((l.aw as i32 - total as i32) / 2).max(0);
    let y = l.ay + 1;

    draw_text(g, cx, y, s1, C_LEFT);
    draw_text(g, cx + s1.len() as i32, y, &left, C_TEXT);
    draw_text(g, cx + (s1.len() + left.len()) as i32, y, s3, C_TEXT);
    draw_text(
        g,
        cx + (s1.len() + left.len() + s3.len()) as i32,
        y,
        &right,
        C_TEXT,
    );
    draw_text(
        g,
        cx + (s1.len() + left.len() + s3.len() + right.len()) as i32,
        y,
        s5,
        C_RIGHT,
    );
}

fn draw_hp_bar(g: &mut Grid, x: i32, y: i32, width: usize, hp: i32) {
    if width < 4 {
        return;
    }
    let inner = width - 2;
    let filled = ((hp as f32 / MAX_HP as f32) * inner as f32).round() as usize;
    g.set(x, y, '[', C_BORDER);
    for i in 0..inner {
        if i < filled {
            g.set(x + 1 + i as i32, y, '█', hp_color(hp));
        } else {
            g.set(x + 1 + i as i32, y, '░', C_BORDER);
        }
    }
    g.set(x + width as i32 - 1, y, ']', C_BORDER);
}

fn draw_fighter(
    g: &mut Grid,
    state: FighterState,
    ox: f32,
    oy: f32,
    scale: f32,
    color: Color,
    mirrored: bool,
) {
    let mut pose = pose_for(state);
    if mirrored {
        pose = mirror(pose);
    }
    let joints: Vec<(i32, i32)> = pose
        .iter()
        .map(|(x, y)| {
            (
                (ox + x * scale).round() as i32,
                (oy + y * scale).round() as i32,
            )
        })
        .collect();

    for &(a, b) in BONES.iter() {
        draw_line(g, joints[a].0, joints[a].1, joints[b].0, joints[b].1, color);
    }

    let (hx, hy) = joints[HEAD];
    let hr = (0.45 * scale).round() as i32;
    if hr <= 1 {
        // Small/medium size: a single-cell round head that stays proportional and clear of the HP bars above.
        g.set(hx, hy, 'O', color);
    } else {
        for dy in -hr..=hr {
            for dx in -hr..=hr {
                if dx * dx + dy * dy <= hr * hr {
                    g.set(hx + dx, hy + dy, '█', color);
                }
            }
        }
    }
}

fn draw_projectiles(g: &mut Grid, l: &Layout, game: &Game, oy: f32, left_ox: f32, right_ox: f32) {
    for p in &game.projectiles {
        // The attacker's joint (punch = right fist, kick = right foot) in design space; start/end geometry matches pose_*.
        let (jx, jy) = match p.kind {
            StrikeKind::Punch => {
                let pose = pose_punch();
                (pose[RHAND].0, pose[RHAND].1)
            }
            StrikeKind::Kick => {
                let pose = pose_kick();
                (pose[RFOOT].0, pose[RFOOT].1)
            }
        };
        // Attacker's joint -> opponent's near side (the same joint, mirrored), interpolated by progress.
        let (sx, tx) = match p.owner {
            Side::Left => (left_ox + jx * l.scale, right_ox + (MIRROR_X - jx) * l.scale),
            Side::Right => (right_ox + (MIRROR_X - jx) * l.scale, left_ox + jx * l.scale),
        };
        let gx = (sx + (tx - sx) * p.progress).round() as i32;
        let gy = (oy + jy * l.scale).round() as i32;
        g.set(gx, gy, '●', projectile_color(p.owner));
    }
}

fn draw_ko(g: &mut Grid, l: &Layout, game: &Game) {
    let winner = match game.ko_side {
        Some(Side::Left) => "P1 WINS!",
        Some(Side::Right) => "P2 WINS!",
        None => return,
    };
    let ko = "K.O.";
    let y = l.ay + l.ah as i32 / 2 - 1;
    let kx = l.ax + ((l.aw as i32 - ko.len() as i32) / 2).max(0);
    draw_text(g, kx, y, ko, C_KO);
    let wx = l.ax + ((l.aw as i32 - winner.len() as i32) / 2).max(0);
    draw_text(g, wx, y + 1, winner, C_KO);
}

fn build_grid(game: &Game, w: u16, h: u16) -> Grid {
    let (w, h) = (w as usize, h as usize);
    let mut g = Grid::new(w, h);
    let l = compute_layout(w, h);

    if l.scale < 0.3 || l.aw < 12 || l.ah < 6 {
        let msg = "terminal too small, enlarge the window";
        let cx = (w as i32 - msg.len() as i32) / 2;
        draw_text(&mut g, cx, (h as i32 / 2).max(0), msg, C_TEXT);
        return g;
    }

    draw_border(&mut g, &l);

    // Fighters stand with feet on the inner bottom edge; the left faces right, the right is mirrored to face left.
    let bottom = l.ay + l.ah as i32 - 2;
    let oy = bottom as f32 - DESIGN_H * l.scale;
    let left_ox = (l.ax + 3) as f32;
    let right_ox = (l.ax + l.aw as i32 - 3) as f32 - DESIGN_W * l.scale;

    draw_fighter(&mut g, game.left.state, left_ox, oy, l.scale, C_LEFT, false);
    draw_fighter(
        &mut g,
        game.right.state,
        right_ox,
        oy,
        l.scale,
        C_RIGHT,
        true,
    );

    if !game.round_over {
        draw_projectiles(&mut g, &l, game, oy, left_ox, right_ox);
    }

    // Draw HP bars and score last so they always sit above the fighters (even if a head pokes up).
    let bar_y = l.ay + 2;
    let bar_w = ((l.aw as i32 - 12) / 2).max(4) as usize;
    draw_text(&mut g, l.ax + 1, bar_y, "P1", C_LEFT);
    draw_hp_bar(&mut g, l.ax + 4, bar_y, bar_w, game.left.hp);
    draw_hp_bar(
        &mut g,
        l.ax + l.aw as i32 - 4 - bar_w as i32,
        bar_y,
        bar_w,
        game.right.hp,
    );
    draw_text(&mut g, l.ax + l.aw as i32 - 3, bar_y, "P2", C_RIGHT);
    draw_score(&mut g, &l, game);

    if game.round_over {
        draw_ko(&mut g, &l, game);
    }

    g
}

/// Writes the frame to the terminal (raw mode + alternate screen, overwriting line by line to avoid flicker).
pub fn draw(out: &mut impl Write, game: &Game, w: u16, h: u16) -> io::Result<()> {
    let grid = build_grid(game, w, h);
    let (w, h) = (w as usize, h as usize);

    queue!(out, MoveTo(0, 0))?;
    for y in 0..h {
        queue!(out, MoveTo(0, y as u16))?;
        let mut x = 0;
        while x < w {
            let cell = grid.get(x, y);
            let mut run = String::new();
            while x < w && grid.get(x, y).color == cell.color {
                run.push(grid.get(x, y).ch);
                x += 1;
            }
            queue!(out, SetForegroundColor(cell.color), Print(run))?;
        }
    }
    out.flush()
}

/// A plain-text, color-free version for `--dump` and debugging.
pub fn compose_plain(game: &Game, w: u16, h: u16) -> String {
    build_grid(game, w, h).to_plain_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{FighterState, Game, Projectile, Side, StrikeKind};

    // ---- Pure-function unit tests ----

    #[test]
    fn mirror_flips_x_around_center() {
        let p = pose_idle();
        let m = mirror(p);
        for (a, b) in p.iter().zip(m.iter()) {
            assert!(
                (a.0 + b.0 - MIRROR_X).abs() < 1e-6,
                "x should mirror about {MIRROR_X}"
            );
            assert_eq!(a.1, b.1, "y should stay unchanged");
        }
    }

    #[test]
    fn every_pose_stays_in_design_bounds() {
        for pose in [
            pose_idle(),
            pose_windup(),
            pose_punch(),
            pose_kick(),
            pose_hit(),
        ] {
            for &(x, y) in pose.iter() {
                assert!((0.0..=DESIGN_W).contains(&x), "x {x} exceeds design width");
                assert!((0.0..=DESIGN_H).contains(&y), "y {y} exceeds design height");
            }
        }
    }

    #[test]
    fn layout_centers_one_third_arena() {
        let l = compute_layout(120, 45);
        assert_eq!(l.aw, 40, "width should be 120/3");
        assert_eq!(l.ah, 15, "height should be 45/3");
        assert_eq!(l.ax, 40, "centered horizontally (120-40)/2");
        assert_eq!(l.ay, 15, "centered vertically (45-15)/2");
    }

    #[test]
    fn hp_color_thresholds() {
        assert_eq!(hp_color(80), Color::Green);
        assert_eq!(hp_color(40), Color::Yellow);
        assert_eq!(hp_color(10), Color::Red);
    }

    #[test]
    fn projectile_color_matches_owner() {
        assert_eq!(projectile_color(Side::Left), C_LEFT);
        assert_eq!(projectile_color(Side::Right), C_RIGHT);
    }

    #[test]
    fn projectile_anchors_to_fist_and_opponent() {
        let (w, h) = (160u16, 48u16);
        let l = compute_layout(w as usize, h as usize);
        let bottom = l.ay + l.ah as i32 - 2;
        let oy = bottom as f32 - DESIGN_H * l.scale;
        let left_ox = (l.ax + 3) as f32;
        let right_ox = (l.ax + l.aw as i32 - 3) as f32 - DESIGN_W * l.scale;

        let pose = pose_punch();
        let (jx, jy) = (pose[RHAND].0, pose[RHAND].1);

        let mut game = Game::new();
        game.projectiles.push(Projectile {
            owner: Side::Left,
            kind: StrikeKind::Punch,
            progress: 0.0,
            dmg: 10,
        });

        // Progress 0: overlaps the attacker's right fist.
        let mut g = Grid::new(w as usize, h as usize);
        draw_projectiles(&mut g, &l, &game, oy, left_ox, right_ox);
        let fist_x = (left_ox + jx * l.scale).round() as i32;
        let fist_y = (oy + jy * l.scale).round() as i32;
        assert_eq!(
            g.get(fist_x as usize, fist_y as usize).ch,
            '●',
            "the projectile should start at the attacker's fist"
        );

        // Progress 1: overlaps the opponent's near side (the mirrored right fist).
        game.projectiles[0].progress = 1.0;
        let mut g2 = Grid::new(w as usize, h as usize);
        draw_projectiles(&mut g2, &l, &game, oy, left_ox, right_ox);
        let near_x = (right_ox + (MIRROR_X - jx) * l.scale).round() as i32;
        assert_eq!(
            g2.get(near_x as usize, fist_y as usize).ch,
            '●',
            "the projectile should end at the opponent's near side"
        );
    }

    // ---- Whole-frame rendering unit tests ----

    #[test]
    fn ko_overlay_renders() {
        let mut game = Game::new();
        game.left.hp = 0;
        game.left.state = FighterState::Down;
        game.right_score = 1;
        game.ko_side = Some(Side::Right);
        game.round_over = true;

        let s = compose_plain(&game, 160, 48);
        assert!(s.contains("K.O."), "should show K.O.");
        assert!(s.contains("P2 WINS!"), "should show P2 as the winner");
    }

    #[test]
    fn arena_is_centered() {
        let game = Game::new();
        let s = compose_plain(&game, 160, 48);
        // The top border should appear on row 16 (48/3=16), centered horizontally.
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 48);
        let top_border = lines[16];
        assert!(
            top_border.contains('┌'),
            "row 16 should be the arena's top border: {top_border}"
        );
    }
}
