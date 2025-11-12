//! Hierarchy panel for displaying and editing the scene entity tree
//!
//! This module provides:
//! - Tree view of all scene entities with parent-child relationships
//! - Expand/collapse controls for parent entities
//! - Inline name editing
//! - Visibility toggles (show/hide in viewport)
//! - Lock toggles (prevent selection/modification)
//! - Click-to-select synchronization with viewport
//! - Multi-select support (Ctrl+click)
//! - Integrated asset browser at bottom of panel

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::editor::core::types::EditorEntity;
use crate::editor::objects::selection::{SelectionSet, Selected};
use crate::editor::objects::grouping::Group;
use crate::editor::objects::placement::{start_placement, start_placement_asset, PlacementState, PlacementAsset};
use crate::editor::objects::primitives::AssetCatalog;

/// Resource to track which entities have their children expanded in the hierarchy
#[derive(Resource, Default)]
pub struct HierarchyState {
    /// Set of entity IDs that are expanded (showing children)
    pub expanded: HashSet<Entity>,
    /// Entity currently being renamed (if any)
    pub renaming: Option<Entity>,
    /// Text buffer for name editing
    pub name_buffer: String,
}

impl HierarchyState {
    /// Check if an entity is expanded
    pub fn is_expanded(&self, entity: Entity) -> bool {
        self.expanded.contains(&entity)
    }

    /// Toggle expansion state
    pub fn toggle_expanded(&mut self, entity: Entity) {
        if self.expanded.contains(&entity) {
            self.expanded.remove(&entity);
        } else {
            self.expanded.insert(entity);
        }
    }

    /// Start renaming an entity
    pub fn start_rename(&mut self, entity: Entity, current_name: &str) {
        self.renaming = Some(entity);
        self.name_buffer = current_name.to_string();
    }

    /// Finish renaming (returns entity and new name if confirmed)
    pub fn finish_rename(&mut self) -> Option<(Entity, String)> {
        if let Some(entity) = self.renaming {
            let name = self.name_buffer.clone();
            self.renaming = None;
            self.name_buffer.clear();
            Some((entity, name))
        } else {
            None
        }
    }

    /// Cancel renaming
    pub fn cancel_rename(&mut self) {
        self.renaming = None;
        self.name_buffer.clear();
    }
}

/// Marker component for entities that are hidden in the viewport
#[derive(Component)]
pub struct Hidden;

/// Marker component for entities that are locked (cannot be selected)
#[derive(Component)]
pub struct Locked;

/// Represents a discovered GLB/GLTF asset file
#[derive(Debug, Clone)]
pub struct GlbAsset {
    /// Display name (filename without extension)
    pub name: String,
    /// Full path to the asset file
    pub path: PathBuf,
    /// Path relative to the selected asset directory
    pub relative_path: PathBuf,
}

/// Resource to track asset browser state
#[derive(Resource)]
pub struct AssetBrowserState {
    /// Selected asset directory (if any)
    pub asset_directory: Option<PathBuf>,
    /// List of discovered GLB/GLTF files
    pub glb_assets: Vec<GlbAsset>,
    /// Whether the GLB section is expanded
    pub glb_section_expanded: bool,
    /// Pending directory selection task
    pub pending_directory_task: bool,
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        let asset_directory = Self::find_asset_directory();
        let mut state = Self {
            asset_directory,
            glb_assets: Vec::new(),
            glb_section_expanded: true,
            pending_directory_task: false,
        };

        // Automatically scan on startup if we found the directory
        if state.asset_directory.is_some() {
            state.scan_glb_files();
        }

        state
    }
}

impl AssetBrowserState {
    /// Attempt to find the asset directory automatically
    fn find_asset_directory() -> Option<PathBuf> {
        // Try to find the assets directory relative to the current working directory
        let current_dir = std::env::current_dir().ok()?;

        // Try common patterns
        let candidates = vec![
            current_dir.join("packages/stalkerlike/assets"),
            current_dir.join("assets"),
            current_dir.join("../assets"),
        ];

        for candidate in candidates {
            if candidate.exists() && candidate.is_dir() {
                info!("Found asset directory: {:?}", candidate);
                return Some(candidate);
            }
        }

        warn!("Could not automatically find asset directory");
        None
    }

    /// Scan the asset directory for GLB/GLTF files
    pub fn scan_glb_files(&mut self) {
        self.glb_assets.clear();

        let Some(dir) = &self.asset_directory.clone() else {
            return;
        };

        if !dir.exists() || !dir.is_dir() {
            warn!("Asset directory does not exist or is not a directory: {:?}", dir);
            return;
        }

        // Recursively scan for .glb and .gltf files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                self.scan_directory_recursive(&path, dir);
            }
        }

        // Sort by path for consistent display
        self.glb_assets.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        info!("Found {} GLB/GLTF files in {:?}", self.glb_assets.len(), dir);
    }

    fn scan_directory_recursive(&mut self, path: &PathBuf, base_dir: &PathBuf) {
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "glb" || ext_str == "gltf" {
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let relative_path = path
                        .strip_prefix(base_dir)
                        .unwrap_or(path)
                        .to_path_buf();

                    self.glb_assets.push(GlbAsset {
                        name,
                        path: path.clone(),
                        relative_path,
                    });
                }
            }
        } else if path.is_dir() {
            // Recursively scan subdirectories
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    self.scan_directory_recursive(&entry.path(), base_dir);
                }
            }
        }
    }
}

/// Render the hierarchy panel
pub fn hierarchy_ui(
    mut contexts: EguiContexts,
    mut hierarchy_state: ResMut<HierarchyState>,
    mut asset_browser_state: ResMut<AssetBrowserState>,
    mut selection: ResMut<SelectionSet>,
    mut commands: Commands,
    // Query for all editor entities
    editor_query: Query<(Entity, Option<&Name>, Option<&Children>, Option<&ChildOf>), With<EditorEntity>>,
    // Query for component markers
    marker_query: Query<(
        Has<Group>,
        Has<Selected>,
        Has<Hidden>,
        Has<Locked>,
    )>,
    keyboard: Res<ButtonInput<KeyCode>>,
    // Asset browser resources
    asset_catalog: Res<AssetCatalog>,
    mut placement_state: ResMut<PlacementState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::left("hierarchy")
        .default_width(250.0)
        .resizable(true)
        .show(ctx, |ui| {
            // === HIERARCHY SECTION ===
            ui.heading("Hierarchy");
            ui.separator();

            // Collect root entities (those without parents)
            let mut root_entities: Vec<Entity> = editor_query
                .iter()
                .filter(|(_, _, _, child_of)| child_of.is_none())
                .map(|(entity, _, _, _)| entity)
                .collect();

            // Sort root entities by entity index for consistent ordering
            root_entities.sort_by_key(|e| e.index());

            if root_entities.is_empty() {
                ui.label("No entities in scene");
                ui.separator();
                ui.label("Add objects from Assets below");
            } else {
                // Render each root entity and its children recursively
                // Calculate available height for hierarchy (leave space for assets)
                let available_height = ui.available_height() - 200.0; // Reserve 200px for assets
                egui::ScrollArea::vertical()
                    .id_salt("hierarchy_scroll")
                    .max_height(available_height)
                    .show(ui, |ui| {
                        for entity in root_entities {
                            render_entity_node(
                                ui,
                                entity,
                                &mut hierarchy_state,
                                &mut selection,
                                &mut commands,
                                &editor_query,
                                &marker_query,
                                &keyboard,
                                0, // indentation level
                            );
                        }
                    });
            }

            // === ASSET BROWSER SECTION ===
            ui.add_space(8.0);
            ui.separator();
            ui.heading("Assets");
            ui.separator();

            // Asset directory selection
            ui.horizontal(|ui| {
                ui.label("Asset Directory:");
                if ui.button("Browse...").clicked() {
                    // Trigger directory picker (will be handled by a system)
                    asset_browser_state.pending_directory_task = true;
                }
            });

            if let Some(dir) = &asset_browser_state.asset_directory {
                ui.label(format!("📁 {}", dir.display()));
                ui.horizontal(|ui| {
                    if ui.button("🔄 Refresh").clicked() ||
                       (keyboard.just_pressed(KeyCode::F5)) {
                        asset_browser_state.scan_glb_files();
                    }
                    ui.label(format!("{} files", asset_browser_state.glb_assets.len()));
                });
            } else {
                ui.colored_label(egui::Color32::GRAY, "No directory selected");
            }

            ui.add_space(8.0);

            // GLB/GLTF Models Section
            if !asset_browser_state.glb_assets.is_empty() {
                ui.separator();
                let _header_response = ui.horizontal(|ui| {
                    let arrow = if asset_browser_state.glb_section_expanded { "▼" } else { "▶" };
                    if ui.small_button(arrow).clicked() {
                        asset_browser_state.glb_section_expanded = !asset_browser_state.glb_section_expanded;
                    }
                    ui.label("3D Models");
                });

                if asset_browser_state.glb_section_expanded {
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("glb_assets_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for glb_asset in &asset_browser_state.glb_assets.clone() {
                                let button_text = format!("🎨 {}", glb_asset.name);
                                if ui.button(button_text).clicked() {
                                    info!("Starting placement for GLB: {:?} (relative: {:?})",
                                          glb_asset.path, glb_asset.relative_path);

                                    // Use the placement system for GLB models
                                    // Use relative_path to avoid "unapproved path" errors in Bevy 0.16
                                    start_placement_asset(
                                        &mut placement_state,
                                        PlacementAsset::GlbModel {
                                            name: glb_asset.name.clone(),
                                            path: glb_asset.relative_path.clone(),
                                        },
                                        &mut commands,
                                        &mut meshes,
                                        &mut materials,
                                        Some(&asset_server),
                                    );
                                }
                            }
                        });
                }
            }

            ui.add_space(8.0);
            ui.separator();

            // Primitives Section
            ui.label("Primitives");
            ui.add_space(4.0);

            // Render asset buttons in a scrollable area
            egui::ScrollArea::vertical()
                .id_salt("assets_scroll")
                .show(ui, |ui| {
                for primitive in &asset_catalog.primitives {
                    if ui.button(&primitive.name).clicked() {
                        start_placement(
                            &mut placement_state,
                            primitive.clone(),
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                        );
                    }
                }
            });

            if placement_state.active {
                ui.add_space(8.0);
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "Placement Mode");
                if let Some(asset) = &placement_state.selected_asset {
                    let asset_name = match asset {
                        PlacementAsset::Primitive(prim) => &prim.name,
                        PlacementAsset::GlbModel { name, .. } => name,
                    };
                    ui.label(format!("Placing: {}", asset_name));
                }
                ui.label("Click to place");
                ui.label("ESC to cancel");
            }
        });

    // Handle name editing completion
    if let Some((entity, new_name)) = hierarchy_state.finish_rename() {
        if !new_name.is_empty() {
            commands.entity(entity).insert(Name::new(new_name));
            info!("Renamed entity {:?}", entity);
        }
    }
}

/// Recursively render an entity node and its children
#[allow(clippy::too_many_arguments)]
fn render_entity_node(
    ui: &mut egui::Ui,
    entity: Entity,
    hierarchy_state: &mut HierarchyState,
    selection: &mut SelectionSet,
    commands: &mut Commands,
    editor_query: &Query<(Entity, Option<&Name>, Option<&Children>, Option<&ChildOf>), With<EditorEntity>>,
    marker_query: &Query<(Has<Group>, Has<Selected>, Has<Hidden>, Has<Locked>)>,
    keyboard: &Res<ButtonInput<KeyCode>>,
    indent_level: usize,
) {
    let Ok((_, name_opt, children_opt, _)) = editor_query.get(entity) else {
        return;
    };

    let Ok((is_group, is_selected, is_hidden, is_locked)) = marker_query.get(entity) else {
        return;
    };

    ui.horizontal(|ui| {
        // Indentation
        ui.add_space(indent_level as f32 * 16.0);

        // Expand/collapse arrow (only if entity has children)
        let has_children = children_opt.is_some() && !children_opt.unwrap().is_empty();
        if has_children {
            let is_expanded = hierarchy_state.is_expanded(entity);
            let arrow = if is_expanded { "▼" } else { "▶" };
            if ui.small_button(arrow).clicked() {
                hierarchy_state.toggle_expanded(entity);
            }
        } else {
            // Empty space to align with entities that have children
            ui.add_space(20.0);
        }

        // Visibility toggle (eye icon)
        let eye_icon = if is_hidden { "👁" } else { "👁" };
        let eye_color = if is_hidden {
            egui::Color32::GRAY
        } else {
            egui::Color32::WHITE
        };
        if ui.small_button(egui::RichText::new(eye_icon).color(eye_color)).clicked() {
            if is_hidden {
                commands.entity(entity).remove::<Hidden>();
                commands.entity(entity).insert(Visibility::Inherited);
                info!("Showing entity {:?}", entity);
            } else {
                commands.entity(entity).insert(Hidden);
                commands.entity(entity).insert(Visibility::Hidden);
                info!("Hiding entity {:?}", entity);
            }
        }

        // Lock toggle (lock icon)
        let lock_icon = if is_locked { "🔒" } else { "🔓" };
        if ui.small_button(lock_icon).clicked() {
            if is_locked {
                commands.entity(entity).remove::<Locked>();
            } else {
                commands.entity(entity).insert(Locked);
            }
        }

        // Entity name (or debug ID if no name)
        let entity_name = if let Some(name) = name_opt {
            name.as_str().to_string()
        } else {
            format!("Entity {:?}", entity)
        };

        // Entity type indicator (for groups)
        let prefix = if is_group { "📁 " } else { "   " };

        // Selection highlight background
        let text_color = if is_selected {
            egui::Color32::YELLOW
        } else if is_locked {
            egui::Color32::GRAY
        } else {
            egui::Color32::WHITE
        };

        // Check if this entity is being renamed
        if hierarchy_state.renaming == Some(entity) {
            // Show text edit field
            let response = ui.text_edit_singleline(&mut hierarchy_state.name_buffer);
            if response.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // Confirm rename
                    let _ = hierarchy_state.finish_rename();
                } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    // Cancel rename
                    hierarchy_state.cancel_rename();
                }
            }
            // Auto-focus the text field
            response.request_focus();
        } else {
            // Show entity name as clickable label
            let full_name = format!("{}{}", prefix, entity_name);
            let label = ui.selectable_label(is_selected, egui::RichText::new(full_name).color(text_color));

            // Handle click-to-select
            if label.clicked() && !is_locked {
                let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

                if ctrl_held {
                    // Multi-select: toggle selection
                    if selection.contains(entity) {
                        selection.remove(entity);
                        commands.entity(entity).remove::<Selected>();
                    } else {
                        selection.add(entity);
                        commands.entity(entity).insert(Selected);
                    }
                } else {
                    // Single-select: clear previous and select this one
                    // Clear previous selection
                    for &prev_entity in &selection.entities.clone() {
                        commands.entity(prev_entity).remove::<Selected>();
                    }
                    selection.clear();

                    // Select this entity
                    selection.add(entity);
                    commands.entity(entity).insert(Selected);
                }
            }

            // Double-click to rename
            if label.double_clicked() && !is_locked {
                hierarchy_state.start_rename(entity, &entity_name);
            }

            // Right-click context menu (future feature)
            label.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    hierarchy_state.start_rename(entity, &entity_name);
                    ui.close();
                }
                if ui.button("Delete").clicked() {
                    commands.entity(entity).despawn();
                    selection.remove(entity);
                    ui.close();
                }
                if ui.button("Duplicate").clicked() {
                    // TODO: Implement duplicate (Ctrl+D functionality)
                    ui.close();
                }
            });
        }
    });

    // Render children if expanded
    if hierarchy_state.is_expanded(entity) {
        if let Some(children) = children_opt {
            for child in children.iter() {
                render_entity_node(
                    ui,
                    child,
                    hierarchy_state,
                    selection,
                    commands,
                    editor_query,
                    marker_query,
                    keyboard,
                    indent_level + 1,
                );
            }
        }
    }
}

/// Component to track pending directory picker task
#[derive(Component)]
pub struct DirectoryPickerTask(Task<Option<PathBuf>>);

/// System to handle directory picker dialog requests
pub fn handle_directory_picker(
    mut commands: Commands,
    mut asset_browser_state: ResMut<AssetBrowserState>,
) {
    if asset_browser_state.pending_directory_task {
        asset_browser_state.pending_directory_task = false;

        // Spawn async directory picker task
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            rfd::AsyncFileDialog::new()
                .set_title("Select Asset Directory")
                .pick_folder()
                .await
                .map(|handle| handle.path().to_path_buf())
        });

        commands.spawn(DirectoryPickerTask(task));
    }
}

/// System to poll directory picker tasks
pub fn poll_directory_picker_tasks(
    mut commands: Commands,
    mut asset_browser_state: ResMut<AssetBrowserState>,
    mut tasks: Query<(Entity, &mut DirectoryPickerTask)>,
) {
    for (task_entity, mut task) in tasks.iter_mut() {
        if let Some(result) = block_on(futures_lite::future::poll_once(&mut task.0)) {
            // Task is complete, despawn it
            commands.entity(task_entity).despawn();

            if let Some(path) = result {
                info!("Selected asset directory: {:?}", path);
                asset_browser_state.asset_directory = Some(path);
                asset_browser_state.scan_glb_files();
                asset_browser_state.glb_section_expanded = true;
            }
        }
    }
}
