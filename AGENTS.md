# Repository Guidelines

## Project Structure & Module Organization

XNote is a Vue 3 + TypeScript frontend packaged with a Tauri/Rust backend. Frontend code lives in `src/`: `components/` for Vue UI, `views/` for route screens, `stores/` for Pinia state, `utils/` for helpers, `types/` for shared types, and `styles/` for SCSS. Routing is in `src/router/index.ts`; the app entrypoint is `src/main.ts`. Backend code is under `src-tauri/src/`, organized by modules such as `notes/`, `tags/`, `storage/`, `sync/`, and `config/`. App icons are in `src-tauri/icons/`; tracked docs assets are under `docs/`.

## Build, Test, and Development Commands

- `npm install`: install frontend and Tauri CLI dependencies.
- `npm run dev`: start the Vite dev server on port `1421`.
- `npm run tauri dev`: run the full desktop app in development mode.
- `npm run build`: build the frontend with Vite.
- `npm run tauri build`: create the production Tauri bundle.
- `./build-release.sh`: run the release build script.
- `npm test`: placeholder frontend test command; currently exits successfully without running tests.
- `cd src-tauri && cargo test`: run Rust backend tests.
- `npm run version`: sync the npm package version into `src-tauri/tauri.conf.json`.

## Coding Style & Naming Conventions

Use TypeScript strict mode and the `@/*` alias for imports from `src`. Keep Vue components in PascalCase filenames, Pinia stores and utilities in lower camel-case or concise domain names, and Rust modules in snake_case. Follow existing two-space indentation in Vue/TypeScript and standard `cargo fmt` output for Rust. Prefer SCSS variables from `src/styles/variables.scss`.

## Testing Guidelines

Place Rust unit tests beside the relevant backend module or in files such as `src-tauri/src/config/tests.rs`. For frontend work, add a real test runner before relying on `*.test.ts`; `src/utils/logger.test.ts` is currently an executable example. Run `cargo test` for backend changes and `npm run build` for TypeScript/Vue changes.

## Commit & Pull Request Guidelines

Git history uses short imperative commits and some conventional prefixes, for example `fix: save note error...` and `ci: temporarily disable linux build`. Prefer `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, or `chore:`. Pull requests should describe the change, list verification commands, link issues, and include screenshots or recordings for UI changes.

## Security & Configuration Tips

Do not commit local note data, generated bundles, secrets, or machine-specific paths. Treat `config.json` and Tauri filesystem permissions carefully; they affect stored content and desktop app access.
