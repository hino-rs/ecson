//! ECS resources shared across the application.

use crate::channels::NetworkEvent;
use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

/// Internal resource for O(1) lookup of an `Entity` by its network connection ID (`u64`).
#[derive(Resource, Default)]
pub struct ConnectionMap(pub HashMap<u64, Entity>);

/// Resource for O(1) lookup of the set of `Entity`s in a room, keyed by room name (`String`).
#[derive(Resource, Default)]
pub struct RoomMap(pub HashMap<String, HashSet<Entity>>);

/// Resource holding the channel sender used to send events from ECS to the Tokio side.
#[derive(Resource)]
pub struct NetworkSender(pub mpsc::Sender<NetworkEvent>);
