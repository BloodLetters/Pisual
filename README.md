<div align="center">

<img src="assets/Pisual.png" alt="Pisual Logo" width="180" />

## Pisual
**lightweight hologram plugin for Pumpkin MC.**


</div>

## About Pisual

**Pisual** is a hologram management plugin built with Rust for the [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft server. It allows server administrators to easily create, configure, and manage dynamic floating text holograms in the world.

### Features
- **Multi-line Holograms**: Add, update, and remove individual lines on the fly.
- **Data Persistence**: Automatically load and save holograms to disk.
- **Billboard Orientations**: Support for fixed, vertical, and center-aligned billboard displays.


## Roadmap / TODO

- [x] **Adding IPC**: Provide inter-process communication / plugin API for external services and integrations.
- [x] **Entity Icon Support**: Showing entity, item, and block displays attached to holograms.
- [x] **Hologram Animations**: Cycle lines, color fades, and floating movement effects.
- [x] **Modular Storage Architecture**: Directory-based per-hologram storage (`holograms/<id>/data.json`) with fault tolerance and legacy migration.
- [ ] **Internal Placeholders & Auto-Refresh**: Support dynamic placeholder variables (e.g., `{player}`, `{online}`, `{max_players}`, `{ping}`, `{tps}`) with configurable refresh intervals.
- [ ] **Interactive Click Actions**: Execute player/console commands, send chat messages, or play sounds when a player clicks/interacts with a hologram.
- [ ] **Per-Player Personalization & Conditional Visibility**: Restrict hologram visibility by permission, distance, or player conditions, with per-player placeholder evaluation.
- [ ] **Hologram Pages & Slideshows**: Multi-page holograms with automated rotation timers and interactive next/previous page click navigation.
- [ ] **Rich Color Formatting & Gradients**: Full hex/RGB color code support (`&#RRGGBB`), multi-color gradients, and rainbow text animations.
- [ ] **Native Display Interpolation**: Leverage Minecraft display entity interpolation for buttery-smooth rotation, bobbing physics, and scale transitions.
- [ ] **Ambient Particle Effects**: Attach customizable particle effects (e.g., rings, swirls, sparkles, halos) around holograms and visual items.
- [ ] **Hologram Templates & Presets**: Pre-configured templates for leaderboards, welcome boards, server stats, and shop showcases.
- [ ] **In-Game Interactive Alignment Tool**: Fine-tune hologram heights, line spacings, and item offsets interactively with commands or tools.

## Documentation

📖 **Official IPC Documentation & Interactive Playground**: [https://bloodletters.github.io/Pisual/](https://bloodletters.github.io/Pisual/)

Explore the full Inter-Plugin Communication (IPC) specification, generate JSON payloads live, and inspect Rust integration examples directly in your browser.


## Commands
| Command | Description |
| :--- | :--- |
| `/pisual create <id>` | Create a new hologram at your position |
| `/pisual delete <id>` | Remove an existing hologram |
| `/pisual list` | List all existing holograms |
| `/pisual lines <id> <add\|set\|remove>` | Manage hologram lines |
| `/pisual visual <id> <entity\|item\|block\|offset\|scale\|faceplayer\|clear>` | Attach, adjust offset/scale, face player, or clear visual display |
| `/pisual teleport <id> [here]` | Teleport to a hologram or bring it to you |
| `/pisual save` / `/pisual reload` | Save or reload holograms from storage |
| `/pisual help` | Display in-game help menu |


## Building

To build the plugin targeting WebAssembly for Pumpkin:

```bash
cargo build --target wasm32-wasip2 --release
```

Copy the compiled `.wasm` file from `target/wasm32-wasip2/release/` into your Pumpkin server's `plugins/` folder.
