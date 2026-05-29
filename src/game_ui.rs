use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy::{app::AppExit, prelude::*};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameMode>()
            .init_resource::<Score>()
            .init_resource::<SelectedMap>()
            .init_resource::<PauseState>()
            .add_systems(Startup, setup_ui_camera)
            .add_systems(OnEnter(GameMode::Menu), (show_menu_cursor, spawn_menu))
            .add_systems(
                OnEnter(GameMode::Playing),
                (reset_score, clear_pause_state, lock_play_cursor, spawn_hud),
            )
            .add_systems(
                Update,
                (update_button_colors, handle_main_menu_actions).run_if(in_state(GameMode::Menu)),
            )
            .add_systems(
                Update,
                (
                    update_hud_score,
                    toggle_pause_menu,
                    handle_pause_menu_actions,
                    update_button_colors,
                )
                    .run_if(in_state(GameMode::Playing)),
            );
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

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

pub fn gameplay_unpaused(pause: Res<PauseState>) -> bool {
    !pause.paused
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedMap {
    pub kind: MapKind,
}

impl Default for SelectedMap {
    fn default() -> Self {
        Self {
            kind: MapKind::Desert,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapKind {
    Desert,
    Forest,
    Night,
}

impl MapKind {
    const ALL: [Self; 3] = [Self::Desert, Self::Forest, Self::Night];

    fn title(self) -> &'static str {
        match self {
            Self::Desert => "Desert Market",
            Self::Forest => "Greenwood",
            Self::Night => "Night Quarter",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Desert => "Clean routes, sun-baked walls, sparse cover",
            Self::Forest => "Open clearings, trees, ruins, soft green light",
            Self::Night => "Moonlit streets, lamps, darker sightlines",
        }
    }
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

#[derive(Component, Clone, Copy)]
enum UiAction {
    Start,
    SelectMap(MapKind),
    Resume,
    MainMenu,
    Quit,
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct KillText;

#[derive(Component)]
struct PauseRoot;

const PANEL: Color = Color::srgba(0.025, 0.022, 0.017, 0.74);
const BUTTON: Color = Color::srgb(0.56, 0.43, 0.27);
const BUTTON_HOVER: Color = Color::srgb(0.72, 0.57, 0.36);
const BUTTON_PRESS: Color = Color::srgb(0.39, 0.28, 0.17);
const BUTTON_SELECTED: Color = Color::srgb(0.38, 0.58, 0.42);
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
                row_gap: px(18),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.018, 0.014, 0.62)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Market Sweep"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(TEXT),
            ));
            root.spawn((
                Text::new("Choose a map, then clear the infected."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.76, 0.64)),
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: px(8),
                    width: px(420),
                    margin: UiRect::top(px(8)),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|panel| {
                for kind in MapKind::ALL {
                    panel
                        .spawn((
                            Button,
                            Node {
                                min_height: px(58),
                                padding: UiRect::axes(px(14), px(8)),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                row_gap: px(2),
                                ..default()
                            },
                            BackgroundColor(BUTTON),
                            UiAction::SelectMap(kind),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(kind.title()),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.08, 0.06, 0.04)),
                            ));
                            button.spawn((
                                Text::new(kind.subtitle()),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.15, 0.11, 0.07)),
                            ));
                        });
                }
            });
            spawn_text_button(root, "Start", UiAction::Start, 240.0, 58.0, 27.0);
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

fn spawn_pause_menu(commands: &mut Commands) {
    commands
        .spawn((
            DespawnOnExit(GameMode::Playing),
            PauseRoot,
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(14),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.018, 0.014, 0.66)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 46.0,
                    ..default()
                },
                TextColor(TEXT),
            ));
            spawn_text_button(root, "Return", UiAction::Resume, 260.0, 54.0, 24.0);
            spawn_text_button(root, "Main Menu", UiAction::MainMenu, 260.0, 54.0, 24.0);
            spawn_text_button(root, "Close Game", UiAction::Quit, 260.0, 54.0, 24.0);
        });
}

fn spawn_text_button(
    parent: &mut ChildSpawnerCommands,
    label: &'static str,
    action: UiAction,
    width: f32,
    height: f32,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: px(width),
                height: px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::top(px(8)),
                ..default()
            },
            BackgroundColor(BUTTON),
            action,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(Color::srgb(0.08, 0.06, 0.04)),
            ));
        });
}

fn update_button_colors(
    selected_map: Res<SelectedMap>,
    mut buttons: Query<(&Interaction, &UiAction, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, action, mut color) in &mut buttons {
        let base = match action {
            UiAction::SelectMap(kind) if *kind == selected_map.kind => BUTTON_SELECTED,
            _ => BUTTON,
        };

        *color = match *interaction {
            Interaction::Pressed => BUTTON_PRESS.into(),
            Interaction::Hovered => BUTTON_HOVER.into(),
            Interaction::None => base.into(),
        };
    }
}

fn handle_main_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
    interactions: Query<(&Interaction, &UiAction), (Changed<Interaction>, With<Button>)>,
    mut selected_map: ResMut<SelectedMap>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    let mut should_start = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);

    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            UiAction::Start => should_start = true,
            UiAction::SelectMap(kind) => selected_map.kind = *kind,
            UiAction::Resume | UiAction::MainMenu | UiAction::Quit => {}
        }
    }

    if should_start {
        next_state.set(GameMode::Playing);
    }
}

fn toggle_pause_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut pause: ResMut<PauseState>,
    roots: Query<Entity, With<PauseRoot>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if pause.paused {
        pause.paused = false;
        despawn_pause_menu(&mut commands, &roots);
        set_cursor_locked(&mut cursor_options);
    } else {
        pause.paused = true;
        set_cursor_visible(&mut cursor_options);
        spawn_pause_menu(&mut commands);
    }
}

fn handle_pause_menu_actions(
    mut commands: Commands,
    interactions: Query<(&Interaction, &UiAction), (Changed<Interaction>, With<Button>)>,
    mut pause: ResMut<PauseState>,
    roots: Query<Entity, With<PauseRoot>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut app_exit_writer: MessageWriter<AppExit>,
) {
    if !pause.paused {
        return;
    }

    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            UiAction::Resume => {
                pause.paused = false;
                despawn_pause_menu(&mut commands, &roots);
                set_cursor_locked(&mut cursor_options);
            }
            UiAction::MainMenu => {
                pause.paused = false;
                next_state.set(GameMode::Menu);
            }
            UiAction::Quit => {
                app_exit_writer.write(AppExit::Success);
            }
            UiAction::Start | UiAction::SelectMap(_) => {}
        }
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

fn clear_pause_state(mut pause: ResMut<PauseState>) {
    pause.paused = false;
}

fn show_menu_cursor(mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_visible(&mut cursor_options);
}

fn lock_play_cursor(mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_locked(&mut cursor_options);
}

fn set_cursor_visible(cursor_options: &mut Query<&mut CursorOptions, With<PrimaryWindow>>) {
    let Ok(mut cursor_options) = cursor_options.single_mut() else {
        return;
    };

    cursor_options.visible = true;
    cursor_options.grab_mode = CursorGrabMode::None;
}

fn set_cursor_locked(cursor_options: &mut Query<&mut CursorOptions, With<PrimaryWindow>>) {
    let Ok(mut cursor_options) = cursor_options.single_mut() else {
        return;
    };

    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn despawn_pause_menu(commands: &mut Commands, roots: &Query<Entity, With<PauseRoot>>) {
    for entity in roots {
        commands.entity(entity).despawn();
    }
}
