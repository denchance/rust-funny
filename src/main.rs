//! A command-line stick-figure brawl: two stick figures fight automatically
//! in the centered 1/9 of the terminal, rescaling with the window. Press `q` or `Esc` to quit.
//!
//! Debugging: `cargo run -- --dump [W H]` renders one plain-text frame to stdout (no TTY needed).

use std::io::{self, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use rust_funny::game::Game;
use rust_funny::render;

const FRAME: Duration = Duration::from_millis(66); // ~15 FPS

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--dump") {
        dump(&args);
        return;
    }

    if let Err(e) = run_tui() {
        eprintln!("error: {e}");
    }
}

/// Renders one plain-text frame for inspecting the layout without a TTY.
fn dump(args: &[String]) {
    let mut nums = Vec::new();
    for a in args {
        if let Ok(n) = a.parse::<u16>() {
            nums.push(n);
        }
    }
    let w = nums.first().copied().unwrap_or(80);
    let h = nums.get(1).copied().unwrap_or(24);

    let mut game = Game::new();
    // Advance a few frames so the scene shows HP loss and punch/kick poses.
    for _ in 0..40 {
        game.tick();
    }
    print!("{}", render::compose_plain(&game, w, h));
}

fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen, Hide)?;

    let result = event_loop(&mut out);

    execute!(out, LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;
    result
}

fn event_loop(out: &mut impl Write) -> io::Result<()> {
    let (mut w, mut h) = crossterm::terminal::size().unwrap_or((80, 24));
    if w == 0 {
        w = 80;
    }
    if h == 0 {
        h = 24;
    }

    let mut game = Game::new();

    loop {
        // Handle all pending events without blocking (resize, quit).
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Resize(nw, nh) => {
                    w = nw;
                    h = nh;
                }
                Event::Key(k) => {
                    if matches!(k.code, KeyCode::Char('q') | KeyCode::Esc) {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        game.tick();
        render::draw(out, &game, w, h)?;

        std::thread::sleep(FRAME);
    }
}
