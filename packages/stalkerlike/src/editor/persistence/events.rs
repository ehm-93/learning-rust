//! File operation events
//!
//! These events represent user actions related to file operations
//! (new, open, save, save as) and decouple the UI from the actual
//! file operation logic.

use bevy::prelude::*;

/// Event to trigger a new file
#[derive(Event)]
pub struct NewFileEvent;

/// Event to trigger open file dialog
#[derive(Event)]
pub struct OpenFileEvent;

/// Event to trigger save
#[derive(Event)]
pub struct SaveEvent;

/// Event to trigger save as dialog
#[derive(Event)]
pub struct SaveAsEvent;
