use bevy::prelude::*;
use crate::world::InteractionEvent;

use super::{
    components::*,
    resources::*,
};



/// Initialize portal configurations with modifiers
pub fn initialize_portals(
    mut modifier_system: ResMut<ModifierSystem>,
    mut portal_query: Query<&mut Portal>,
    progression_state: Res<ProgressionState>,
) {
    let mut rng = rand::rng();

    // Get the current available depths
    let available_depths = progression_state.get_available_depths();
    let current_depth = available_depths.first().copied().unwrap_or(1);

    // Generate modifiers for the current depth
    modifier_system.generate_portal_modifiers(current_depth, &mut rng);

    // Update portal components with their modifiers
    for mut portal in portal_query.iter_mut() {
        portal.depth = current_depth;
        portal.modifiers = modifier_system.get_portal_modifiers(current_depth, portal.id);
    }

    info!("Portals initialized for depth {} with modifiers", current_depth);
}

/// Update portal display texts with current modifier information
pub fn update_portal_displays(
    portal_query: Query<&Portal>,
    mut display_query: Query<&mut Text2d, With<PortalDisplay>>,
    display_component_query: Query<&PortalDisplay>,
) {
    for mut text2d in display_query.iter_mut() {
        // Find the corresponding PortalDisplay component
        if let Some(display_component) = display_component_query.iter()
            .find(|_d| {
                // This is a bit hacky - we need to match the display component to the text
                // In a more robust system, we'd store entity IDs or use a better linking method
                true // For now, we'll update all displays
            }) {

            // Find the portal with the matching ID
            if let Some(portal) = portal_query.iter()
                .find(|p| p.id == display_component.portal_id) {

                let mut display_text = format!("Depth {}\n", portal.depth);

                if portal.modifiers.is_empty() {
                    display_text.push_str("No Modifiers");
                } else {
                    for modifier in &portal.modifiers {
                        display_text.push_str(&format!("• {}\n", modifier.display_name()));
                    }
                }

                **text2d = display_text;
            }
        }
    }
}



/// Handle portal interactions through the event system
pub fn handle_portal_interaction_events(
    mut interaction_events: EventReader<InteractionEvent>,
    mut portal_activation_events: EventWriter<crate::events::PortalActivationEvent>,
    portal_query: Query<&Portal>,
) {
    for event in interaction_events.read() {
        // Check if the interaction is with a portal (by checking if the entity has a Portal component)
        if let Ok(portal) = portal_query.get(event.target_entity) {
            info!("Portal interaction detected: {:?} -> depth {}", portal.id, portal.depth);

            // Send portal activation event instead of directly transitioning
            portal_activation_events.write(crate::events::PortalActivationEvent {
                portal_id: portal.id,
                depth: portal.depth,
                modifiers: portal.modifiers.iter().map(|m| m.display_name()).collect(),
            });

            info!("Portal activation event sent for {:?}", portal.id);
            return;
        }
    }
}

/// Direct portal activation handler - currently disabled since dungeons are not implemented
pub fn handle_portal_activation(
    mut events: EventReader<crate::events::PortalActivationEvent>,
    current_state: Res<State<crate::world::states::WorldState>>,
    mut _next_state: ResMut<NextState<crate::world::states::WorldState>>,
) {
    use crate::world::states::WorldState;

    for event in events.read() {
        if matches!(current_state.get(), WorldState::Cathedral) {
            info!("Cathedral: Portal activated (depth {}) - dungeons not yet implemented", event.depth);
            // TODO: Implement dungeon transitions when dungeon system is ready
        }
    }
}
