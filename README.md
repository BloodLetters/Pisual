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
- [x] **Internal Placeholders & Auto-Refresh**: Support dynamic placeholder variables (e.g., `{player}`, `{online}`, `{max_players}`, `{ping}`, `{tps}`) with configurable refresh intervals.
- [x] **Interactive Click Actions**: Execute player/console commands, send chat messages, or play sounds when a player clicks/interacts with a hologram.
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
| `/pisual action <id> <list\|clear\|remove\|add\|size>` | Configure interactive click actions and hitbox size |
| `/pisual element <id> <list\|add\|remove\|offset\|scale\|size\|action>` | Manage multi-visual elements and per-element click actions |
| `/pisual interval <id> [ticks]` | Configure or view auto-refresh interval |
| `/pisual visual <id> <entity\|item\|block\|offset\|scale\|faceplayer\|clear>` | Attach, adjust offset/scale, face player, or clear visual display |
| `/pisual teleport <id> [here]` | Teleport to a hologram or bring it to you |
| `/pisual save` / `/pisual reload` | Save or reload holograms from storage |
| `/pisual help` | Display in-game help menu |


## Dynamic Placeholders

Pisual provides built-in dynamic internal placeholders for hologram lines with automatic background refresh. The refresh interval is configurable per hologram (defaulting to 20 ticks / 1.0 second for holograms containing placeholders).

### Available Placeholders

#### Player Placeholders
Evaluated for the nearest player (or target player when available):

| Placeholder | Description |
| :--- | :--- |
| `{player_name}` / `{player}` | Name of the player |
| `{ping}` | Latency of the player in milliseconds |
| `{health}` | Current player health (e.g., `20.0`) |
| `{max_health}` | Maximum player health (e.g., `20.0`) |
| `{food}` / `{food_level}` | Player food / hunger level (`0`-`20`) |
| `{saturation}` | Player food saturation level |
| `{level}` / `{exp_level}` | Player experience level |
| `{exp}` / `{points}` | Player total experience points |
| `{gamemode}` | Game mode (`Survival`, `Creative`, `Adventure`, `Spectator`) |
| `{item_in_hand}` | Item currently held in main hand |
| `{player_x}`, `{player_y}`, `{player_z}` | Player coordinates formatted to 2 decimals |
| `{player_uuid}` / `{uuid}` | Player unique UUID |

#### Server Placeholders
Evaluated from global server state:

| Placeholder | Description |
| :--- | :--- |
| `{online_players}` / `{online}` | Number of currently connected players |
| `{max_players}` / `{max}` | Maximum player capacity configured on server |
| `{tps}` | Current server ticks per second (ideal `20.00`) |
| `{mspt}` | Average milliseconds per tick |
| `{server_motd}` / `{motd}` | Server message of the day |
| `{server_difficulty}` / `{difficulty}` | Server difficulty (`Peaceful`, `Easy`, `Normal`, `Hard`) |
| `{server_version}` / `{version}` | Pumpkin server version |
| `{memory_used}` | Used memory in MB |
| `{memory_total}` / `{memory_max}` | Total allocated memory in MB |
| `{cpu_count}` | Available CPU logical core count |
| `{total_holograms}` / `{holograms}` | Total number of registered holograms |

#### World Placeholders
Evaluated from the world hosting the hologram:

| Placeholder | Description |
| :--- | :--- |
| `{world}` / `{world_name}` | Name of the world |
| `{world_time}` / `{time}` | World time of day in ticks |
| `{world_age}` | Total world age in ticks |
| `{mc_time}` | Formatted 24-hour Minecraft time (e.g., `06:00`, `14:30`) |
| `{mc_day}` | Total in-game days elapsed |
| `{world_players}` | Number of online players currently in this world |
| `{is_raining}` | Whether it is currently raining or snowing |
| `{is_thundering}` | Whether a thunderstorm is active |

#### Hologram Placeholders
Evaluated from the hologram's own properties:

| Placeholder | Description |
| :--- | :--- |
| `{holo_id}` / `{hologram_id}` | Hologram ID |
| `{x}`, `{y}`, `{z}` | Hologram world coordinates |
| `{line_count}` | Number of lines in the hologram |
| `{billboard}` | Hologram billboard orientation mode |
| `{view_range}` | Entity view distance range |


## Interactive Click Actions

Pisual holograms can be made interactive by attaching click actions. When actions are configured, an interaction hitbox entity is automatically spawned and synchronized with the hologram.

### Action Types
- **`player_command`**: Dispatches a command executed as the clicking player (e.g. `spawn`, `warp shop`).
- **`console_command`**: Dispatches a command executed by server console (e.g. `give {player} diamond 1`).
- **`message`**: Sends a formatted chat message directly to the player (supports color codes like `&a`, `&l`).
- **`sound`**: Plays a sound effect to the player (format: `sound_name [volume] [pitch]`, e.g. `entity.experience_orb.pickup 1.0 1.0`).

All actions support dynamic placeholders (such as `{player}`, `{online}`, `{holo_id}`) which are resolved at the moment of interaction.

### Action Commands
- `/pisual action <id> list` - List all configured click actions on a hologram.
- `/pisual action <id> clear` - Remove all click actions and despawn the interaction hitbox.
- `/pisual action <id> remove <index>` - Remove a specific action by its index.
- `/pisual action <id> add <any|right|left> <player_command|console_command|message|sound> <value>` - Add a new click action.
- `/pisual action <id> size [width] [height]` - View or adjust the interaction hitbox dimensions (default: 1.0 x 1.0).


## Multi-Visual Elements (Interactive 3D Components)

Pisual allows attaching multiple independent visual elements (items, blocks, or mob entities) to a single hologram. Each element has its own:
- 3D offset (`offset_x`, `offset_y`, `offset_z`) relative to the hologram base position.
- Scale and rotation.
- **Dedicated interactive click actions** with an individual interaction hitbox at the element's position.

This enables building interactive 3D in-game UI menus, confirm/cancel dialogue buttons (e.g. red block on left, green block on right), and shopping showcases within a single hologram!

### Element Commands
- `/pisual element <id> list` - List all visual elements attached to a hologram.
- `/pisual element <id> add <elem_id> <item|block|entity> <name> [x] [y] [z]` - Add or update a visual element.
- `/pisual element <id> remove <elem_id>` - Remove a visual element and despawn its entities.
- `/pisual element <id> offset <elem_id> <x> <y> <z>` - Adjust element offset coordinates.
- `/pisual element <id> scale <elem_id> <scale> [y] [z]` - Adjust element display scale.
- `/pisual element <id> size <elem_id> [width] [height]` - Adjust interaction hitbox dimensions for this element.
- `/pisual element <id> action <elem_id> list` - List click actions for this element.
- `/pisual element <id> action <elem_id> add <any|right|left> <action_type> <value>` - Add click action to this element.
- `/pisual element <id> action <elem_id> remove <index>` - Remove an action from this element.
- `/pisual element <id> action <elem_id> clear` - Clear all click actions for this element.


## Building

To build the plugin targeting WebAssembly for Pumpkin:

```bash
cargo build --target wasm32-wasip2 --release
```

Copy the compiled `.wasm` file from `target/wasm32-wasip2/release/` into your Pumpkin server's `plugins/` folder.
