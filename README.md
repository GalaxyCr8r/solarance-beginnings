<div align="center">

# Solarance: Beginnings

<img src="./client/assets/Solarance_Logo.png" alt="Solarance:Beginnings Logo. Text over a black planet with a pure white horizon lit up as a cresent in the bottom left of the circle." />

**A Top-Down 2D Living Universe Sci-Fi Space MMO**

[Discord](https://discord.gg/34xzCtsKxe) 🌌 [Bluesky](https://bsky.app/profile/galaxycr8r.bsky.social) 🌌 [Itch.io](https://galaxycr8r.itch.io/solarance-beginnings)

<img src="./screenshot.png" width="50%" alt="A screenshot from v0.2.0, a ship just transitted a jump gate and a asteroid is nearby." />

</div>

Solarance is a 2D top down game idea I've had for years now. Heavily inspired by
Escape Velocity: Nova, X2/X3, Freelancer, and numerous other entries in the space
adventure/building genre. This is a test project to explore Rust, Macroquad, and
[SpacetimeDB](https://spacetimedb.com/) to finally make the space MMO I've always wanted to make. You can help
either by contributing code - or just by playing this test client. Thank you for
joining me on this journey!

Goal is not to be a 2D EVE or Star Citizen, but a game where you can hopefully have fun
combat and creating stations with friends and exploring the unfolding universe.

For this initial stab, I'm focusing on real-time asteroids-like movement for combat.
However I expect this to be unsustainable in the long term, and in fact, not preferable
especially for keeping track of the MANY NPC entities that I want to be moving throughout
the universe all the time.

## Running the Game

### Prerequisites

- **Rust** (latest stable) - Install via [rustup](https://rustup.rs/)
- **SpacetimeDB CLI** - Install with `curl -sSf https://install.spacetimedb.com | bash`
- **Platform Dependencies** - For Macroquad graphics: https://github.com/not-fl3/macroquad#linux
- **Taskfile** (optional) - For convenient build commands: https://taskfile.dev/

### Quick Start

1. **Clone and setup environment:**

   ```bash
   git clone https://github.com/GalaxyCr8r/solarance-beginnings.git
   cd solarance-beginnings
   cp client/.env.template client/.env
   ```

2. **Choose your SpacetimeDB instance** in `client/.env`:

   - `https://maincloud.spacetimedb.com` - Public test instance (may be unstable)
   - blank or `localhost` - Run your own local server (recommended for development)

3. **Run the game:**
   - **With Taskfile:**
     - `task start` Run in a separate terminal (starts SpacetimeDB)
     - `task server:publish-clear` (builds server, publishes to SpacetimeDB, clears STDB database)
     - `task client:run-full` (generates client bindings, runs client)
   - **Manual:** See individual steps below

### Manual Setup

**For local development (recommended):**

1. **Start SpacetimeDB locally:**

   ```bash
   spacetimedb start
   ```

2. **Build and publish the server module:**

   ```bash
   cd server
   cargo build
   spacetimedb publish solarance-spacetime-module
   cd ..
   ```

3. **Run the client:**
   ```bash
   cd client
   cargo run --release
   ```

**For testnet (public instance):**

- Set `SPACETIMEDB_URI=testnet` in `client/.env`
- Run `cargo run --release` in the `client/` directory
- Note: Testnet may not always be available or up-to-date

### First Time Playing

- Create an account when prompted
- Choose a username (this will be your in-game identity)
- Your ship will spawn in a random sector (tbd)
- Use WASD or arrow keys to move, mouse to target objects
- Try mining asteroids and using jump gates to explore! (Most hotkeys are noted with perenstheses on the UI!)

## Current State of the Project

The project continues to evolve with **Milestone 4** introducing faction warfare, NPC autonomy, and basic combat systems building upon the economic foundation established in v0.3.0.

### Milestone 4 Key Features (Current Development)

- **Faction Warfare & Combat System**

  - Players can create new factions or join existing ones
  - Basic combat mechanics with ship-to-ship engagement
  - Faction-based conflict resolution and territorial disputes
  - Combat damage system affecting ship functionality

- **Autonomous NPC Ecosystem**

  - Faction NPCs spawn proportionally to active player count
  - NPCs engage in autonomous trading between stations
  - NPC raiding parties target rival faction assets
  - Guardian NPCs patrol and defend faction territories
  - Simple behavior trees drive NPC decision-making

- **Living Universe Simulation**

  - NPCs visible and interactive in the game world
  - Autonomous NPC task queues for complex behaviors
  - Dynamic faction influence based on NPC and player activity
  - Real-time faction warfare with NPC participation

### Version 0.3.0 Foundation Features (Completed)

- **Station Economy & Trading System**

  - Functional station docking and undocking mechanics
  - Trading port modules with buy/sell functionality
  - Dynamic pricing system based on station inventory
  - Resource processing and production chains
  - Station module construction and management

- **Server Messaging & Communication**
  The following are only features of the Server message channel - in general chat there's no way to do these.. yet.

  - Admin messaging system for server announcements
  - Targeted player messaging with privacy protection
  - Group messaging capabilities for coordinated play
  - Server error feedback integrated into game actions
  - Unread message indicators in chat interface

- **Station Production Infrastructure**

  - Refinery modules that process raw ores into refined materials
  - Automated production timers and resource conversion
  - Station module blueprints and construction system
  - Multi-tier resource processing (raw → refined → manufactured)
  - Station inventory management and storage systems

- **Faction System & Communication**

  - Dedicated faction chat channels for coordinated gameplay
  - Comprehensive faction management interface with member lists and relations
  - All players now belong to a faction (defaulting to "Factionless" for new players)
  - Faction reputation system showing standings between different groups
  - Real-time faction member status and activity tracking

- **Enhanced User Interface**

  - Resizable chat window with improved message display
  - Distance-based radar icon scaling for better spatial awareness
  - Redesigned cargo panel with capacity indicators
  - Improved station interaction interfaces
  - Better visual feedback for docking and trading actions

- **Technical Infrastructure Upgrades**

  - Upgraded to SpacetimeDSL 0.10.0 with improved foreign key relationships
  - Enhanced database schema with proper referential integrity
  - Improved subscription system for real-time multiplayer updates
  - Better error handling and graceful failure recovery
  - Optimized network synchronization for station interactions

### Version 0.2.0 Foundation Features

- **Core Gameplay Loop**: Mining, cargo management, jump gate travel, and exploration
- **Player Systems**: Account creation, ship spawning, WASD movement with physics
- **UI Framework**: Comprehensive GUI system with minimap, galaxy map, and ship details
- **Multiplayer Infrastructure**: Real-time synchronization and player identity system
- **Cross-Platform Support**: macOS compatibility and improved asset loading

### Planned Core Features

- **Player-led Factions (PvP)**

  - Not just a group of players, Factions have numerous NPCs serving them
  - New players can choose to spawn as part of a faction if the faction allows it
  - Eventually I want Factions to be able to research new technologies that other factions don't have

- **Ship Building and Upgrades (PvE)**

  - Ship customization with different equipment types
  - Component-based ship damage system (planned)

- **Economy and Trading (PvE)**
  - Resource gathering from asteroids
  - Station-based trading system (planned)
  - Dynamic economy with supply and demand (planned)

### Contribution

The goals are set up fairly granularly in the form of [Issue Milestones](https://github.com/GalaxyCr8r/solarance-beginnings/milestones)

However almost NONE of the issues have descriptions. Please contact me on the SpacetimeDB discord or just comment on an issue that you'll like me to flesh out so it can be completed!

## License

All code is GPL 3.0

All art assets I withhold license to use. That may change when/if I get new artwork.
