# Windows 7 Agent Work Log & Memory Ledger

This file is the persistent memory and task ledger for all agents contributing to the Craftrinth Windows 7 Edition. If an agent hits a rate limit, context window truncation, or session restart, any subsequent agent **must read this file first** to immediately resume context without losing progress.

---

## Agent Handoff Protocol & Rules
1. **Log Every Task**: When starting or finishing any task, append an entry to the `Task & Memory History` table below with timestamps, agent name/ID, what was done, and why.
2. **State & Architectural Findings**: Record any quirks, OS limitations (e.g. Win32 API boundaries, compiler flags, DLL dependencies), and decisions.
3. **Hardware Scanner & Mesa3D Specs**:
	- Target GPUs: Intel HD 2000/3000, GMA 3100/4500, legacy ATI/Nvidia.
	- Driver detection: Query `wglCreateContext` / `glGetString(GL_VERSION)` or DXGI / WMI `Win32_VideoController`.
	- Mesa3D deployment: Fetch verified release of `opengl32.dll` (llvmpipe / D3D12 / software rasterizer) and place it in the game's working directory or pass via `PATH` isolation.
4. **PrimeAuthLib Sync Specs**:
	- Dual-target: Must provide agent jars for both legacy Windows 7 launcher and modern standard Craftrinth installations.
	- Target directory: `packages/app-lib/resources/primeauthlib/`.

---

## Architecture & Specifications

### Hardware Inspection Engine
- **Target OS**: Windows 7 SP1 (x86 & x64), Windows 8 / 8.1.
- **Detector**: Checks GPU vendor, device ID, and OpenGL version support.
- **Mesa3D Auto-Downloader**:
	- Triggers if reported OpenGL version is `< 3.2` for modern Minecraft (1.17+) or if driver initialization fails.
	- Downloads precompiled Mesa3D `opengl32.dll` build and maps it to the instance launch runtime.

### PrimeAuthLib Version Sync
- Automates fetching the latest releases of `primeauthlib` from repository releases.
- Distributes runtime JARs for both the Windows 7 edition and regular Craftrinth launcher versions.

---

## Task & Memory History

| Timestamp (UTC/Local) | Agent / Session | Action Taken | Rationale & Context | Output / Changes |
| --------------------- | --------------- | ------------ | ------------------- | ---------------- |
| 2026-11-14 19:37:30 | Antigravity | Directory initialization & documentation | User requested a dedicated `windows7/` initiative with automated OpenGL/Mesa3D hardware detection and PrimeAuthLib sync | Created `windows7/README.md` and `windows7/AGENTS.md` |
| 2026-11-14 19:37:30 | Antigravity | Memory Ledger Protocol setup | Prevent loss of context across agent rate-limits and server reboots | Established structured handoff table in `windows7/AGENTS.md` |

---

## Active Todo & Roadmap

- [ ] Implement Windows 7 hardware GPU & driver scanner (querying WMI/DXGI/OpenGL).
- [ ] Implement automated Mesa3D `opengl32.dll` download & injection module for legacy GPUs.
- [ ] Implement PrimeAuthLib auto-sync updater for both Windows 7 and regular launcher platforms.
- [ ] Implement lightweight C++ native launcher UI shell for Windows 7.
