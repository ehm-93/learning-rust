//! Save and load systems triggered by keyboard shortcuts or UI events

use bevy::prelude::*;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::editor::persistence::scene::{save_scene, load_scene};
use crate::editor::persistence::events::{NewFileEvent, OpenFileEvent, SaveEvent, SaveAsEvent};
use crate::editor::core::types::{EditorEntity, PlayerSpawn, RigidBodyType, GlbModel};
use crate::editor::ui::confirmation_dialog::{ConfirmationDialog, ErrorDialog, PendingAction, AutoSaveRecoveryDialog, AutoSaveChoice};

/// Resource for showing autosave notifications
#[derive(Resource, Default)]
pub struct AutoSaveNotification {
    /// Message to display
    pub message: Option<String>,
    /// Timer for how long to show the notification
    pub timer: Timer,
}

impl AutoSaveNotification {
    /// Show a notification for 3 seconds
    pub fn show(&mut self, message: String) {
        self.message = Some(message);
        self.timer = Timer::new(Duration::from_secs(3), TimerMode::Once);
    }

    /// Update the timer and clear message when expired
    pub fn update(&mut self, delta: Duration) {
        if self.message.is_some() {
            self.timer.tick(delta);
            if self.timer.finished() {
                self.message = None;
            }
        }
    }
}

/// Resource tracking the current scene file
#[derive(Resource)]
pub struct CurrentFile {
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub last_saved: Option<Instant>,
}

impl Default for CurrentFile {
    fn default() -> Self {
        Self {
            path: None,
            dirty: false,
            last_saved: None,
        }
    }
}

impl CurrentFile {
    /// Get the current file path or the default path
    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    /// Set the current file path
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    /// Mark the file as clean (saved)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.last_saved = Some(Instant::now());
    }

    /// Mark the file as dirty (unsaved changes)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if a file path is set
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    /// Get the filename for display
    pub fn get_filename(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            .unwrap_or("untitled")
            .to_string()
    }

    /// Get a human-readable string for how long ago the file was saved
    pub fn get_last_saved_text(&self) -> String {
        match self.last_saved {
            Some(instant) => {
                let elapsed = instant.elapsed();
                if elapsed.as_secs() < 60 {
                    "just now".to_string()
                } else if elapsed.as_secs() < 3600 {
                    format!("{}m ago", elapsed.as_secs() / 60)
                } else {
                    format!("{}h ago", elapsed.as_secs() / 3600)
                }
            }
            None => "never".to_string(),
        }
    }
}

/// System to handle save scene keyboard shortcut (Ctrl+S)
pub fn save_scene_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_file: ResMut<CurrentFile>,
    mut save_as_events: EventWriter<SaveAsEvent>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
        Option<&PlayerSpawn>,
        Option<&RigidBodyType>,
        Option<&GlbModel>,
        Option<&PointLight>,
        Option<&SpotLight>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    directional_light: Query<(&DirectionalLight, &Transform), Without<EditorEntity>>,
    ambient_light: Res<AmbientLight>,
) {
    // Check for Ctrl+S
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyS) {
            // If no file is open, trigger Save As dialog instead
            if !current_file.has_path() {
                info!("No file open, opening Save As dialog...");
                save_as_events.write(SaveAsEvent);
                return;
            }

            // Get the current file path (we know it exists from has_path check)
            let Some(path) = current_file.get_path() else {
                return;
            };

            // Ensure the directory exists
            if let Some(parent) = path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    error!("Failed to create directory {}: {}", parent.display(), e);
                    return;
                }
            }

            match save_scene(path.clone(), editor_entities, meshes, materials, directional_light, ambient_light) {
                Ok(()) => {
                    info!("Scene saved to {}", path.display());
                    current_file.set_path(path.clone());
                    current_file.mark_clean();

                    // Delete the autosave file after successful save
                    let autosave_path = path.with_file_name(
                        format!("{}.autosave", path.file_name().unwrap().to_string_lossy())
                    );
                    if autosave_path.exists() {
                        if let Err(e) = std::fs::remove_file(&autosave_path) {
                            warn!("Failed to delete autosave file after save: {}", e);
                        } else {
                            info!("Deleted autosave file after successful save");
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to save scene: {}", e);
                }
            }
        }
    }
}

/// System to handle load scene keyboard shortcut (Ctrl+O)
/// Opens the file picker dialog to select a scene file
pub fn load_scene_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_file: Res<CurrentFile>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut open_file_events: EventWriter<OpenFileEvent>,
) {
    // Check for Ctrl+O
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyO) {
            // Check for unsaved changes
            if current_file.is_dirty() {
                dialog.request(PendingAction::OpenFile);
            } else {
                // Trigger the Open File dialog
                info!("Opening file picker dialog...");
                open_file_events.write(OpenFileEvent);
            }
        }
    }
}

/// System to mark scene as dirty when entities are modified
pub fn mark_scene_dirty(
    mut current_file: ResMut<CurrentFile>,
    changed_entities: Query<
        Entity,
        (
            With<EditorEntity>,
            Or<(
                Changed<Transform>,
                Changed<Mesh3d>,
                Changed<MeshMaterial3d<StandardMaterial>>,
            )>,
        ),
    >,
) {
    if !changed_entities.is_empty() {
        current_file.mark_dirty();
    }
}

/// System to handle new file event
pub fn handle_new_file(
    mut events: EventReader<NewFileEvent>,
    mut current_file: ResMut<CurrentFile>,
    mut commands: Commands,
    editor_entities: Query<Entity, With<EditorEntity>>,
    existing_lights: Query<Entity, With<DirectionalLight>>,
) {
    for _ in events.read() {
        // Clear the scene
        for entity in editor_entities.iter() {
            commands.entity(entity).despawn();
        }

        // Clear existing directional lights
        for entity in existing_lights.iter() {
            commands.entity(entity).despawn();
        }

        // Spawn default lighting
        commands.spawn((
            DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));

        commands.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 400.0,
            ..default()
        });

        // Reset current file
        current_file.path = None;
        current_file.mark_clean();

        info!("New file created");
    }
}

/// System to handle save event
pub fn handle_save(
    mut events: EventReader<SaveEvent>,
    mut current_file: ResMut<CurrentFile>,
    mut error_dialog: ResMut<ErrorDialog>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
        Option<&PlayerSpawn>,
        Option<&RigidBodyType>,
        Option<&GlbModel>,
        Option<&PointLight>,
        Option<&SpotLight>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    directional_light: Query<(&DirectionalLight, &Transform), Without<EditorEntity>>,
    ambient_light: Res<AmbientLight>,
) {
    for _ in events.read() {
        // Skip save if no file is open - should not happen since button is disabled
        if !current_file.has_path() {
            warn!("No file open, cannot save.");
            continue;
        }

        let Some(path) = current_file.get_path() else {
            continue;
        };

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!("Failed to create directory {}: {}", parent.display(), e);
                error_dialog.show_error(
                    "Save Failed",
                    format!("Failed to create directory {}:\n{}", parent.display(), e)
                );
                return;
            }
        }

        match save_scene(path.clone(), editor_entities, Res::clone(&meshes), Res::clone(&materials), directional_light, Res::clone(&ambient_light)) {
            Ok(()) => {
                info!("Scene saved to {}", path.display());
                current_file.set_path(path);
                current_file.mark_clean();
            }
            Err(e) => {
                error!("Failed to save scene: {}", e);
                error_dialog.show_error(
                    "Save Failed",
                    format!("Failed to save scene:\n{}", e)
                );
            }
        }
    }
}

/// Component to track pending file open task
#[derive(Component)]
pub(crate) struct FileOpenTask(Task<Option<PathBuf>>);

/// System to handle open file event
/// Opens a file picker dialog to select a scene file
pub fn handle_open_file(
    mut events: EventReader<OpenFileEvent>,
    mut commands: Commands,
) {
    for _ in events.read() {
        // Spawn async file dialog task
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            rfd::AsyncFileDialog::new()
                .add_filter("Scene Files", &["yaml", "yml"])
                .set_title("Open Scene")
                .pick_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        });

        commands.spawn(FileOpenTask(task));
    }
}

/// System to poll file open tasks and load the scene when ready
pub fn poll_file_open_tasks(
    mut commands: Commands,
    mut current_file: ResMut<CurrentFile>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut autosave_dialog: ResMut<AutoSaveRecoveryDialog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut tasks: Query<(Entity, &mut FileOpenTask)>,
    editor_entities: Query<Entity, With<EditorEntity>>,
    existing_lights: Query<Entity, With<DirectionalLight>>,
) {
    // First, check if we have a pending autosave dialog with a user choice
    if autosave_dialog.show && autosave_dialog.user_choice.is_some() {
        let choice = autosave_dialog.user_choice.unwrap();
        let original_path = autosave_dialog.original_path.clone().unwrap();
        let autosave_path = autosave_dialog.autosave_path.clone().unwrap();

        let load_path = match choice {
            AutoSaveChoice::RecoverAutosave => {
                info!("User chose to recover from autosave");
                autosave_path.clone()
            }
            AutoSaveChoice::LoadOriginal => {
                info!("User chose to load original file");
                original_path.clone()
            }
        };

        // Clear the dialog before loading
        autosave_dialog.close();

        // Clear existing scene entities
        for entity in editor_entities.iter() {
            commands.entity(entity).despawn();
        }

        // Clear existing directional lights (load_scene will create new ones)
        for entity in existing_lights.iter() {
            commands.entity(entity).despawn();
        }

        match load_scene(load_path.clone(), &mut commands, &mut meshes, &mut materials, &asset_server) {
            Ok(()) => {
                info!("Scene loaded from {}", load_path.display());
                // Always set the current file to the original path, not the autosave
                current_file.set_path(original_path);
                current_file.mark_clean();

                // If we loaded from autosave, delete it now
                if choice == AutoSaveChoice::RecoverAutosave {
                    if let Err(e) = std::fs::remove_file(&autosave_path) {
                        warn!("Failed to delete autosave file after recovery: {}", e);
                    } else {
                        info!("Deleted autosave file after successful recovery");
                    }
                }
            }
            Err(e) => {
                error!("Failed to load scene: {}", e);
                error_dialog.show_error(
                    "Load Failed",
                    format!("Failed to load scene from {}:\n{}", load_path.display(), e)
                );
            }
        }
        return; // Don't process new tasks this frame
    }

    // Process new file open tasks
    for (task_entity, mut task) in tasks.iter_mut() {
        if let Some(result) = block_on(futures_lite::future::poll_once(&mut task.0)) {
            // Task is complete, despawn it
            commands.entity(task_entity).despawn();

            if let Some(path) = result {
                // Check if an autosave file exists
                let autosave_path = path.with_file_name(
                    format!("{}.autosave", path.file_name().unwrap().to_string_lossy())
                );

                if autosave_path.exists() {
                    // Show the autosave recovery dialog and wait for user input
                    info!("Autosave found at {}, showing recovery dialog", autosave_path.display());
                    autosave_dialog.request(path, autosave_path);
                    continue;
                }

                // No autosave - proceed with loading immediately
                // Clear existing scene entities
                for entity in editor_entities.iter() {
                    commands.entity(entity).despawn();
                }

                // Clear existing directional lights (load_scene will create new ones)
                for entity in existing_lights.iter() {
                    commands.entity(entity).despawn();
                }

                match load_scene(path.clone(), &mut commands, &mut meshes, &mut materials, &asset_server) {
                    Ok(()) => {
                        info!("Scene loaded from {}", path.display());
                        current_file.set_path(path);
                        current_file.mark_clean();
                    }
                    Err(e) => {
                        error!("Failed to load scene: {}", e);
                        error_dialog.show_error(
                            "Load Failed",
                            format!("Failed to load scene from {}:\n{}", path.display(), e)
                        );
                    }
                }
            }
        }
    }
}

/// Component to track pending save as task
#[derive(Component)]
pub(crate) struct FileSaveTask(Task<Option<PathBuf>>);

/// System to handle save as event
/// Opens a file picker dialog to choose where to save the scene
pub fn handle_save_as(
    mut events: EventReader<SaveAsEvent>,
    mut commands: Commands,
) {
    for _ in events.read() {
        // Spawn async file dialog task
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            rfd::AsyncFileDialog::new()
                .add_filter("Scene Files", &["yaml", "yml"])
                .set_title("Save Scene As")
                .save_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        });

        commands.spawn(FileSaveTask(task));
    }
}

/// System to poll save as tasks and save the scene when ready
pub fn poll_save_as_tasks(
    mut commands: Commands,
    mut current_file: ResMut<CurrentFile>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut tasks: Query<(Entity, &mut FileSaveTask)>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
        Option<&PlayerSpawn>,
        Option<&RigidBodyType>,
        Option<&GlbModel>,
        Option<&PointLight>,
        Option<&SpotLight>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    directional_light: Query<(&DirectionalLight, &Transform), Without<EditorEntity>>,
    ambient_light: Res<AmbientLight>,
) {
    for (task_entity, mut task) in tasks.iter_mut() {
        if let Some(result) = block_on(futures_lite::future::poll_once(&mut task.0)) {
            // Task is complete, despawn it
            commands.entity(task_entity).despawn();

            if let Some(path) = result {
                // Ensure the directory exists
                if let Some(parent) = path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        error!("Failed to create directory {}: {}", parent.display(), e);
                        error_dialog.show_error(
                            "Save Failed",
                            format!("Failed to create directory {}:\n{}", parent.display(), e)
                        );
                        return;
                    }
                }

                match save_scene(path.clone(), editor_entities, Res::clone(&meshes), Res::clone(&materials), directional_light, Res::clone(&ambient_light)) {
                    Ok(()) => {
                        info!("Scene saved as {}", path.display());
                        current_file.set_path(path.clone());
                        current_file.mark_clean();

                        // Delete the autosave file after successful save
                        let autosave_path = path.with_file_name(
                            format!("{}.autosave", path.file_name().unwrap().to_string_lossy())
                        );
                        if autosave_path.exists() {
                            if let Err(e) = std::fs::remove_file(&autosave_path) {
                                warn!("Failed to delete autosave file after save: {}", e);
                            } else {
                                info!("Deleted autosave file after successful save");
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to save scene: {}", e);
                        error_dialog.show_error(
                            "Save Failed",
                            format!("Failed to save scene:\n{}", e)
                        );
                    }
                }
            }
        }
    }
}

/// Autosave interval options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoSaveInterval {
    Disabled,
    OneMinute,
    FiveMinutes,
    TenMinutes,
    FifteenMinutes,
    ThirtyMinutes,
}

impl AutoSaveInterval {
    /// Get the duration for this interval
    pub fn duration(&self) -> Option<Duration> {
        match self {
            AutoSaveInterval::Disabled => None,
            AutoSaveInterval::OneMinute => Some(Duration::from_secs(60)),
            AutoSaveInterval::FiveMinutes => Some(Duration::from_secs(5 * 60)),
            AutoSaveInterval::TenMinutes => Some(Duration::from_secs(10 * 60)),
            AutoSaveInterval::FifteenMinutes => Some(Duration::from_secs(15 * 60)),
            AutoSaveInterval::ThirtyMinutes => Some(Duration::from_secs(30 * 60)),
        }
    }

    /// Get a human-readable label for this interval
    pub fn label(&self) -> &'static str {
        match self {
            AutoSaveInterval::Disabled => "Disabled",
            AutoSaveInterval::OneMinute => "1 minute",
            AutoSaveInterval::FiveMinutes => "5 minutes",
            AutoSaveInterval::TenMinutes => "10 minutes",
            AutoSaveInterval::FifteenMinutes => "15 minutes",
            AutoSaveInterval::ThirtyMinutes => "30 minutes",
        }
    }

    /// Get all available intervals
    pub fn all() -> &'static [AutoSaveInterval] {
        &[
            AutoSaveInterval::Disabled,
            AutoSaveInterval::OneMinute,
            AutoSaveInterval::FiveMinutes,
            AutoSaveInterval::TenMinutes,
            AutoSaveInterval::FifteenMinutes,
            AutoSaveInterval::ThirtyMinutes,
        ]
    }
}

/// Resource to track autosave timing
#[derive(Resource)]
pub struct AutoSaveTimer {
    /// Timer for autosave interval
    pub timer: Timer,
    /// Current interval setting
    pub interval: AutoSaveInterval,
}

impl Default for AutoSaveTimer {
    fn default() -> Self {
        let interval = AutoSaveInterval::FiveMinutes;
        let duration = interval.duration().unwrap_or(Duration::from_secs(5 * 60));
        Self {
            timer: Timer::new(duration, TimerMode::Repeating),
            interval,
        }
    }
}

impl AutoSaveTimer {
    /// Set a new interval and reset the timer
    pub fn set_interval(&mut self, interval: AutoSaveInterval) {
        self.interval = interval;
        if let Some(duration) = interval.duration() {
            self.timer = Timer::new(duration, TimerMode::Repeating);
        }
    }

    /// Check if autosave is enabled
    pub fn is_enabled(&self) -> bool {
        self.interval != AutoSaveInterval::Disabled
    }
}

/// System to automatically save the scene at regular intervals
pub fn autosave_system(
    time: Res<Time>,
    mut timer: ResMut<AutoSaveTimer>,
    current_file: Res<CurrentFile>,
    mut notification: ResMut<AutoSaveNotification>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
        Option<&PlayerSpawn>,
        Option<&RigidBodyType>,
        Option<&GlbModel>,
        Option<&PointLight>,
        Option<&SpotLight>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    directional_light: Query<(&DirectionalLight, &Transform), Without<EditorEntity>>,
    ambient_light: Res<AmbientLight>,
) {
    // Tick the timer
    timer.timer.tick(time.delta());

    // Only autosave if:
    // 1. Autosave is enabled (not Disabled)
    // 2. Timer finished
    // 3. There's a file open
    // 4. There are unsaved changes
    if timer.is_enabled()
        && timer.timer.just_finished()
        && current_file.has_path()
        && current_file.is_dirty()
    {
        let Some(path) = current_file.get_path() else {
            return;
        };

        // Create autosave filename by appending .autosave to the original filename
        let autosave_path = if let Some(filename) = path.file_name() {
            let autosave_filename = format!("{}.autosave", filename.to_string_lossy());
            path.with_file_name(autosave_filename)
        } else {
            // Fallback: use the parent directory with .autosave extension
            path.with_extension("autosave")
        };

        // Ensure the directory exists
        if let Some(parent) = autosave_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!("Autosave failed to create directory {}: {}", parent.display(), e);
                notification.show(format!("Autosave failed: {}", e));
                return;
            }
        }

        match save_scene(autosave_path.clone(), editor_entities, meshes, materials, directional_light, ambient_light) {
            Ok(()) => {
                info!("Autosaved scene to {}", autosave_path.display());
                notification.show(format!("Autosaved to {}", autosave_path.file_name().unwrap_or_default().to_string_lossy()));
                // Note: We don't mark the file as clean since autosave is a backup,
                // not a real save. The user still needs to save manually.
            }
            Err(e) => {
                error!("Autosave failed: {}", e);
                notification.show(format!("Autosave failed: {}", e));
            }
        }
    }
}
