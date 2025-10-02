use bevy::prelude::*;

/// Dungeon marker component for entities that belong to the dungeon scene
#[derive(Component, Debug)]
pub struct DungeonEntity;

/// Portal to sanctuary from dungeon
#[derive(Component, Debug)]
pub struct DungeonExitPortal;
