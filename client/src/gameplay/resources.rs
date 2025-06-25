use std::{ collections::HashMap };
use macroquad::prelude::*;

pub struct Resources {
    pub ship_textures: HashMap<&'static str, Texture2D>,
    pub asteroid_textures: HashMap<&'static str, Texture2D>,
    pub station_textures: HashMap<&'static str, Texture2D>,
    pub jumpgate_textures: HashMap<&'static str, Texture2D>,
    pub effect_textures: HashMap<&'static str, Texture2D>,
    pub sun_textures: HashMap<&'static str, Texture2D>,
    pub planet_textures: HashMap<&'static str, Texture2D>,
}

impl Resources {
    pub async fn new() -> Result<Resources, macroquad::Error> {
        let mut resources = Resources {
            ship_textures: HashMap::new(),
            asteroid_textures: HashMap::new(),
            station_textures: HashMap::new(),
            jumpgate_textures: HashMap::new(),
            effect_textures: HashMap::new(),
            sun_textures: HashMap::new(),
            planet_textures: HashMap::new(),
        };

        // Load asset textures
        info!("Loading textures...");

        resources.sun_textures.insert("star.1", load_linear_sprite("stars/star.png").await?);
        //resources.sun_textures.insert("star.2", load_linear_sprite("stars/star02.png").await?);

        resources.planet_textures.insert(
            "planet.1",
            load_linear_sprite("planets/rockplanet.png").await?
        );
        resources.planet_textures.insert(
            "planet.2",
            load_linear_sprite("planets/GasGiant1.png").await?
        );
        resources.planet_textures.insert(
            "planet.shadow.1",
            load_linear_sprite("planets/PlanetShadows.png").await?
        );

        resources.planet_textures.insert(
            "moon.1",
            load_linear_sprite("planets/barren_moon.png").await?
        );

        resources.ship_textures.insert(
            "lc.phalanx",
            load_linear_sprite("ships/lc/phalanx.png").await?
        );
        resources.ship_textures.insert(
            "lc.column",
            load_linear_sprite("ships/lc/Column.png").await?
        );
        resources.ship_textures.insert(
            "rf.javelin",
            load_linear_sprite("ships/rf/javelin.png").await?
        );

        resources.station_textures.insert(
            "lc.station.1",
            load_linear_sprite("stations/lrak_outpost.png").await?
        );
        resources.station_textures.insert(
            "iwa.station.1",
            load_linear_sprite("stations/iwa_generic_station.PNG").await?
        );
        resources.station_textures.insert(
            "station.capital",
            load_linear_sprite("stations/station_capital.png").await?
        );
        resources.station_textures.insert(
            "station.large",
            load_linear_sprite("stations/station_large.png").await?
        );
        resources.station_textures.insert(
            "station.medium",
            load_linear_sprite("stations/station_medium.png").await?
        );
        resources.station_textures.insert(
            "station.small",
            load_linear_sprite("stations/station_small.png").await?
        );
        resources.station_textures.insert(
            "station.outpost",
            load_linear_sprite("stations/station_outpost.png").await?
        );
        resources.station_textures.insert(
            "station.satellite",
            load_linear_sprite("stations/station_satellite.png").await?
        );

        resources.jumpgate_textures.insert(
            "warpgate_north",
            load_linear_sprite("stations/warpgate_north.png").await?
        );
        resources.jumpgate_textures.insert(
            "warpgate_west",
            load_linear_sprite("stations/warpgate_west.png").await?
        );
        resources.jumpgate_textures.insert(
            "warpgate_south",
            load_linear_sprite("stations/warpgate_south.png").await?
        );
        resources.jumpgate_textures.insert(
            "warpgate_east",
            load_linear_sprite("stations/warpgate_east.png").await?
        );

        resources.effect_textures.insert(
            "bullet.1",
            load_linear_sprite("ships/bullet01.png").await?
        );
        resources.effect_textures.insert(
            "bullet.2",
            load_linear_sprite("ships/bullet02.png").await?
        );
        resources.effect_textures.insert(
            "engineflare",
            load_linear_sprite("ships/engineflare.png").await?
        );

        resources.asteroid_textures.insert(
            "asteroid.1",
            load_linear_sprite("asteroids/asteroid01.png").await?
        );
        resources.asteroid_textures.insert(
            "asteroid.2",
            load_linear_sprite("asteroids/asteroid02.png").await?
        );
        resources.asteroid_textures.insert(
            "asteroid.3",
            load_linear_sprite("asteroids/asteroid03.png").await?
        );
        resources.asteroid_textures.insert(
            "asteroid.4",
            load_linear_sprite("asteroids/asteroid04.png").await?
        );
        resources.asteroid_textures.insert(
            "asteroid.5",
            load_linear_sprite("asteroids/asteroid05.png").await?
        );
        resources.asteroid_textures.insert("crate.0", load_linear_sprite("crate.png").await?);

        info!("Building texture atlas...");
        //build_textures_atlas();

        Ok(resources)
    }
}

async fn load_linear_sprite(path: &str) -> Result<Texture2D, macroquad::Error> {
    let texture = load_texture(path).await?;
    texture.set_filter(FilterMode::Nearest);
    Ok(texture)
}
