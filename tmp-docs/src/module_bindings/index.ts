// HAND-WRITTEN SUBSET OF THE MODULE BINDINGS — NOT CLI-GENERATED.
//
// This file mirrors the exact structure `spacetime generate --lang typescript`
// emits for SpacetimeDB 2.x, but only declares the tables/reducers this
// website needs. Column names, order, and types MUST match the server structs
// in `server/src/tables/*.rs` — BSATN deserialization is positional.
//
// If the server schema changes (or once a 2.6-compatible CLI is available),
// replace this file with real generated bindings:
//
//   spacetime generate --lang typescript \
//     --out-dir tmp-docs/src/module_bindings --project-path server
//
// Schema snapshot: server/src/tables/{players,factions,sectors,star_system,
// jumpgates,messages}.rs as of module version 0.6.0.

/* eslint-disable */
import {
  DbConnectionBuilder as __DbConnectionBuilder,
  DbConnectionImpl as __DbConnectionImpl,
  SubscriptionBuilderImpl as __SubscriptionBuilderImpl,
  convertToAccessorMap as __convertToAccessorMap,
  makeQueryBuilder as __makeQueryBuilder,
  procedures as __procedures,
  reducerSchema as __reducerSchema,
  reducers as __reducers,
  schema as __schema,
  t as __t,
  table as __table,
  type DbConnectionConfig as __DbConnectionConfig,
  type ErrorContextInterface as __ErrorContextInterface,
  type EventContextInterface as __EventContextInterface,
  type Infer as __Infer,
  type QueryBuilder as __QueryBuilder,
  type ReducerEventContextInterface as __ReducerEventContextInterface,
  type RemoteModule as __RemoteModule,
  type SubscriptionEventContextInterface as __SubscriptionEventContextInterface,
  type SubscriptionHandleImpl as __SubscriptionHandleImpl,
} from 'spacetimedb';

// ---------------------------------------------------------------------------
// Shared value types (server: solarance-shared / spacetimedsl wrappers)
// ---------------------------------------------------------------------------

/** `solarance_shared::Vec2` — a plain `{ x, y }` product. */
const vec2 = () => __t.object('Vec2', { x: __t.f32(), y: __t.f32() });

/**
 * `spacetimedsl` ID wrapper structs are single-field products: `{ value: T }`.
 * Only fields whose *declared* server type is the wrapper (e.g.
 * `Player.faction_id: FactionId`) use this; `#[use_wrapper]` columns stay raw.
 */
const factionIdWrapper = () => __t.object('FactionId', { value: __t.u32() });

// ---------------------------------------------------------------------------
// Row types — field order MUST match the Rust struct declaration order.
// ---------------------------------------------------------------------------

/** server/src/tables/players.rs — `Player` (public) */
const playerRow = __t.row({
  id: __t.identity().primaryKey(),
  username: __t.string(),
  credits: __t.u64(),
  loggedIn: __t.bool().name('logged_in'),
  factionId: factionIdWrapper().name('faction_id'),
  lastLogin: __t.option(__t.timestamp()).name('last_login'),
  createdAt: __t.timestamp().name('created_at'),
  modifiedAt: __t.timestamp().name('modified_at'),
});

/** server/src/tables/factions.rs — `Faction` (public) */
const factionRow = __t.row({
  id: __t.u32().primaryKey(),
  parentId: __t.option(factionIdWrapper()).name('parent_id'),
  name: __t.string(),
  shortName: __t.string().name('short_name'),
  description: __t.string(),
  tier: __t.enum('FactionTier', [
    'Alliance',
    'Galactic',
    'Conglomerate',
    'Guild',
    'Corporation',
    'Squad',
  ]),
  joinable: __t.bool(),
  capitalStationId: __t.option(__t.u64()).name('capital_station_id'),
});

/** server/src/tables/sectors.rs — `Sector` (public) */
const sectorRow = __t.row({
  id: __t.u64().primaryKey(),
  systemId: __t.u32().name('system_id'),
  name: __t.string(),
  description: __t.option(__t.string()),
  controllingFactionId: __t.u32().name('controlling_faction_id'),
  securityLevel: __t.u8().name('security_level'),
  sunlight: __t.f32(),
  anomalous: __t.f32(),
  nebula: __t.f32(),
  rareOre: __t.f32().name('rare_ore'),
  x: __t.f32(),
  y: __t.f32(),
  backgroundGfxKey: __t.option(__t.string()).name('background_gfx_key'),
});

/** server/src/tables/star_system.rs — `StarSystem` (public) */
const starSystemRow = __t.row({
  id: __t.u32().primaryKey(),
  name: __t.string(),
  mapCoordinates: vec2().name('map_coordinates'),
  spectral: __t.enum('SpectralKind', ['O', 'B', 'A', 'F', 'G', 'K', 'M']),
  luminosity: __t.u8(),
  controllingFactionId: __t.u32().name('controlling_faction_id'),
});

/** server/src/tables/star_system.rs — `StarSystemObject` (public) */
const starSystemObjectRow = __t.row({
  id: __t.u32().primaryKey(),
  systemId: __t.u32().name('system_id'),
  kind: __t.enum('StarSystemObjectKind', [
    'Star',
    'Planet',
    'Moon',
    'AsteroidBelt',
    'NebulaBelt',
  ]),
  orbitAu: __t.f32().name('orbit_au'),
  rotationOrWidthKm: __t.f32().name('rotation_or_width_km'),
  gfxKey: __t.option(__t.string()).name('gfx_key'),
});

/** server/src/tables/jumpgates.rs — `JumpGate` (public) */
const jumpGateRow = __t.row({
  id: __t.u64().primaryKey(),
  currentSectorId: __t.u64().name('current_sector_id'),
  targetSectorId: __t.u64().name('target_sector_id'),
  targetGateArrivalPos: vec2().name('target_gate_arrival_pos'),
  targetGateArrivalRotation: __t.f32().name('target_gate_arrival_rotation'),
  gfxKey: __t.option(__t.string()).name('gfx_key'),
  isActive: __t.bool().name('is_active'),
  position: vec2(),
  rotation: __t.f32(),
});

/** server/src/tables/messages.rs — `ServerChannelMessage` (public MOTD) */
const serverChannelMessageRow = __t.row({
  id: __t.u64().primaryKey(),
  body: __t.string(),
  createdAt: __t.timestamp().name('created_at'),
});

/**
 * server/src/tables/messages.rs — `GalaxyChannelMessage`, exposed to clients
 * through the `my_galaxy_chat` view (gated server-side to registered players).
 */
const galaxyChannelMessageRow = __t.row({
  id: __t.u64().primaryKey(),
  galaxyId: __t.u32().name('galaxy_id'),
  sender: __t.enum('MessageSender', {
    Player: __t.identity(),
    System: __t.unit(),
  }),
  body: __t.string(),
  createdAt: __t.timestamp().name('created_at'),
});

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

const tablesSchema = __schema({
  player: __table({ name: 'player', public: true }, playerRow),
  faction: __table({ name: 'faction', public: true }, factionRow),
  sector: __table({ name: 'sector', public: true }, sectorRow),
  starSystem: __table({ name: 'star_system', public: true }, starSystemRow),
  starSystemObject: __table(
    { name: 'star_system_object', public: true },
    starSystemObjectRow
  ),
  jumpGate: __table({ name: 'jump_gate', public: true }, jumpGateRow),
  serverChannelMessage: __table(
    { name: 'server_channel_message', public: true },
    serverChannelMessageRow
  ),
  // Server-side `#[view] my_galaxy_chat` — subscribed like a table.
  myGalaxyChat: __table(
    { name: 'my_galaxy_chat', public: true },
    galaxyChannelMessageRow
  ),
});

// ---------------------------------------------------------------------------
// Reducers
// ---------------------------------------------------------------------------

const reducersSchema = __reducers(
  // server/src/logic/players/registration.rs
  __reducerSchema('register_playername', {
    identity: __t.identity(),
    username: __t.string(),
    factionId: __t.u32(),
  }),
  // server/src/logic/chat_messages.rs
  __reducerSchema('send_galaxy_chat', {
    message: __t.string(),
  })
);

const proceduresSchema = __procedures();

// ---------------------------------------------------------------------------
// Remote module assembly (same shape as CLI-generated output)
// ---------------------------------------------------------------------------

const REMOTE_MODULE = {
  versionInfo: {
    cliVersion: '2.6.1' as const,
  },
  tables: tablesSchema.schemaType.tables,
  reducers: reducersSchema.reducersType.reducers,
  ...proceduresSchema,
} satisfies __RemoteModule<
  typeof tablesSchema.schemaType,
  typeof reducersSchema.reducersType,
  typeof proceduresSchema
>;

/** Query-builder handles for subscriptions: `tables.player`, `tables.sector`… */
export const tables: __QueryBuilder<typeof tablesSchema.schemaType> =
  __makeQueryBuilder(tablesSchema.schemaType);

/** Reducer handles: `reducers.registerPlayername`, `reducers.sendGalaxyChat`. */
export const reducers = __convertToAccessorMap(
  reducersSchema.reducersType.reducers
);

// Row types for app code.
export type Player = __Infer<typeof playerRow>;
export type Faction = __Infer<typeof factionRow>;
export type Sector = __Infer<typeof sectorRow>;
export type StarSystem = __Infer<typeof starSystemRow>;
export type StarSystemObject = __Infer<typeof starSystemObjectRow>;
export type JumpGate = __Infer<typeof jumpGateRow>;
export type ServerChannelMessage = __Infer<typeof serverChannelMessageRow>;
export type GalaxyChannelMessage = __Infer<typeof galaxyChannelMessageRow>;

export type EventContext = __EventContextInterface<typeof REMOTE_MODULE>;
export type ReducerEventContext = __ReducerEventContextInterface<
  typeof REMOTE_MODULE
>;
export type SubscriptionEventContext = __SubscriptionEventContextInterface<
  typeof REMOTE_MODULE
>;
export type ErrorContext = __ErrorContextInterface<typeof REMOTE_MODULE>;
export type SubscriptionHandle = __SubscriptionHandleImpl<typeof REMOTE_MODULE>;

export class SubscriptionBuilder extends __SubscriptionBuilderImpl<
  typeof REMOTE_MODULE
> {}

export class DbConnectionBuilder extends __DbConnectionBuilder<DbConnection> {}

export class DbConnection extends __DbConnectionImpl<typeof REMOTE_MODULE> {
  static builder = (): DbConnectionBuilder => {
    return new DbConnectionBuilder(
      REMOTE_MODULE,
      (config: __DbConnectionConfig<typeof REMOTE_MODULE>) =>
        new DbConnection(config)
    );
  };

  override subscriptionBuilder = (): SubscriptionBuilder => {
    return new SubscriptionBuilder(this);
  };
}
