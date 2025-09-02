# Changelog - Version 0.3.0

## Overview

Version 0.3.0 represents a major milestone in Solarance: Beginnings, introducing the foundation of a player-driven economy, comprehensive faction systems, and robust communication infrastructure. This release transforms the game from a basic exploration experience into a living, interactive universe where players can engage in meaningful economic activities and coordinate through faction-based gameplay.

### What's New in v0.3.0:

- **Station Economy & Trading** - Functional docking, buy/sell mechanics, and dynamic pricing
- **Resource Processing Chains** - Multi-tier production from raw ores to manufactured goods
- **Faction System** - Dedicated chat channels, member management, and reputation tracking
- **Server Messaging** - Admin announcements, group messaging, and error feedback
- **Enhanced Station Modules** - Refineries, manufacturing, solar arrays, farms, and more
- **Improved User Interface** - Resizable chat, distance-based radar, redesigned cargo panel
- **Technical Upgrades** - SpacetimeDSL 0.10.0, better database schema, optimized networking
- **Visual Assets** - New station types, planetary textures, and UI improvements

## 🏭 Station Economy & Trading System

### New Features

- **Station Docking & Undocking**: Complete docking mechanics with proper validation and state management
- **Trading Port Modules**: Functional buy/sell system with dynamic pricing based on station inventory
- **Resource Processing Chains**: Multi-tier production system (raw ores → refined materials → manufactured goods)
- **Station Module Construction**: Blueprint-based construction system for expanding station capabilities
- **Dynamic Pricing**: Market prices adjust based on supply and demand at individual stations
- **Inventory Management**: Comprehensive station storage and cargo tracking systems

### Station Modules Added

- **Refinery Modules**: Process raw ores into refined materials with automated timers
- **Manufacturing Modules**: Convert refined materials into complex manufactured goods
- **Solar Array Modules**: Generate power for station operations
- **Farm Modules**: Produce food and organic materials
- **Laboratory Modules**: Research and development capabilities
- **Storage Depot**: Enhanced cargo storage capacity
- **Trading Port**: Buy/sell interface for player-station commerce
- **Residential Modules**: Population and workforce management
- **Hospital, Embassy, Observatory**: Specialized station functions

## 💬 Server Messaging & Communication

### New Features

- **Admin Messaging System**: Server administrators can send targeted announcements
- **Group Messaging**: Coordinate with multiple players through secure group channels
- **Privacy Protection**: Message visibility controls to prevent information leakage
- **Server Error Feedback**: Game actions now provide clear error messages through the chat system
- **Unread Message Indicators**: Visual feedback for new messages in chat interface
- **Targeted Player Messaging**: Direct communication between specific players

### Technical Improvements

- Integrated server messaging into existing game reducers
- Enhanced chat widget to display different message types
- Improved message persistence and synchronization

## 🏛️ Faction System & Communication

### New Features

- **Faction Chat Channels**: Dedicated communication channels for faction members
- **Comprehensive Faction Management**: Interface showing member lists, relations, and standings
- **Universal Faction Membership**: All players now belong to a faction (defaulting to "Factionless")
- **Faction Reputation System**: Track standings and relationships between different factions
- **Real-time Member Status**: Live updates on faction member activity and presence
- **Hierarchical Faction Structure**: Support for parent-child faction relationships

### Faction Infrastructure

- Faction creation and management systems
- Member invitation and management tools
- Faction-specific timers and automated processes
- Enhanced faction definitions with detailed metadata

## 🎨 Enhanced User Interface

### Improvements

- **Resizable Chat Window**: Improved message display with adjustable window size
- **Distance-based Radar Scaling**: Radar icons scale based on distance for better spatial awareness
- **Redesigned Cargo Panel**: Bottom panel layout with capacity indicators and totals
- **Improved Station Interfaces**: Better visual feedback for docking and trading actions
- **Enhanced Ship Details**: Minimum height constraints and better information display
- **Automatic Ship Selection**: Ships are automatically selected when docking

### Visual Enhancements

- New station assets: capital, large, medium, small, outpost, satellite stations
- Additional planetary assets: barren moon textures
- Cargo box visual assets
- Improved UI scaling and responsiveness

## 🔧 Technical Infrastructure Upgrades

### Database & Backend

- **SpacetimeDSL 0.10.0**: Upgraded to latest version with improved foreign key relationships
- **Enhanced Schema**: Proper referential integrity with foreign key constraints
- **Improved Subscriptions**: Better real-time multiplayer synchronization
- **Error Handling**: Graceful failure recovery and better error reporting
- **Network Optimization**: Optimized synchronization for station interactions

### Code Quality

- Comprehensive test suites for server messaging functionality
- Improved code modularity with better separation of concerns
- Enhanced utility functions for common operations
- Better documentation and code organization

## 🐛 Bug Fixes

- Fixed schema generation errors on Windows systems
- Resolved issues with ship cargo not displaying in out-of-play screen
- Fixed station bracket sizing issues
- Corrected problems with loading prior global/sector chat messages
- Fixed buying/selling reducers to properly validate docking status
- Resolved asteroid type randomization issues
- Fixed water barrels incorrectly spawning from asteroids

## 📁 New Files & Structure

### Client-side Additions

- `client/src/gameplay/gui/asset_utils.rs` - Reusable UI components
- `client/src/gameplay/gui/out_of_play_screen.rs` - Enhanced out-of-game interface
- `client/src/gameplay/render/in_sector.rs` - Sector-specific rendering
- `client/src/gameplay/render/star_system.rs` - System-wide rendering
- `client/src/gameplay/server_messages.rs` - Server message handling
- `client/src/lib.rs` - Library structure improvements

### Server-side Additions

- `server/src/types/server_messages/` - Complete server messaging module
- `server/src/types/factions/definitions.rs` - Faction data structures
- `server/src/types/factions/timers.rs` - Faction automation
- `server/src/types/factions/utility.rs` - Faction helper functions
- `server/src/types/stations/modules/` - Comprehensive station module system
- `server/src/types/npcs.rs` - NPC system foundation

### Documentation & Configuration

- `db_flow.md` - Database flow documentation
- `procyon_system_definition.json` - Star system configuration
- `.kiro/` directory - Development tooling and specifications
- Enhanced README with current feature status

## 🔄 Breaking Changes

- All players must now belong to a faction (automatic assignment to "Factionless" for existing players)
- Station interaction requires proper docking validation
- Chat system now handles multiple message types with different display formats
- Database schema changes require fresh database initialization

## 🎯 Performance Improvements

- Optimized database queries with proper indexing
- Improved network synchronization for multiplayer interactions
- Better memory management for large station inventories
- Enhanced rendering performance for complex station interfaces

## 🚀 Developer Experience

- Enhanced development tooling with Kiro integration
- Improved build scripts and task automation
- Better error messages and debugging information
- Comprehensive test coverage for new features

---

This release establishes the foundation for a truly interactive space MMO experience, where players can engage in meaningful economic activities, coordinate through faction systems, and participate in a living, breathing universe. The infrastructure is now in place for future expansions into more complex gameplay mechanics, PvP systems, and advanced economic simulation.
