use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

mod game;
use game::*;

/// Represents the possible states of the game.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
}

/// Stores timing configuration for movement and gravity.
#[derive(Resource)]
struct GameConfig {
    move_timer: Timer,
    fall_timer: Timer,
}

/// Marker component for the currently falling tetromino.
#[derive(Component)]
struct ActiveBlock;

/// Marker component for blocks that have landed and are part of the grid.
#[derive(Component)]
struct LandedBlock;

/// Marker component for Game Over UI text.
#[derive(Component)]
struct GameOverText;

/// Marker component for Score UI text.
#[derive(Component)]
struct ScoreText;

/// Marker component for Next Piece UI text.
#[derive(Component)]
struct NextPieceText;

/// Marker component for the level UI text.
#[derive(Component)]
struct LevelText;

/// Marker component for the layer visualization container.
#[derive(Component)]
struct LayerVisualizer;

/// Stores handles to audio assets.
#[derive(Resource)]
struct AudioHandles {
    move_sound: Handle<AudioSource>,
    rotate_sound: Handle<AudioSource>,
    drop_sound: Handle<AudioSource>,
    clear_sound: Handle<AudioSource>,
    bgm: Handle<AudioSource>,
}

/// Cache for landed blocks rendering information.
#[derive(Resource, Default)]
struct LandedIndicators(Vec<(Transform, Color)>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D Tetris".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(GameGrid::new())
        .insert_resource(DirtyGrid(true)) // Start dirty to render initial floor
        .insert_resource(GameConfig {
            move_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            fall_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
        })
        .insert_resource(NextPiece(get_random_piece())) // Initialize next piece
        .insert_resource(GameStats::new())
        .init_resource::<LandedIndicators>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_tetromino,
                tetromino_movement,
                tetromino_render_active,
                gravity_system,
                check_layers,
                render_landed_grid,
                render_cached_indicators,
                render_boundaries,
                render_next_piece_preview,
                ui_system,
                update_layer_visualization,
                clear_dirty_flag,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, pause_input)
        .add_systems(OnEnter(GameState::GameOver), game_over_setup)
        .add_systems(
            Update,
            game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .run();
}

/// Resource storing the type of the next tetromino to be spawned.
#[derive(Resource)]
struct NextPiece(TetrominoType);

/// Returns a random TetrominoType.
fn get_random_piece() -> TetrominoType {
    let shapes = [
        TetrominoType::I,
        TetrominoType::O,
        TetrominoType::T,
        TetrominoType::Z,
        TetrominoType::L,
        TetrominoType::Tripod,
        TetrominoType::ScrewL,
        TetrominoType::ScrewR,
    ];
    let mut rng = rand::rng();
    shapes[rng.random_range(0..shapes.len())]
}

/// Renders a 3D preview of the next piece using gizmos.
fn render_next_piece_preview(mut gizmos: Gizmos, next_piece: Res<NextPiece>) {
    // Position the preview to the left side of the main grid below the score UI.
    let preview_pivot = Vec3::new(-10.0, 0.0, -2.0);
    let shapes = next_piece.0.get_shape_blocks();
    let color = next_piece.0.get_color();

    for pos in shapes {
        let global_pos = preview_pivot + Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
        gizmos.cuboid(Transform::from_translation(global_pos), color);
    }
}

/// Initializes the game world: camera, lights, audio, and UI.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let audio_handles = AudioHandles {
        move_sound: asset_server.load("sounds/move.ogg"),
        rotate_sound: asset_server.load("sounds/rotate.ogg"),
        drop_sound: asset_server.load("sounds/drop.ogg"),
        clear_sound: asset_server.load("sounds/clear.ogg"),
        bgm: asset_server.load("sounds/bgm.ogg"),
    };

    // Start background music
    commands.spawn((
        AudioPlayer::new(audio_handles.bgm.clone()),
        PlaybackSettings::LOOP,
    ));

    commands.insert_resource(audio_handles);

    let center_x = (GRID_WIDTH as f32 - 1.0) / 2.0;
    let center_z = (GRID_DEPTH as f32 - 1.0) / 2.0;

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(center_x, 25.0, center_z)
            .looking_at(Vec3::new(center_x, 0.0, center_z), Vec3::NEG_Z),
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 2000000.0, // Lumens, high for visibility
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        ..default()
    });

    // Grid Floor Visualization (Static)
    let grid_color = Color::srgb(0.2, 0.2, 0.2);
    for x in 0..GRID_WIDTH {
        for z in 0..GRID_DEPTH {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.9, 0.1, 0.9))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: grid_color,
                    ..default()
                })),
                Transform::from_xyz(x as f32, -0.5, z as f32),
            ));
        }
    }

    // UI Setup
    commands.spawn((
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));

    commands.spawn((
        Text::new("Next: "),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        NextPieceText,
    ));

    commands.spawn((
        Text::new("Level: 1"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            left: Val::Px(10.0),
            ..default()
        },
        LevelText,
    ));

    // Initial instruction
    commands.spawn((
        Text::new("WASD/Arrows: Move | Q/E/R: Rotate (+Shift: Reverse) | Space: Drop | P: Pause"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Layer Visualization Container
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            width: Val::Percent(25.0),
            height: Val::Percent(90.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        LayerVisualizer,
    ));
}

/// Spawns a new tetromino at the top of the grid.
fn spawn_tetromino(
    mut commands: Commands,
    query: Query<&ActiveBlock>,
    mut next_piece: ResMut<NextPiece>,
    game_grid: Res<GameGrid>,
    mut next_piece_state: ResMut<NextState<GameState>>,
) {
    if query.iter().next().is_some() {
        return;
    }

    let piece_type = next_piece.0;
    next_piece.0 = get_random_piece();

    let start_pos = IVec3::new(GRID_WIDTH / 2, GRID_HEIGHT, GRID_DEPTH / 2);
    let shapes = piece_type.get_shape_blocks();

    // Check game over
    if !can_place(&shapes, start_pos, &game_grid) {
        next_piece_state.set(GameState::GameOver);
        return;
    }

    commands.spawn((
        Tetromino {
            piece_type,
            positions: shapes,
            pivot: start_pos,
            color: piece_type.get_color(),
        },
        ActiveBlock,
    ));
}
/// Renders the currently active (falling) tetromino using wireframes.
fn tetromino_render_active(mut gizmos: Gizmos, query: Query<&Tetromino, With<ActiveBlock>>) {
    for tetromino in &query {
        for pos in &tetromino.positions {
            let global_pos = tetromino.pivot + *pos;
            // Draw wireframe box
            gizmos.cuboid(
                Transform::from_translation(Vec3::new(
                    global_pos.x as f32,
                    global_pos.y as f32,
                    global_pos.z as f32,
                )),
                Color::WHITE,
            );
            // Make it look "wired" by adding an inner cross or something if needed, but cuboid edges is what "wireframe" usually means.
            // To make it distinct, we can use the tetromino color for the wire.
            gizmos.cuboid(
                Transform::from_translation(Vec3::new(
                    global_pos.x as f32,
                    global_pos.y as f32,
                    global_pos.z as f32,
                ))
                .with_scale(Vec3::splat(0.95)),
                tetromino.color,
            );
        }
    }
}

/// Renders the grid cage boundaries.
fn render_boundaries(mut gizmos: Gizmos) {
    let w = GRID_WIDTH as f32;
    let h = GRID_HEIGHT as f32;
    let d = GRID_DEPTH as f32;

    let grid_color = Color::srgb(0.3, 0.3, 0.3);
    let border_color = Color::srgb(0.6, 0.6, 0.6);

    // Vertical lines at grid intersections on the boundaries
    for x in 0..=GRID_WIDTH {
        let x_f = x as f32 - 0.5;
        // Front and back walls
        gizmos.line(
            Vec3::new(x_f, -0.5, -0.5),
            Vec3::new(x_f, h - 0.5, -0.5),
            grid_color,
        );
        gizmos.line(
            Vec3::new(x_f, -0.5, d - 0.5),
            Vec3::new(x_f, h - 0.5, d - 0.5),
            grid_color,
        );
    }
    for z in 0..=GRID_DEPTH {
        let z_f = z as f32 - 0.5;
        // Left and right walls
        gizmos.line(
            Vec3::new(-0.5, -0.5, z_f),
            Vec3::new(-0.5, h - 0.5, z_f),
            grid_color,
        );
        gizmos.line(
            Vec3::new(w - 0.5, -0.5, z_f),
            Vec3::new(w - 0.5, h - 0.5, z_f),
            grid_color,
        );
    }

    // Horizontal rings at each y level
    for y in 0..=GRID_HEIGHT {
        let y_f = y as f32 - 0.5;
        let color = if y % 5 == 0 { border_color } else { grid_color };

        gizmos.line(
            Vec3::new(-0.5, y_f, -0.5),
            Vec3::new(w - 0.5, y_f, -0.5),
            color,
        );
        gizmos.line(
            Vec3::new(w - 0.5, y_f, -0.5),
            Vec3::new(w - 0.5, y_f, d - 0.5),
            color,
        );
        gizmos.line(
            Vec3::new(w - 0.5, y_f, d - 0.5),
            Vec3::new(-0.5, y_f, d - 0.5),
            color,
        );
        gizmos.line(
            Vec3::new(-0.5, y_f, d - 0.5),
            Vec3::new(-0.5, y_f, -0.5),
            color,
        );
    }
}

/// Handles user input for horizontal/depth movement and rotation.
fn tetromino_movement(
    mut query: Query<&mut Tetromino, With<ActiveBlock>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_grid: Res<GameGrid>,
    time: Res<Time>,
    mut config: ResMut<GameConfig>,
    audio: Res<AudioHandles>,
    mut commands: Commands,
) {
    if let Some(mut tetromino) = query.iter_mut().next() {
        config.move_timer.tick(time.delta());

        if config.move_timer.is_finished() {
            let mut move_delta = IVec3::ZERO;
            if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
                move_delta.x -= 1;
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD)
            {
                move_delta.x += 1;
            }
            if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
                move_delta.z -= 1;
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
                move_delta.z += 1;
            }

            // Movement
            if move_delta != IVec3::ZERO {
                if can_place(
                    &tetromino.positions,
                    tetromino.pivot + move_delta,
                    &game_grid,
                ) {
                    tetromino.pivot += move_delta;
                    commands.spawn((
                        AudioPlayer::new(audio.move_sound.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }
        }

        let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let forward = !shift;

        // Rotation (Immediate, not timer based for responsiveness)
        if keyboard_input.just_pressed(KeyCode::KeyQ) {
            // Rotate Y
            if try_rotate_with_kicks(&mut tetromino, IVec3::Y, forward, &game_grid) {
                commands.spawn((
                    AudioPlayer::new(audio.rotate_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
        if keyboard_input.just_pressed(KeyCode::KeyE) {
            // Rotate X
            if try_rotate_with_kicks(&mut tetromino, IVec3::X, forward, &game_grid) {
                commands.spawn((
                    AudioPlayer::new(audio.rotate_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
        if keyboard_input.just_pressed(KeyCode::KeyR) {
            // Rotate Z
            if try_rotate_with_kicks(&mut tetromino, IVec3::Z, forward, &game_grid) {
                commands.spawn((
                    AudioPlayer::new(audio.rotate_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }

        // Hard Drop
        if keyboard_input.just_pressed(KeyCode::Space) {
            let new_pivot = calculate_hard_drop(&tetromino, &game_grid);
            if new_pivot != tetromino.pivot {
                tetromino.pivot = new_pivot;
                commands.spawn((
                    AudioPlayer::new(audio.drop_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
            // Lock immediately
            config.fall_timer.reset();
        }
    }
}

/// Applies gravity to the active tetromino and locks it if it hits an obstacle.
fn gravity_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Tetromino), With<ActiveBlock>>,
    mut game_grid: ResMut<GameGrid>,
    mut dirty: ResMut<DirtyGrid>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut config: ResMut<GameConfig>,
) {
    config.fall_timer.tick(time.delta());
    if !config.fall_timer.just_finished() {
        return;
    }

    if let Some((entity, mut tetromino)) = query.iter_mut().next() {
        let next_y_pivot = tetromino.pivot + IVec3::new(0, -1, 0);
        if can_place(&tetromino.positions, next_y_pivot, &game_grid) {
            tetromino.pivot.y -= 1;
        } else {
            // Lock
            if game_grid.lock_tetromino(&tetromino, &mut dirty) {
                next_state.set(GameState::GameOver);
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Renders the landed blocks in the grid. Uses level-based coloring for depth perception.
fn render_landed_grid(
    mut commands: Commands,
    game_grid: Res<GameGrid>,
    dirty: Res<DirtyGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut indicators: ResMut<LandedIndicators>,
    query: Query<Entity, With<LandedBlock>>,
) {
    if !dirty.0 {
        return;
    }
    for entity in &query {
        commands.entity(entity).despawn();
    }

    // Clear indicators cache
    indicators.0.clear();

    // Redraw
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            for z in 0..GRID_DEPTH {
                if let Some(piece_type) = game_grid.get(x, y, z) {
                    // 1. Solid Level Color Block
                    let level_color = Color::hsl((y as f32 * 40.0) % 360.0, 0.9, 0.5);
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: level_color,
                            ..default()
                        })),
                        Transform::from_xyz(x as f32, y as f32, z as f32),
                        LandedBlock,
                    ));
                    let piece_color = piece_type.get_color();
                    indicators.0.push((
                        Transform::from_xyz(x as f32, y as f32, z as f32)
                            .with_scale(Vec3::splat(1.01)),
                        piece_color,
                    ));
                }
            }
        }
    }
}

/// Renders tetromino piece type indicators on landed blocks using gizmos.
fn render_cached_indicators(mut gizmos: Gizmos, indicators: Res<LandedIndicators>) {
    for (transform, color) in &indicators.0 {
        gizmos.cuboid(*transform, *color);
    }
}

/// Checks for full horizontal layers and clears them. Updates score and level.
fn check_layers(
    mut game_grid: ResMut<GameGrid>,
    mut dirty: ResMut<DirtyGrid>,
    mut stats: ResMut<GameStats>,
    mut config: ResMut<GameConfig>,
    audio: Res<AudioHandles>,
    mut commands: Commands,
) {
    let layers_cleared = game_grid.clear_full_layers(&mut dirty);
    if layers_cleared > 0 {
        if stats.add_layers(layers_cleared) {
            let new_speed = stats.get_fall_speed();
            config
                .fall_timer
                .set_duration(Duration::from_secs_f32(new_speed));
            println!("Level Up! Level: {}, Speed: {:.1}s", stats.level, new_speed);
        }
        commands.spawn((
            AudioPlayer::new(audio.clear_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// Handles pause input.
fn pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

/// Updates UI text for score, next piece, and level.
fn ui_system(
    stats: Res<GameStats>,
    next: Res<NextPiece>,
    mut score_query: Query<
        &mut Text,
        (With<ScoreText>, Without<NextPieceText>, Without<LevelText>),
    >,
    mut next_query: Query<&mut Text, (With<NextPieceText>, Without<LevelText>)>,
    mut level_query: Query<&mut Text, With<LevelText>>,
) {
    for mut text in &mut score_query {
        text.0 = format!("Score: {}", stats.score);
    }
    for mut text in &mut next_query {
        text.0 = format!("Next: {:?}", next.0);
    }
    for mut text in &mut level_query {
        text.0 = format!("Level: {}", stats.level);
    }
}

/// Spawns Game Over UI.
fn game_over_setup(mut commands: Commands) {
    commands.spawn((
        Text::new("GAME OVER\nPress R to Restart"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(40.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        GameOverText,
    ));
}

/// Handles input during the Game Over state to restart the game.
fn game_over_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut game_grid: ResMut<GameGrid>,
    mut dirty: ResMut<DirtyGrid>,
    mut stats: ResMut<GameStats>,
    mut config: ResMut<GameConfig>,
    active_query: Query<Entity, With<ActiveBlock>>,
    landed_query: Query<Entity, With<LandedBlock>>,
    text_query: Query<Entity, With<GameOverText>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset Game
        *game_grid = GameGrid::new();
        *stats = GameStats::new();
        dirty.0 = true;
        config.fall_timer.set_duration(Duration::from_secs_f32(0.8));
        for entity in &active_query {
            commands.entity(entity).despawn();
        }
        for entity in &landed_query {
            commands.entity(entity).despawn();
        }
        for entity in &text_query {
            commands.entity(entity).despawn();
        }

        next_state.set(GameState::Playing);
    }
}

/// System to clear the dirty flag at the end of the frame.
fn clear_dirty_flag(mut dirty: ResMut<DirtyGrid>) {
    dirty.0 = false;
}

/// Updates the 2D layer visualization panel on the right side of the screen.
/// Redraws 2D representations for Layer 0 and any other layers containing locked blocks.
fn update_layer_visualization(
    mut commands: Commands,
    game_grid: Res<GameGrid>,
    dirty: Res<DirtyGrid>,
    query: Query<(Entity, Option<&Children>), With<LayerVisualizer>>,
) {
    if !dirty.0 {
        return;
    }
    if let Some((container, children)) = query.iter().next() {
        if let Some(children) = children {
            for &child in children {
                commands.entity(child).despawn();
            }
        }
        for y in 0..GRID_HEIGHT {
            // Always show Layer 0, others only if they have locked cells
            if y == 0 || !game_grid.is_layer_empty(y) {
                commands.entity(container).with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(100.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                flex_direction: FlexDirection::Column,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                        ))
                        .with_children(|grid_parent| {
                            grid_parent.spawn((
                                Text::new(format!("Layer {}", y)),
                                Node {
                                    height: Val::Px(15.0),
                                    ..default()
                                },
                            ));
                            grid_parent
                                .spawn(Node {
                                    display: Display::Grid,
                                    grid_template_columns: RepeatedGridTrack::px(8, 10.0),
                                    grid_template_rows: RepeatedGridTrack::px(8, 10.0),
                                    ..default()
                                })
                                .with_children(|cells_parent| {
                                    for z in 0..GRID_DEPTH {
                                        for x in 0..GRID_WIDTH {
                                            let color = if let Some(pt) = game_grid.get(x, y, z) {
                                                pt.get_color()
                                            } else {
                                                Color::NONE
                                            };
                                            cells_parent.spawn((
                                                Node {
                                                    width: Val::Px(8.0),
                                                    height: Val::Px(8.0),
                                                    border: UiRect::all(Val::Px(0.5)),
                                                    ..default()
                                                },
                                                BackgroundColor(color),
                                                BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
                                            ));
                                        }
                                    }
                                });
                        });
                });
            }
        }
    }
}
