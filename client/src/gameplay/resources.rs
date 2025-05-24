use std::{collections::HashMap};
use macroquad::prelude::*;

pub struct Resources {
    pub ship_textures: HashMap<&'static str, Texture2D>,
    pub bullet_texture: Texture2D,
    pub station_texture: Texture2D,
    pub sun_texture: Texture2D,
}

impl Resources {
    pub async fn new() -> Result<Resources, FileError>
    {
        let mut ship_textures: HashMap<&'static str, Texture2D> = HashMap::new();

        // Load asset textures
        info!("Loading textures...");
        let sun_texture: Texture2D =
            load_texture("stars/star.png").await?;
        sun_texture.set_filter(FilterMode::Nearest);
        let ship_texture: Texture2D =
            load_texture("ships/lc/phalanx.png").await?;
        ship_texture.set_filter(FilterMode::Nearest);
        let ship2_texture: Texture2D =
            load_texture("ships/rf/javelin.png").await?;
        ship2_texture.set_filter(FilterMode::Nearest);
        let station_texture: Texture2D =
            load_texture("ships/lc/generic_station.png").await?;
            station_texture.set_filter(FilterMode::Nearest);
        let bullet_texture: Texture2D =
            load_texture("ships/bullet02.png").await?;
        bullet_texture.set_filter(FilterMode::Linear);

        info!("Building texture atlas...");
        build_textures_atlas();
        ship_textures.insert("lc.phalanx", ship_texture);
        ship_textures.insert("rf.javelin", ship2_texture);

        Ok(Resources {
            ship_textures,
            bullet_texture,
            station_texture,
            sun_texture,
        })
    }
}
