use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameMode>()
            .init_resource::<Score>()
            .add_systems(Startup, setup_ui_camera)
            .add_systems(OnEnter(GameMode::Menu), (show_menu_cursor, spawn_menu))
            .add_systems(
                OnEnter(GameMode::Playing),
                (reset_score, lock_play_cursor, spawn_hud),
            )
            .add_systems(
                Update,
                (menu_button_colors, start_game_from_menu).run_if(in_state(GameMode::Menu)),
            )
            .add_systems(Update, update_hud_score.run_if(in_state(GameMode::Playing)));
    }
}

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameMode {
    #[default]
    Menu,
    Playing,
}

#[derive(Resource, Default)]
pub struct Score {
    pub kills: u32,
    pub points: u32,
}

#[derive(Component, Clone, Copy)]
pub struct ScoreValue {
    pub points: u32,
}

impl ScoreValue {
    pub const fn new(points: u32) -> Self {
        Self { points }
    }
}

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct KillText;

const PANEL: Color = Color::srgba(0.025, 0.022, 0.017, 0.74);
const BUTTON: Color = Color::srgb(0.62, 0.47, 0.27);
const BUTTON_HOVER: Color = Color::srgb(0.76, 0.59, 0.35);
const BUTTON_PRESS: Color = Color::srgb(0.44, 0.32, 0.18);
const TEXT: Color = Color::srgb(0.95, 0.90, 0.80);

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));
}

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(GameMode::Menu),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(20),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.018, 0.014, 0.58)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Market Sweep"),
                TextFont {
                    font_size: 58.0,
                    ..default()
                },
                TextColor(TEXT),
            ));
            root.spawn((
                Text::new("Clear the old town before the horde owns the rooftops."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.76, 0.64)),
            ));
            root.spawn((
                Button,
                Node {
                    width: px(240),
                    height: px(58),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::top(px(12)),
                    ..default()
                },
                BackgroundColor(BUTTON),
                StartButton,
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("Start"),
                    TextFont {
                        font_size: 27.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.08, 0.06, 0.04)),
                ));
            });
        });
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(GameMode::Playing),
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                left: px(16),
                min_width: px(190),
                padding: UiRect::axes(px(14), px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .with_children(|hud| {
            hud.spawn((
                Text::new("Score 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT),
                ScoreText,
            ));
            hud.spawn((
                Text::new("Kills 0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.80, 0.74, 0.62)),
                KillText,
            ));
        });
}

fn menu_button_colors(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in &mut buttons {
        *color = match *interaction {
            Interaction::Pressed => BUTTON_PRESS.into(),
            Interaction::Hovered => BUTTON_HOVER.into(),
            Interaction::None => BUTTON.into(),
        };
    }
}

fn start_game_from_menu(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    let pressed_button = buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if pressed_button || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(GameMode::Playing);
    }
}

fn update_hud_score(
    score: Res<Score>,
    mut score_texts: Query<&mut Text, With<ScoreText>>,
    mut kill_texts: Query<&mut Text, (With<KillText>, Without<ScoreText>)>,
) {
    if !score.is_changed() {
        return;
    }

    for mut text in &mut score_texts {
        text.0 = format!("Score {}", score.points);
    }
    for mut text in &mut kill_texts {
        text.0 = format!("Kills {}", score.kills);
    }
}

fn reset_score(mut score: ResMut<Score>) {
    *score = Score::default();
}

fn show_menu_cursor(mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    let Ok(mut cursor_options) = cursor_options.single_mut() else {
        return;
    };

    cursor_options.visible = true;
    cursor_options.grab_mode = CursorGrabMode::None;
}

fn lock_play_cursor(mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    let Ok(mut cursor_options) = cursor_options.single_mut() else {
        return;
    };

    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}
