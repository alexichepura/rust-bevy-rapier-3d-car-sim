use bevy::prelude::*;
use bevy::utils::HashSet;

#[derive(Default)]
pub struct GamepadLobby {
    pub gamepads: HashSet<Gamepad>,
}

pub fn gamepad_stage_preupdate_system(
    mut lobby: ResMut<GamepadLobby>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent {
                gamepad,
                event_type,
            } => {
                lobby.gamepads.insert(*gamepad);
                println!("{:?} Connected", gamepad);
            }
            GamepadEvent {
                gamepad,
                event_type,
            } => {
                lobby.gamepads.remove(gamepad);
                println!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}
