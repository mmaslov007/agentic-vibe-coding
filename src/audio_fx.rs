use bevy::{audio::Volume, prelude::*};
use std::time::Duration;

pub struct AudioFxPlugin;

impl Plugin for AudioFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_audio_fx);
    }
}

#[derive(Resource)]
pub struct SoundEffects {
    pub rifle_shot: Handle<Pitch>,
    pub pistol_shot: Handle<Pitch>,
    pub footstep: Handle<Pitch>,
    pub zombie_groan: Handle<Pitch>,
}

pub fn play_sound(commands: &mut Commands, sound: Handle<Pitch>, volume: f32, speed: f32) {
    commands.spawn((
        AudioPlayer(sound),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(volume))
            .with_speed(speed),
    ));
}

fn setup_audio_fx(mut commands: Commands, mut pitch_assets: ResMut<Assets<Pitch>>) {
    commands.insert_resource(SoundEffects {
        rifle_shot: pitch_assets.add(Pitch::new(118.0, Duration::from_millis(75))),
        pistol_shot: pitch_assets.add(Pitch::new(170.0, Duration::from_millis(95))),
        footstep: pitch_assets.add(Pitch::new(64.0, Duration::from_millis(115))),
        zombie_groan: pitch_assets.add(Pitch::new(46.0, Duration::from_millis(620))),
    });
}
