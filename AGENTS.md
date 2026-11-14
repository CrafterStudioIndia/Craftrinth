# Craftrinth Monorepo & Multi-Agent Memory Ledger

This file serves as the master instruction and persistent memory ledger for all agents across all versions and subprojects of Craftrinth (Linux, Windows modern, Windows 7 C++ Edition, Backend, Frontend).

When an agent hits a rate limit, context window truncation, or server session restart, **the incoming agent must read this file first** to immediately resume state and tasks without losing context.

---

## Architecture Overview

- **Tooling:** [Turborepo](https://turbo.build/) (`turbo.jsonc`) + [pnpm workspaces](https://pnpm.io/workspaces) (`pnpm-workspace.yaml`) + Cargo Workspaces + CMake (Windows 7 C++)
- **Frontend (Web/App):** Vue 3, Vite, Tailwind CSS v3
- **Desktop Shell (Modern):** Tauri 2 / Rust (`apps/app`)
- **Desktop Shell (Windows 7 / Legacy):** C++20 Native Win32 / Direct2D (`windows7/`)
- **Core Launcher Engine:** Rust (`packages/app-lib`)
- **Backend API:** Rust (`apps/labrinth`)
- **Indentation:** Use TAB everywhere, never spaces

### Directory Map

| Directory | Target / Scope | Technology Stack |
| --------- | -------------- | ---------------- |
| `windows7/` | Windows 7 / 8 / 8.1 low-spec systems | C++20, Win32 API, Direct2D, RAII / Memory-Safe Core |
| `apps/app/` | Modern Desktop Shell (Linux / Windows 10+ / macOS) | Tauri 2, Rust |
| `apps/app-frontend/` | Main Desktop UI | Vue 3, Vite, Pinia, Tailwind CSS |
| `packages/app-lib/` | Shared Launcher Engine & Pipeline | Rust, SQLite (`app.db`), Tokio |
| `packages/ui/` | Shared UI Component Library | `@modrinth/ui` (Vue 3) |
| `packages/assets/` | Shared Icons and Styles | `@modrinth/assets` |
| `apps/labrinth/` | Backend API Service | Rust, PostgreSQL, Clickhouse |

---

## Agent Handoff Protocol & Memory Ledger

Whenever an agent takes action or reaches a milestone:
1. Append a row to the **Agent Task & Context Ledger** below.
2. Note exact rationale, technical decisions, and outputs.
3. If an agent is preempted or ratelimited, the subsequent agent picks up directly from the **Active State & Next Actions**.

### Agent Task & Context Ledger

| Timestamp (UTC/Local) | Agent / Engine | Target / Subsystem | Task Summary | Rationale & Technical Context | Result / Commit |
| --------------------- | -------------- | ------------------ | ------------ | ----------------------------- | --------------- |
| 2026-10-27 22:05 | Antigravity | Desktop (Tauri/Vue) | Fixed AppImage compilation & duplicate account bug | Ensured fresh frontend embedding in Tauri release and player head canvas rendering | Verified working `dist/Craftrinth-x86_64.AppImage` |
| 2026-11-07 15:41 | Antigravity | Git / Repo | Created remote repository & resolved shallow graft | GitHub rejected shallow push; recreated clean root commit and pushed to `CrafterStudioIndia/Craftrinth` | Commit `96001dd` pushed to `main` |
| 2026-11-07 15:44 | Antigravity | Documentation | Comprehensive README overhaul | Documented architecture, all outbound network endpoints, and cracked/skin pipeline | Commit `0bc60b3` |
| 2026-11-14 19:37 | Antigravity | Windows 7 | Initialized Windows 7 legacy initiative | Set up roadmap for automated GPU scanner, Mesa3D fallback, and PrimeAuthLib sync | Commit `0a37634` |
| 2026-11-14 19:42 | Antigravity | Monorepo Master | Promoted AGENTS.md to master global ledger | Established universal memory persistence across all versions and agents | Commit pending |
| 2026-11-14 19:44 | Antigravity | Windows 7 (C++) | Built C++20 Memory-Enforced Win7 Engine | Implemented Win32 native hardware scanner, Mesa3D injector, and PrimeAuthLib sync | Implemented in `windows7/` |

---

## Active State & Next Actions
1. **Windows 7 Native C++ Engine (`windows7/`)**:
	- C++20 with strict RAII, smart pointer ownership (`std::unique_ptr`, `std::shared_ptr`), bounds-checked spans (`std::span`), and zero raw pointer leaks.
	- Security mitigation flags: ASLR (`/DYNAMICBASE`), DEP/NX (`/NXCOMPAT`), SafeSEH, Control Flow Guard (`/guard:cf`).
	- Integrated hardware inspection: detects OpenGL < 3.2 on legacy GPUs (Intel HD 2000/3000, GMA) and auto-deploys Mesa3D `opengl32.dll`.
	- Integrated PrimeAuthLib auto-sync engine: downloads modern runtime agents for both Windows 7 and modern editions.
