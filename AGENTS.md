# Repository Guidelines

## Project Structure & Module Organization
This repository is a Tauri 2 desktop widget with a Vite frontend. Frontend source lives in `src/`: `main.js` boots the app, `src/app/` contains controllers, state, rendering, Tauri service wrappers, and update logic, `src/components/` holds reusable UI widgets, and `src/styles/` plus `src/themes/` contain CSS and theme variants. Rust backend code lives in `src-tauri/src/`, with quota parsing under `src-tauri/src/quota/` and app integration split across commands, tray, settings, logging, and window state modules. Tauri configuration is in `src-tauri/tauri.conf.json`; release helper scripts are in `scripts/`; documentation images are in `docs/assets/`.

## Build, Test, and Development Commands
- `npm run ci:install`: install pinned Node dependencies with CI-friendly flags.
- `npm run dev`: run the Vite frontend on `127.0.0.1:1420`.
- `npm run tauri:dev`: launch the full desktop app during development.
- `npm run build`: build the frontend only.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust unit tests.
- `npm run tauri:build` or `npm run tauri:build:nsis`: create desktop release bundles.
- `npm run release:github`: build updater-enabled NSIS artifacts and generate `latest.json`.

## Coding Style & Naming Conventions
Use ES modules in JavaScript, 2-space indentation, semicolons, and camelCase names. Keep controller factories named `createX`, matching existing files such as `createQuotaController`. CSS files use kebab-case class names and custom properties. Rust should follow standard `rustfmt` formatting, snake_case modules/functions, and small focused modules under `src-tauri/src/`.

## Testing Guidelines
Rust tests are inline `#[cfg(test)]` modules near the code they validate, especially for quota normalization, settings, sessions, and logging. Add or update tests when changing parsing, persistence, or platform-sensitive behavior. There is no frontend test script currently, so manually verify UI changes with `npm run tauri:dev` and document that verification in the PR.

## Commit & Pull Request Guidelines
Recent commits use short, direct Chinese summaries and occasional merge commits; no strict Conventional Commits format is enforced. Keep commit messages concise and behavior-focused, for example `优化默认主题悬浮球边缘`. Pull requests should include a clear description, linked issue when applicable, commands run, and screenshots or screen recordings for visible UI/theme changes. Never commit generated signing keys or local secrets; updater keys belong in repository secrets.
