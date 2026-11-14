# Craftrinth for Windows 7 & Legacy Systems

> **Status:** Coming Soon (Active Development)

Craftrinth Windows 7 Edition is a dedicated, ultra-low-overhead lightweight client designed specifically for legacy Windows environments (Windows 7 / 8 / 8.1) and constrained/budget hardware.

---

## Key Features & Innovations

### 1. Automated Hardware Inspection & OpenGL Fallback (Mesa3D)
Older graphics hardware (such as Intel HD Graphics 2000/3000, legacy GMA series, old AMD/ATI Radeon, and GeForce 6/7/8 series) often lack native drivers for OpenGL 3.2+ or 4.x required by modern Minecraft versions (1.17+).
- **Auto Hardware Scanner:** Checks system GPU capabilities, driver vendor string, and supported OpenGL feature sets on startup.
- **Automated Mesa3D Provisioning:** If hardware lacks required OpenGL capabilities or reports crashing/outdated vendor drivers, Craftrinth automatically fetches and configures **Mesa3D Software Rasterizer (`opengl32.dll`)** directly into the game instance environment.
- **Seamless Game Launch:** Allows newer Minecraft versions to boot and run on legacy GPUs that would otherwise crash with `GLFW error 65542: WGL: The driver does not appear to support OpenGL`.

### 2. Universal PrimeAuthLib Auto-Sync
- Automatically pulls the latest **PrimeAuthLib** agent releases (`primeauthlib-*.jar`) for both Windows 7 legacy launchers and modern regular systems.
- Ensures cracked/offline custom skin rendering works out of the box across all versions (1.21.x and newer) without needing third-party skin mods installed.

### 3. Native Low-Memory Architecture
- Built without Chromium/WebView2 overhead to ensure minimal RAM and CPU utilization.
- Fast startup, minimal disk footprint, and zero telemetry bloat.

---

## Directory Overview

```
windows7/
├── AGENTS.md        # Real-time multi-agent activity, task log, and memory handoff ledger
├── README.md        # Windows 7 edition specifications and roadmap
└── [core components under development]
```
