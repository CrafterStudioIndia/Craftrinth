# Craftrinth

Craftrinth is a high-performance, community-focused Minecraft launcher built on the Modrinth/Theseus platform. It combines Modrinth's complete mod and modpack ecosystem with native offline account support, custom skin rendering pipelines, Discord Rich Presence integration, and a modern desktop interface.

---

## Architecture & Codebase Structure

Craftrinth is structured as a Turborepo monorepo using `pnpm` workspaces for frontend packages and Cargo workspaces for Rust components:

```
Craftrinth/
├── apps/
│	├── app/                 # Tauri 2 desktop shell (Rust)
│	│	├── src/api/         # Tauri IPC commands & native bindings
│	│	└── tauri.conf.json  # Application permissions & window settings
│	├── app-frontend/        # Main launcher UI (Vue 3, Vite, Tailwind CSS)
│	│	├── src/app/shell/   # Sidebar, title bar, navigation layout
│	│	├── src/components/  # Offline login, modals, Discord RPC widgets
│	│	├── src/features/    # Account manager, instance settings, onboarding
│	│	└── src/pages/       # Browse, Skins, Instance management, Library
│	├── frontend/            # Web platform interface (Nuxt 3)
│	├── labrinth/            # Backend API service (Rust / Actix)
│	└── docs/                # Developer documentation (Astro)
├── packages/
│	├── app-lib/             # Core launcher engine (Rust)
│	│	├── resources/       # Bundled runtime JARs (PrimeAuthLib)
│	│	├── src/api/         # Instance runner, Minecraft auth, process monitor
│	│	├── src/launcher/    # Java execution, args parser, custom skin loader
│	│	└── src/state/       # SQLite database (app.db), settings, Discord RPC
│	├── ui/                  # Shared UI components library (@modrinth/ui)
│	├── assets/              # Icons, stylesheets, branding assets (@modrinth/assets)
│	└── api-client/          # TypeScript client for Modrinth API
└── buildappimage.sh         # Linux AppImage compilation & packaging script
```

---

## What We Changed

Craftrinth extends the upstream launcher with key modifications for offline convenience and visual customization:

### 1. Offline & Cracked Account Support
- **Modal Login Flow**: Added `OfflineSignInModal.vue` allowing instant username entry without requiring Microsoft authentication.
- **Credential Storage**: Accounts are registered directly into the local SQLite database (`app.db`) as offline credentials with tokens flagged for offline validation.
- **Bypass Verification**: Launch arguments adapt when launching offline accounts, setting username and offline UUID tokens while maintaining compatibility with online and LAN servers.

### 2. Dual-Layer Skin & Head Pipeline
- **Local Skin Storage**: Skins are stored locally in `~/.craftrinth/launcher/user/skin/{username}.png`.
- **Bundled PrimeAuthLib**: Packaged `primeauthlib` runtime agents inside `packages/app-lib/resources/primeauthlib/` targeting Minecraft 1.21.x and modern versions. The JVM args injector automatically loads the agent (`-javaagent:...`) so skins render in-game on multiplayer and singleplayer worlds without third-party mods.
- **In-Memory Head Avatar Generator**: Implemented canvas-based 8x8 slice-and-scale in `player-head.ts` with base and outer hat layer blending. Head avatars are created directly as memory object URLs, bypassing webview storage bottlenecks.
- **Skin Studio & Fetcher**: `Skins.vue` supports local PNG uploads, 3D interactive previewing, and fetching skins from any valid username.

### 3. Discord Rich Presence (RPC)
- **Game State & Timer**: Real-time presence reporting current launcher activity ("Playing <Instance>" or "Idling..."), elapsed playtime, and launcher status.
- **Player Head Integration**: In `custom_skin.rs`, the player's 8x8 head texture is cropped, scaled to 64x64, and provisioned with a secure HTTPS URL stored in `{username}_head.url`. This URL is passed to Discord's RPC `small_image` asset so your actual skin head displays directly on Discord.

### 4. UI & Theme Revamp
- **Sidebar Layout**: Integrated a sleek profile header at the top of the sidebar displaying the active player head, username, quick instance switcher, and installations manager.
- **Adaptive Brand & Icons**: Craftrinth identity with pixel-aligned instance icons and dark-mode theme adaptations.

### 5. Standalone AppImage Build Pipeline
- `buildappimage.sh` builds the frontend with Vite, compiles the release binary with embedded assets via Cargo, and bundles everything into a portable AppImage without requiring system FUSE setup.

---

## Network Connections & External Traffic

Craftrinth communicates with external endpoints only when necessary for game operations, content downloads, and account management:

| Destination / Endpoint | Protocol | Purpose | Trigger / Condition |
| ---------------------- | -------- | ------- | ------------------- |
| `api.modrinth.com` | HTTPS | Fetch mod metadata, modpacks, loaders, versions, changelogs, search | Browsing or updating instances, installing mods |
| `cdn.modrinth.com` | HTTPS | Download mod jars, resource packs, world files | Installing content or modpack assets |
| `piston-meta.mojang.com` / `launchermeta.mojang.com` | HTTPS | Retrieve official Minecraft version manifests and release indices | Initial installation or game version check |
| `libraries.minecraft.net` | HTTPS | Download vanilla game client JARs, natives, and library files | Installing or updating game versions |
| `resources.download.minecraft.net` | HTTPS | Vanilla game assets (sounds, textures, language maps) | Asset index verification during launch |
| `meta.fabricmc.net` / `maven.neoforged.net` / `maven.minecraftforge.net` / `maven.quiltmc.org` | HTTPS | Loader metadata, installers, and dependency libraries | Installing Fabric, NeoForge, Forge, or Quilt |
| `login.microsoftonline.com` / `user.auth.xboxlive.com` | HTTPS | Microsoft OAuth and Xbox token authentication | **Only** when signing in with an official Microsoft account |
| `mc-heads.net` | HTTPS | Fallback player head previews | Online accounts without a local custom skin |
| `catbox.moe/user/api.php` | HTTPS | Temporary public hosting for player head PNG | When setting custom skin Discord RPC avatar |
| `127.0.0.1` (Discord IPC Pipe) | Local IPC | Discord Rich Presence updates | While Discord desktop client is running |

> [!NOTE]
> Offline accounts do **not** transmit passwords or authentication requests to Microsoft or Mojang servers. Game launch runs locally with custom skin injection handled via local memory/agents.

---

## Development & Building

### Prerequisites
- **Node.js**: v20 or later
- **pnpm**: v9 or later
- **Rust**: 1.80+ (`rustup toolchain install stable`)
- **System dependencies (Linux)**: `webkit2gtk-4.1`, `openssl`, `libappindicator-gtk3`, `libasound2-dev`

### Running in Development
```bash
# 1. Install dependencies
pnpm install

# 2. Run the desktop application in dev mode
pnpm app:dev
```

### Packaging the Release AppImage
```bash
chmod +x buildappimage.sh
./buildappimage.sh
```
The output executable will be created at:
```bash
dist/Craftrinth-x86_64.AppImage
```

---

## License

Craftrinth is distributed under the GNU General Public License v3 (or upstream component licenses where applicable). See `COPYING.md` for specific package licenses and attribution notices.
