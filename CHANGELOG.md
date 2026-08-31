# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Flying projectiles: after an attack, a dot tinted to match the attacker drifts toward the opponent and reduces their HP on contact.

## [0.1.0] - 2026-08-15

### Added

- Initial release: two stick figures auto-battle in the centered 1/9 of the terminal, rescaling with the window.
- HP bars, punch/kick animations, win/lose detection, and a running score.
- Unit and integration tests, plus `--dump` frame preview without a TTY.
