# Solarance: Beginnings

Solarance is a 2D top down game idea I've had for years now. Heavily inspired by 
Escape Velocity: Nova, X2/X3, Freelancer, and numerous other entries in the space
adventure/building genre.

For this initial stab, I'm focusing on real-time asteroids-like movement for combat.
However I expect this to be unsustainable in the long term, and in fact, not preferable
especially for keeping track of the MANY NPC entities that I want to be moving throughout
the universe all the time.

This is a test project to explore Rust, Macroquad, and SpacetimeDB to finally make the
space MMO I've always wanted to make.

## Current State of the Project

This is a very early prototype. Assuming SpacetimeDB will be able to support it, the following
are the core features:

 - Player-led Factions (PvP)
   - Not just a group of players, Factions have numerous NPCs serving them.
   - New players can choose to spawn as part of a faction if the faction allows it.
   - Eventually I want Factions to be able to research new technologies that other factions don't have.
 - Ship Building and Upgrades (PvE)
 - Mining and Resource Management (PvE)

The goals are set up fairly granularly in the form of [Issue Milestones](https://github.com/GalaxyCr8r/solarance-beginnings/milestones)