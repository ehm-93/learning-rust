use bevy::prelude::*;

/// Sanctuary marker component for entities that belong to the sanctuary scene
#[derive(Component, Debug)]
pub struct SanctuaryEntity;

/// Exit portal back to cathedral
#[derive(Component, Debug)]
pub struct SanctuaryExitPortal;

/// Dungeon portal in sanctuary
#[derive(Component, Debug)]
pub struct SanctuaryDungeonPortal;
