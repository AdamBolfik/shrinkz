mod theme;

use bevy::prelude::*;
use shrinkz::{
    Axis, GameCommand, GameConfig, GameSession, GameSnapshot, Phase, Rect, Vec2 as SimVec2,
    WallView,
};
use theme::{ActiveTheme, ThemeId};

const PLAYFIELD_WIDTH: f32 = 800.0;
const PLAYFIELD_HEIGHT: f32 = 600.0;
const FIXED_HZ: f32 = 60.0;

fn main() {
    let active = ActiveTheme::default();
    let clear = active.palette().window_clear;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shrinkz".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(clear))
        .insert_resource(active)
        .insert_resource(PreferredAxis(Axis::Horizontal))
        .insert_resource(SessionResource(GameSession::new(default_config())))
        .insert_resource(SnapshotResource(
            GameSession::new(default_config()).snapshot(),
        ))
        .insert_resource(PendingCommand(None))
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(
            Update,
            (
                read_input,
                apply_theme_to_ui,
                update_axis_indicator,
                fixed_sim_step,
                sync_snapshot,
                draw_world,
                update_hud,
            )
                .chain(),
        )
        .run();
}

fn default_config() -> GameConfig {
    GameConfig {
        playfield: Rect::new(0.0, 0.0, PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT),
        ..GameConfig::default()
    }
}

#[derive(Resource)]
struct SessionResource(GameSession);

#[derive(Resource)]
struct SnapshotResource(GameSnapshot);

#[derive(Resource)]
struct PendingCommand(Option<GameCommand>);

#[derive(Resource, Clone, Copy)]
struct PreferredAxis(Axis);

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct HelpText;

#[derive(Component)]
struct ThemeLabel;

#[derive(Component)]
struct ThemeCycleButton;

#[derive(Component)]
struct AxisToggleButton;

#[derive(Component)]
struct AxisToggleLabel;

#[derive(Component)]
struct WorldRoot;

#[derive(Component)]
struct DrawnBall;

#[derive(Component)]
struct DrawnWall;

#[derive(Component)]
struct DrawnClaimed;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((WorldRoot, Transform::default(), Visibility::default()));
}

fn setup_ui(mut commands: Commands, theme: Res<ActiveTheme>) {
    let palette = theme.palette();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Shrinkz"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(palette.hud_text),
                HudText,
            ));

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Button,
                    AxisToggleButton,
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(palette.axis_horizontal),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Axis: H"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(palette.button_text),
                        AxisToggleLabel,
                    ));
                });

                row.spawn((
                    Button,
                    ThemeCycleButton,
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(palette.axis_vertical),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("Theme: {}", ThemeId::Midnight.display_name())),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(palette.button_text),
                        ThemeLabel,
                    ));
                });

                row.spawn((
                    Text::new(
                        "[LMB] primary  [RMB]/Shift] vertical  [T] theme  [P] pause  [R] restart  [N] new game",
                    ),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(palette.help_text),
                    HelpText,
                ));
            });
        });
}

fn shift_is_held(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

/// Effective wall axis for primary click: Shift forces vertical; otherwise preferred.
fn effective_axis(preferred: Axis, shift_held: bool) -> Axis {
    if shift_held {
        Axis::Vertical
    } else {
        preferred
    }
}

fn axis_label(axis: Axis, shift_held: bool) -> String {
    match axis {
        Axis::Horizontal => "Axis: H".into(),
        Axis::Vertical if shift_held => "Axis: V (Shift)".into(),
        Axis::Vertical => "Axis: V".into(),
    }
}

fn read_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut preferred: ResMut<PreferredAxis>,
    mut pending: ResMut<PendingCommand>,
    mut theme: ResMut<ActiveTheme>,
    session: Res<SessionResource>,
    axis_toggle_q: Query<&Interaction, (Changed<Interaction>, With<AxisToggleButton>)>,
    theme_toggle_q: Query<&Interaction, (Changed<Interaction>, With<ThemeCycleButton>)>,
) {
    for interaction in &axis_toggle_q {
        if *interaction == Interaction::Pressed {
            preferred.0 = match preferred.0 {
                Axis::Horizontal => Axis::Vertical,
                Axis::Vertical => Axis::Horizontal,
            };
        }
    }

    for interaction in &theme_toggle_q {
        if *interaction == Interaction::Pressed {
            theme.cycle();
        }
    }

    if keys.just_pressed(KeyCode::KeyT) {
        theme.cycle();
    }
    if keys.just_pressed(KeyCode::KeyP) {
        let snap = session.0.snapshot();
        pending.0 = Some(match snap.phase {
            Phase::Paused => GameCommand::Resume,
            _ => GameCommand::Pause,
        });
    }
    if keys.just_pressed(KeyCode::KeyR) {
        pending.0 = Some(GameCommand::RestartLevel);
    }
    if keys.just_pressed(KeyCode::KeyN) {
        pending.0 = Some(GameCommand::RestartGame);
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(world) = camera.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    let origin = screen_to_playfield(world);
    let shift = shift_is_held(&keys);

    if mouse.just_pressed(MouseButton::Left) {
        let axis = effective_axis(preferred.0, shift);
        pending.0 = Some(GameCommand::StartWall { origin, axis });
    } else if mouse.just_pressed(MouseButton::Right) {
        pending.0 = Some(GameCommand::StartWall {
            origin,
            axis: Axis::Vertical,
        });
    }
}

/// Push active theme colors into static UI chrome (clear color, HUD, help, theme button).
fn apply_theme_to_ui(
    theme: Res<ActiveTheme>,
    mut clear: ResMut<ClearColor>,
    // ParamSet: multiple &mut TextColor queries would conflict (B0001) without it.
    mut text_colors: ParamSet<(
        Query<&mut TextColor, With<HudText>>,
        Query<&mut TextColor, With<HelpText>>,
        Query<&mut TextColor, With<ThemeLabel>>,
        Query<&mut TextColor, With<AxisToggleLabel>>,
    )>,
    mut theme_label: Query<&mut Text, With<ThemeLabel>>,
    mut theme_btn_bg: Query<&mut BackgroundColor, With<ThemeCycleButton>>,
) {
    let palette = theme.palette();
    clear.0 = palette.window_clear;

    for mut color in &mut text_colors.p0() {
        color.0 = palette.hud_text;
    }
    for mut color in &mut text_colors.p1() {
        color.0 = palette.help_text;
    }
    for mut color in &mut text_colors.p2() {
        color.0 = palette.button_text;
    }
    for mut color in &mut text_colors.p3() {
        color.0 = palette.button_text;
    }

    let theme_name = format!("Theme: {}", theme.id.display_name());
    for mut text in &mut theme_label {
        if text.0 != theme_name {
            text.0 = theme_name.clone();
        }
    }
    for mut bg in &mut theme_btn_bg {
        *bg = BackgroundColor(palette.axis_vertical);
    }
}

/// Keep the bottom axis control in sync with preferred axis and Shift (vertical override).
fn update_axis_indicator(
    keys: Res<ButtonInput<KeyCode>>,
    preferred: Res<PreferredAxis>,
    theme: Res<ActiveTheme>,
    mut toggle_text: Query<&mut Text, With<AxisToggleLabel>>,
    mut toggle_bg: Query<&mut BackgroundColor, (With<AxisToggleButton>, Without<ThemeCycleButton>)>,
) {
    let palette = theme.palette();
    let shift = shift_is_held(&keys);
    let axis = effective_axis(preferred.0, shift);
    let label = axis_label(axis, shift);
    let color = if shift {
        palette.axis_shift
    } else {
        match preferred.0 {
            Axis::Horizontal => palette.axis_horizontal,
            Axis::Vertical => palette.axis_vertical,
        }
    };

    for mut text in &mut toggle_text {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
    for mut bg in &mut toggle_bg {
        *bg = BackgroundColor(color);
    }
}

fn screen_to_playfield(world: Vec2) -> SimVec2 {
    let local_x = world.x + PLAYFIELD_WIDTH * 0.5;
    let local_y = PLAYFIELD_HEIGHT * 0.5 - world.y;
    SimVec2::new(local_x, local_y)
}

fn playfield_to_world(p: SimVec2) -> Vec3 {
    let x = p.x - PLAYFIELD_WIDTH * 0.5;
    let y = PLAYFIELD_HEIGHT * 0.5 - p.y;
    Vec3::new(x, y, 0.0)
}

fn fixed_sim_step(
    time: Res<Time>,
    mut session: ResMut<SessionResource>,
    mut pending: ResMut<PendingCommand>,
    mut accumulator: Local<f32>,
) {
    *accumulator += time.delta_secs();
    let step = 1.0 / FIXED_HZ;
    let cmd = pending.0.take();
    let mut first = true;
    while *accumulator >= step {
        *accumulator -= step;
        let command = if first {
            first = false;
            cmd
        } else {
            None
        };
        session
            .0
            .apply(command, std::time::Duration::from_secs_f32(step));
    }
    if first {
        if let Some(command) = cmd {
            session
                .0
                .apply(Some(command), std::time::Duration::ZERO);
        }
    }
}

fn sync_snapshot(session: Res<SessionResource>, mut snapshot: ResMut<SnapshotResource>) {
    snapshot.0 = session.0.snapshot();
}

fn draw_world(
    mut commands: Commands,
    snapshot: Res<SnapshotResource>,
    theme: Res<ActiveTheme>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    root_q: Query<Entity, With<WorldRoot>>,
    drawn: Query<Entity, Or<(With<DrawnBall>, With<DrawnWall>, With<DrawnClaimed>)>>,
) {
    for entity in &drawn {
        commands.entity(entity).despawn();
    }

    let Ok(root) = root_q.single() else {
        return;
    };
    let snap = &snapshot.0;
    let palette = theme.palette();

    // Playfield painted as filled; free chambers carved out on top.
    let claimed_mat = materials.add(palette.claimed);
    let free_mat = materials.add(palette.free);
    let border_mat = materials.add(palette.playfield_border);

    let border = 4.0_f32;
    let border_mesh = meshes.add(Rectangle::new(
        PLAYFIELD_WIDTH + border * 2.0,
        PLAYFIELD_HEIGHT + border * 2.0,
    ));
    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Mesh2d(border_mesh),
            MeshMaterial2d(border_mat),
            Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
            DrawnClaimed,
        ));
    });

    let pf_mesh = meshes.add(Rectangle::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT));
    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Mesh2d(pf_mesh),
            MeshMaterial2d(claimed_mat),
            Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            DrawnClaimed,
        ));
    });

    for rect in &snap.free {
        let mesh = meshes.add(Rectangle::new(rect.width.max(0.5), rect.height.max(0.5)));
        let center = SimVec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
        let pos = playfield_to_world(center);
        commands.entity(root).with_children(|parent| {
            parent.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(free_mat.clone()),
                Transform::from_translation(pos),
                DrawnClaimed,
            ));
        });
    }

    let wall_mat = materials.add(palette.wall);
    let growing_mat = materials.add(palette.wall_growing);

    for wall in &snap.walls {
        spawn_wall_mesh(&mut commands, root, &mut meshes, wall_mat.clone(), wall, 1.0);
    }
    if let Some(wall) = &snap.wall_in_progress {
        spawn_wall_mesh(&mut commands, root, &mut meshes, growing_mat, wall, 1.5);
    }

    let ball_mat = materials.add(palette.ball);
    for ball in &snap.balls {
        let mesh = meshes.add(Circle::new(ball.radius));
        let pos = playfield_to_world(ball.position);
        commands.entity(root).with_children(|parent| {
            parent.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(ball_mat.clone()),
                Transform::from_translation(pos + Vec3::Z * 2.0),
                DrawnBall,
            ));
        });
    }
}

fn spawn_wall_mesh(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<ColorMaterial>,
    wall: &WallView,
    z: f32,
) {
    let (w, h, cx, cy) = match wall.axis {
        Axis::Horizontal => {
            let w = (wall.end - wall.start).max(0.5);
            let h = wall.thickness;
            let cx = (wall.start + wall.end) * 0.5;
            let cy = wall.fixed;
            (w, h, cx, cy)
        }
        Axis::Vertical => {
            let w = wall.thickness;
            let h = (wall.end - wall.start).max(0.5);
            let cx = wall.fixed;
            let cy = (wall.start + wall.end) * 0.5;
            (w, h, cx, cy)
        }
    };
    let mesh = meshes.add(Rectangle::new(w, h));
    let pos = playfield_to_world(SimVec2::new(cx, cy));
    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(pos + Vec3::Z * z),
            DrawnWall,
        ));
    });
}

fn update_hud(snapshot: Res<SnapshotResource>, mut hud: Query<&mut Text, With<HudText>>) {
    let snap = &snapshot.0;
    let phase = match snap.phase {
        Phase::Playing => "Playing",
        Phase::Paused => "Paused",
        Phase::LevelClear => "Level clear!",
        Phase::GameOver => "Game over",
    };
    let timer = snap
        .timer
        .as_ref()
        .map(|t| format!("  Time: {:.0}s", t.remaining.as_secs_f32()))
        .unwrap_or_default();
    let text = format!(
        "Shrinkz  |  {phase}  |  Level {}  |  Lives {}  |  Claimed {:.0}%  |  Score {}{}",
        snap.level,
        snap.lives,
        snap.claimed_ratio * 100.0,
        snap.score,
        timer
    );
    for mut hud_text in &mut hud {
        hud_text.0 = text.clone();
    }
}
