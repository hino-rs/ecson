use bevy_ecs::prelude::*;
use std::collections::HashMap;

// Entityを一発で引くための内部リソース
#[derive(Resource, Default)]
pub struct ConnectionMap(pub HashMap<u64, Entity>);

// サーバーのTickレート
#[derive(Resource)]
pub struct ServerTickRate(pub f64);
impl Default for ServerTickRate {
    fn default() -> Self {
        Self::NORMAL
    }
}
impl ServerTickRate {
    pub const ECO: Self = Self(10.0);
    pub const NORMAL: Self = Self(60.0);
    pub const HIGH: Self = Self(120.0);
}