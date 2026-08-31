# rust-funny

A tiny terminal toy: two stick figures duke it out on their own in the middle of your terminal, rescaling smoothly as you stretch the window. No input required — press `q` or `Esc` to quit.

- **Language:** Rust (edition 2021)
- **Dependencies:** [`crossterm`](https://crates.io/crates/crossterm) 0.28 (terminal control), [`rand`](https://crates.io/crates/rand) 0.9 (attack timing & damage)

## Features

- **Auto-battle** — each fighter runs its own state machine (`wind up → punch/kick → recover`), entirely hands-off.
- **HP bars & score** — live health bars (green → yellow → red), a running score, `K.O.` detection, and an automatic rematch.
- **Flying projectiles** — every attack launches a dot that drifts toward the opponent, tinted to match the attacker (left = cyan, right = magenta), and deals damage on contact.
- **Resolution-independent** — the fighters are drawn as a *skeleton of joints connected by lines*, not fixed-size sprites, so the scene scales cleanly to any window size.
- **Seven states, five poses** — idle, windup, punch, kick, hit, recover, down.

## Preview

A single `cargo run -- --dump 80 24` frame: the centered arena with score and HP bars, both fighters, and a projectile (`●`) in flight.

```
┌────────────────────────┐
│      P1 0 : 0 P2       │
│P1 [█████]    [████░] P2│
│   O                 O  │
│   █            ●   ██  │
│   ██                █  │
│   ██               ██  │
└────────────────────────┘
```

## Quick start

```bash
make            # debug build, then copy the binary to ./rust-funny
make release    # optimized build (LTO + strip), also copied to the repo root
make run        # run the interactive TUI (aliases cargo run)
```

Press `q` or `Esc` to quit.

## Make targets

| Command | What it does |
| --- | --- |
| `make` / `make debug` | Debug build, then copy the binary to the repo root |
| `make release` | Optimized build (`[profile.release]`: `lto` + `strip`), copied to root |
| `make run` | Run the interactive fight (`cargo run`) |
| `make run-release` | Run the optimized binary (`cargo run --release`) |
| `make test` | Run all unit + integration tests (`cargo test`) |
| `make fmt` | Format the code (`cargo fmt`) |
| `make clean` | `cargo clean` and remove the copied `./rust-funny` |

## Preview without a TTY

In headless environments (CI, scripts, SSH), `--dump` renders a single plain-text frame to stdout:

```bash
cargo run -- --dump 80 24   # defaults to 80×24; pass any width and height
```

This is the primary way to inspect layout, scaling, and projectile behavior outside an interactive terminal.

## Tests

```bash
make test     # everything

# single tests, by name filter
cargo test --test render arena_is_centered_one_third_of_terminal   # one integration test
cargo test game::tests::take_hit_reduces_hp_and_flinches           # one unit test
```

- **Unit tests** live inline as `#[cfg(test)] mod tests` in `src/` (they may touch private items such as `Game::take_hit`).
- **Integration tests** live in `tests/` and exercise only the public API (`rust_funny::game` / `rust_funny::render`).

## Project structure

```
.
├── Cargo.toml          # package manifest, dependencies, release profile
├── Makefile            # convenience build / test / run targets
├── .gitignore          # ignores target/, the copied binary, editor/system files
├── src/
│   ├── main.rs         # binary entry: terminal setup, fixed-rate loop, --dump
│   ├── lib.rs          # library entry: exposes game / render for tests
│   ├── game.rs         # fight logic: state machine, rounds, score, projectiles
│   └── render.rs       # rendering: skeleton, poses, line drawing, bars, projectiles
└── tests/
    ├── game.rs         # game-logic integration tests
    └── render.rs       # rendering integration tests
```

## Standard GitHub files

This repository follows common GitHub community conventions — a standard set of files to govern contributions, issue reports, security handling, and automation. A quick overview below, with details after.

### At a glance

| File / directory | Purpose |
| --- | --- |
| [`LICENSE`](LICENSE) | MIT license |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution guide (setup, style, PR flow) |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Code of conduct (Contributor Covenant 2.1) |
| [`SECURITY.md`](SECURITY.md) | Vulnerability reporting and handling |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history |
| [`.editorconfig`](.editorconfig) | Cross-editor indentation / line endings / charset |
| [`.gitattributes`](.gitattributes) | LF line endings and text classification |
| `.github/workflows/ci.yml` | Continuous integration (fmt / clippy / test / build) |
| `.github/ISSUE_TEMPLATE/` | Issue templates (bug / feature) |
| `.github/PULL_REQUEST_TEMPLATE.md` | Pull request template |
| `.github/dependabot.yml` | Automated dependency updates |

### Details

#### LICENSE (MIT)

Released under the **MIT** license: you may use, modify, redistribute, and even commercialize the code, provided you keep the copyright and license notice. Include this file when distributing the code or any derivative of it.

#### CONTRIBUTING.md

**Read before contributing.** It covers environment setup, code style (`cargo fmt` + `cargo clippy -D warnings`), commit-message conventions (Conventional Commits), and the full branch → PR workflow.

#### CODE_OF_CONDUCT.md

Based on **Contributor Covenant 2.1**, it applies to every community space (issues, PRs, discussions). It defines welcomed and unacceptable behavior, plus an escalating enforcement ladder (correction → warning → temporary ban → permanent ban). Everyone participates under it by default.

#### SECURITY.md

States which versions are supported and **how to privately report a vulnerability**. ⚠️ If you find one, do *not* disclose it in a public issue — contact the maintainer privately through the channel listed there.

#### CHANGELOG.md

Follows **Keep a Changelog** and semantic versioning, recording changes under `Added` / `Changed` / `Fixed`. **If your PR changes user-visible behavior, add an entry under `[Unreleased]`.**

#### .editorconfig

Keeps indentation, line endings, and charset consistent across editors and IDEs. Rules: 4 spaces for Rust, 2 for YAML, tabs for the Makefile, LF + UTF-8 throughout. Most editors pick this up automatically via the EditorConfig plugin.

#### .gitattributes

Forces **LF** line endings for all text files and marks them as text, so Windows/Unix mixing doesn't pollute whole-file diffs.

#### .github/workflows/ci.yml

A GitHub Actions workflow that runs on every push to `main` and every pull request — in order: `cargo fmt --check` → `cargo clippy -D warnings` → full test suite → release build. **PRs must pass everything before merging.**

#### .github/ISSUE_TEMPLATE/

- `bug_report.yml` — bug-report form (symptoms, reproduction steps, environment).
- `feature_request.yml` — feature-request form (motivation, proposal, alternatives).
- `config.yml` — disables blank issues, requiring one of the two templates.

#### .github/PULL_REQUEST_TEMPLATE.md

Prefills each PR with the required fields: **what changed**, **linked issue**, **type of change**, and a checklist of the verification steps already run (fmt / clippy / test / build).

#### .github/dependabot.yml

Dependabot checks `cargo` and GitHub Actions dependencies weekly and opens an update PR when it finds a new version. CI runs the full suite before merge.

## Architecture

### lib + bin split

`src/lib.rs` exposes `pub mod game; pub mod render;` so the integration tests in `tests/` can drive the public API. `src/main.rs` is a thin binary: it owns the crossterm terminal setup, the fixed-rate loop, and the `--dump` path.

### Rendering: skeleton + line drawing (the key to smooth resizing)

Stick figures are defined as *joints in a normalized design space* (x 0..8, y 0..10, facing +x; the right-side fighter is mirrored via `x → 8 - x`). `compute_layout` derives a single scale factor from the terminal size — the arena is the centered W/3 × H/3 region (1/9 of the area) — and `draw_fighter` maps each joint to a cell, then draws bones with a Bresenham line. That's why the scene resizes smoothly. To change a fighter's appearance or pose, edit the `pose_*` functions in `render.rs`; the joints, bones, and mirror constant all live there.

### Game logic: independent state machines

Each `Fighter` is an independent state machine (`Idle → Windup → Punch/Kick → Recover`, plus `Hit`/`Down`). `Game::tick()` advances both fighters, spawns a projectile at an attack's hit frame, advances projectiles and settles contact damage, and handles KO → score increment → automatic reset. `FighterState` maps 1:1 to poses via `render::pose_for`.

### Projectiles

- An attack's hit frame no longer deals damage instantly; it spawns a `Projectile` (damage, owner, strike kind, progress).
- Position is expressed as **progress (0..1)**: 0 = the attacker's leading joint (punch = right fist, kick = right foot), 1 = the opponent's mirrored joint. The start/end geometry is derived from `pose_*` in `render.rs`, so the projectile stays aligned to the skeleton and scales with the window.
- On contact it deals damage via `take_hit`; leftover projectiles are cleared at the end of each round.

### Main loop

A fixed ~15 FPS loop (`FRAME = 66ms` in `main.rs`) with non-blocking crossterm polling for `Resize` (recompute layout) and quit keys.

## Maintainer guide

### Adding a pose or attack

1. Add a variant to `FighterState` in `src/game.rs`; for attacks, wire the state transitions and frame counts in `update_fighter`.
2. Add `fn pose_<name>() -> Pose` in `src/render.rs`, returning the 13 joints in design-space coordinates (x 0..8, y 0..10, facing +x).
3. Map the new state to the new pose in `pose_for(state)`.
4. Add tests: `every_pose_stays_in_design_bounds` in `render.rs` covers bounds automatically; add a state-transition test in `game.rs`.

### Adjusting colors

- Fighter / projectile colors: `C_LEFT` (cyan) and `C_RIGHT` (magenta) in `render.rs`; projectile color is mapped by owner via `projectile_color`.
- HP bar thresholds: `hp_color` in `render.rs` (>50 green, >25 yellow, else red).

### Tuning attack pacing & damage

- `game.rs` constants: `WINDUP_FRAMES`, `STRIKE_FRAMES`, `HIT_FRAME`, `RECOVER_FRAMES`, `ROUND_PAUSE`.
- Damage ranges: punch `8..16`, kick `14..22` (see `update_fighter`); cooldown `random_range(6..20)`.
- Projectiles: `PROJ_SPEED` (flight speed); the projectile's height and endpoints come from `pose_punch` / `pose_kick` in `render.rs`.

## Conventions

- Unit tests live inline in `src/` (they may access private items); integration tests live in `tests/` and use only the public API.
- `Cargo.lock` is committed (this is a binary application, pinned for reproducible builds).
- Build artifacts are gitignored: `/target/` and the copied `/rust-funny`.

## License & contributing

- **License:** MIT — see [LICENSE](LICENSE).
- **Contributing:** see [CONTRIBUTING.md](CONTRIBUTING.md).
- **Code of conduct:** see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- **Security:** see [SECURITY.md](SECURITY.md).
- **Changelog:** see [CHANGELOG.md](CHANGELOG.md).
- **CI:** GitHub Actions — see [.github/workflows/ci.yml](.github/workflows/ci.yml).
