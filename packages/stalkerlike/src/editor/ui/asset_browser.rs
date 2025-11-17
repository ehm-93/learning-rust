use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::editor::objects::{
    placement::{start_placement, start_placement_asset, PlacementState, PlacementAsset},
    primitives::AssetCatalog,
};
use crate::editor::ui::hierarchy::AssetBrowserState;

/// Resource to track asset browser section expansion state
#[derive(Resource)]
pub struct AssetBrowserSections {
    /// Whether the Primitives section is expanded
    pub primitives_expanded: bool,
    /// Whether the Models section is expanded
    pub models_expanded: bool,
    /// Which model folders are expanded (by directory path)
    pub expanded_folders: HashMap<PathBuf, bool>,
    /// Search filter text (case-insensitive)
    pub search_filter: String,
}

impl Default for AssetBrowserSections {
    fn default() -> Self {
        Self {
            primitives_expanded: true,
            models_expanded: true,
            expanded_folders: HashMap::new(),
            search_filter: String::new(),
        }
    }
}

/// Represents a folder in the asset hierarchy
#[derive(Debug, Clone)]
struct AssetFolder {
    name: String,
    path: PathBuf,
    assets: Vec<usize>, // Indices into AssetBrowserState.glb_assets
    subfolders: Vec<AssetFolder>,
}

impl AssetFolder {
    fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            assets: Vec::new(),
            subfolders: Vec::new(),
        }
    }
}

/// Check if asset name matches filter (case-insensitive)
fn matches_filter(name: &str, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    name.to_lowercase().contains(&filter.to_lowercase())
}

/// Build a folder tree from flat asset list, filtered by search
fn build_folder_tree(assets: &[crate::editor::ui::hierarchy::GlbAsset], filter: &str) -> Vec<AssetFolder> {
    if assets.is_empty() {
        return Vec::new();
    }

    // Group assets by their directory (filter by search)
    let mut folder_map: HashMap<PathBuf, Vec<usize>> = HashMap::new();

    for (idx, asset) in assets.iter().enumerate() {
        // Skip assets that don't match filter
        if !matches_filter(&asset.name, filter) {
            continue;
        }

        let parent_dir = if let Some(parent) = asset.relative_path.parent() {
            parent.to_path_buf()
        } else {
            PathBuf::from("")
        };

        folder_map.entry(parent_dir).or_default().push(idx);
    }

    // Build tree structure
    let mut root_folders = Vec::new();

    // Get unique directories and sort them
    let mut directories: Vec<PathBuf> = folder_map.keys().cloned().collect();
    directories.sort();

    // Build folder hierarchy
    for dir in &directories {
        if dir.as_os_str().is_empty() {
            // Root level assets
            if let Some(asset_indices) = folder_map.get(dir) {
                for &idx in asset_indices {
                    // Create a pseudo-folder for each root asset
                    let asset = &assets[idx];
                    let mut folder = AssetFolder::new(
                        asset.name.clone(),
                        asset.relative_path.clone(),
                    );
                    folder.assets.push(idx);
                    root_folders.push(folder);
                }
            }
        } else {
            // Check if this is a top-level directory (no parent in our set)
            let is_top_level = !directories.iter().any(|other| {
                other != dir && dir.starts_with(other) && other.as_os_str() != ""
            });

            if is_top_level {
                let folder = build_folder_recursive(dir, &folder_map, &directories, assets);
                root_folders.push(folder);
            }
        }
    }

    root_folders
}

/// Recursively build folder structure
fn build_folder_recursive(
    dir: &Path,
    folder_map: &HashMap<PathBuf, Vec<usize>>,
    all_dirs: &[PathBuf],
    assets: &[crate::editor::ui::hierarchy::GlbAsset],
) -> AssetFolder {
    let folder_name = dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut folder = AssetFolder::new(folder_name, dir.to_path_buf());

    // Add direct assets in this folder
    if let Some(asset_indices) = folder_map.get(dir) {
        folder.assets.extend(asset_indices);
    }

    // Find and add subfolders
    for other_dir in all_dirs {
        if other_dir == dir {
            continue;
        }

        // Check if other_dir is a direct child of dir
        if let Some(parent) = other_dir.parent() {
            if parent == dir {
                let subfolder = build_folder_recursive(other_dir, folder_map, all_dirs, assets);
                folder.subfolders.push(subfolder);
            }
        }
    }

    folder
}

/// Render a folder node in the tree
fn render_folder_node(
    ui: &mut egui::Ui,
    folder: &AssetFolder,
    assets: &[crate::editor::ui::hierarchy::GlbAsset],
    sections: &mut AssetBrowserSections,
    placement_state: &mut PlacementState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    indent_level: usize,
) {
    // Check if this folder is expanded
    let is_expanded = sections
        .expanded_folders
        .get(&folder.path)
        .copied()
        .unwrap_or(false);

    // Check if has subfolders or multiple assets
    let has_children = !folder.subfolders.is_empty() || folder.assets.len() > 1;

    ui.horizontal(|ui| {
        // Indentation
        ui.add_space(indent_level as f32 * 12.0);

        // Expand/collapse arrow (only if has subfolders or multiple assets)
        if has_children {
            let arrow = if is_expanded { "▼" } else { "▶" };
            if ui.small_button(arrow).clicked() {
                sections
                    .expanded_folders
                    .insert(folder.path.clone(), !is_expanded);
            }
        } else {
            ui.add_space(20.0); // Alignment space
        }

        // Folder icon and name
        ui.label(format!("📁 {}", folder.name));
    });

    // Show contents if expanded
    if is_expanded || !has_children {
        // Show assets in this folder
        for &asset_idx in &folder.assets {
            let asset = &assets[asset_idx];

            ui.horizontal(|ui| {
                ui.add_space((indent_level + 1) as f32 * 12.0 + 20.0);

                let button_text = format!("🎨 {}", asset.name);
                if ui.button(button_text).clicked() {
                    start_placement_asset(
                        placement_state,
                        PlacementAsset::GlbModel {
                            name: asset.name.clone(),
                            path: asset.relative_path.clone(),
                        },
                        commands,
                        meshes,
                        materials,
                        Some(asset_server),
                    );
                }
            });
        }

        // Show subfolders
        for subfolder in &folder.subfolders {
            render_folder_node(
                ui,
                subfolder,
                assets,
                sections,
                placement_state,
                commands,
                meshes,
                materials,
                asset_server,
                indent_level + 1,
            );
        }
    }
}

/// Render the asset browser panel
pub fn asset_browser_ui(
    mut contexts: EguiContexts,
    asset_catalog: Res<AssetCatalog>,
    asset_browser_state: Res<AssetBrowserState>,
    mut sections: ResMut<AssetBrowserSections>,
    mut placement_state: ResMut<PlacementState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::left("asset_browser")
        .default_width(250.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Assets");
            ui.separator();

            // === SEARCH BAR ===
            ui.horizontal(|ui| {
                ui.label("🔍");
                let response = ui.text_edit_singleline(&mut sections.search_filter);
                if response.changed() {
                    // Search filter changed - no need to do anything else
                    // The filter will be applied when rendering below
                }
                if ui.button("✖").clicked() {
                    sections.search_filter.clear();
                }
            });
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .id_salt("asset_browser_scroll")
                .show(ui, |ui| {
                    // === PRIMITIVES SECTION ===
                    ui.horizontal(|ui| {
                        let arrow = if sections.primitives_expanded {
                            "▼"
                        } else {
                            "▶"
                        };
                        if ui.small_button(arrow).clicked() {
                            sections.primitives_expanded = !sections.primitives_expanded;
                        }
                        ui.heading("Primitives");
                    });

                    if sections.primitives_expanded {
                        ui.add_space(4.0);
                        // Filter primitives by search
                        let filtered_primitives: Vec<_> = asset_catalog.primitives
                            .iter()
                            .filter(|p| matches_filter(&p.name, &sections.search_filter))
                            .collect();

                        if filtered_primitives.is_empty() && !sections.search_filter.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.colored_label(egui::Color32::GRAY, "No matches");
                            });
                        } else {
                            for primitive in filtered_primitives {
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0); // Indent under header
                                    if ui.button(&primitive.name).clicked() {
                                        start_placement(
                                            &mut placement_state,
                                            primitive.clone(),
                                            &mut commands,
                                            &mut meshes,
                                            &mut materials,
                                        );
                                    }
                                });
                            }
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // === MODELS SECTION ===
                    ui.horizontal(|ui| {
                        let arrow = if sections.models_expanded { "▼" } else { "▶" };
                        if ui.small_button(arrow).clicked() {
                            sections.models_expanded = !sections.models_expanded;
                        }
                        ui.heading("Models");
                        if !asset_browser_state.glb_assets.is_empty() {
                            ui.label(format!("({})", asset_browser_state.glb_assets.len()));
                        }
                    });

                    if sections.models_expanded {
                        if asset_browser_state.glb_assets.is_empty() {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.colored_label(egui::Color32::GRAY, "No models found");
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Add .glb files to assets/models/");
                            });
                        } else {
                            ui.add_space(4.0);

                            // Build folder tree from assets (filtered)
                            let folder_tree = build_folder_tree(&asset_browser_state.glb_assets, &sections.search_filter);

                            if folder_tree.is_empty() && !sections.search_filter.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    ui.colored_label(egui::Color32::GRAY, "No matches");
                                });
                            } else {
                                // Render folder tree
                                for folder in &folder_tree {
                                    render_folder_node(
                                        ui,
                                        folder,
                                        &asset_browser_state.glb_assets,
                                        &mut sections,
                                        &mut placement_state,
                                        &mut commands,
                                        &mut meshes,
                                        &mut materials,
                                        &asset_server,
                                        0,
                                    );
                                }
                            }
                        }
                    }

                    ui.add_space(8.0);
                });

            // === PLACEMENT STATUS ===
            if placement_state.active {
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
}
