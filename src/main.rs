use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;
use rand::Rng;

mod game;
use game::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
}

#[derive(Resource)]
struct GameConfig {
    move_timer: Timer,
    fall_timer: Timer,
}

#[derive(Component)]
struct ActiveBlock;

// Marker for landed blocks meshes
#[derive(Component)]
struct LandedBlock(IVec3);

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct NextPieceText;

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
        .insert_resource(GameConfig {
            move_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            fall_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
        })
        .insert_resource(NextPiece(get_random_piece())) // Initialize next piece
        .insert_resource(Score(0))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            spawn_tetromino,
            tetromino_movement,
            tetromino_render_active,
            gravity_system.run_if(on_timer(Duration::from_secs_f32(0.8))), 
            check_lines,
            render_landed_blocks,
            render_boundaries,
            render_next_piece_preview,
            ui_system,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, pause_input)
        .add_systems(OnEnter(GameState::GameOver), game_over_setup)
        .add_systems(Update, game_over_input.run_if(in_state(GameState::GameOver)))
        .run();
}

#[derive(Resource)]
struct NextPiece(TetrominoType);

#[derive(Resource)]
struct Score(u32);

fn get_random_piece() -> TetrominoType {
    let shapes = [
        TetrominoType::I, TetrominoType::O, TetrominoType::T, 
        TetrominoType::S, TetrominoType::Z, TetrominoType::J, TetrominoType::L
    ];
    let mut rng = rand::rng(); 
    shapes[rng.random_range(0..shapes.len())]
}

fn render_next_piece_preview(
    mut gizmos: Gizmos,
    next_piece: Res<NextPiece>,
) {
    let preview_pivot = Vec3::new(8.0, 10.0, 2.5);
    let shapes = get_shape_blocks(next_piece.0);
    let color = get_random_color(next_piece.0);

    for pos in shapes {
        let global_pos = preview_pivot + Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
        gizmos.cuboid(
            Transform::from_translation(global_pos).with_scale(Vec3::splat(0.8)),
            color,
        );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera - "Falling Away" view
    // Grid center is approx (2.5, 7.5, 2.5).
    // We want to be "above" the top (y=15) and looking down.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.5, 22.0, 12.0).looking_at(Vec3::new(2.5, 4.0, 2.5), Vec3::Y),
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
}
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
    
    println!("Spawning: {:?}, Next: {:?}", piece_type, next_piece.0);

    let start_pos = IVec3::new(GRID_WIDTH / 2, GRID_HEIGHT, GRID_DEPTH / 2);
    let shapes = get_shape_blocks(piece_type);
    
    // Check game over
    for block in &shapes {
        let check_pos = start_pos + *block;
        // If spawn position is occupied (checking below the spawn point mainly)
        if game_grid.is_occupied(check_pos.x, check_pos.y, check_pos.z) && check_pos.y < GRID_HEIGHT {
             next_piece_state.set(GameState::GameOver);
             return;
        }
    }

    commands.spawn(( 
        Tetromino {
            shape: piece_type,
            positions: shapes,
            pivot: start_pos,
            color: get_random_color(piece_type),
        },
        ActiveBlock,
    ));
}

fn tetromino_render_active(
    mut gizmos: Gizmos,
    query: Query<&Tetromino, With<ActiveBlock>>,
) {
    for tetromino in &query {
        for pos in &tetromino.positions {
            let global_pos = tetromino.pivot + *pos;
            // Draw wireframe box
            gizmos.cuboid(
                Transform::from_translation(Vec3::new(global_pos.x as f32, global_pos.y as f32, global_pos.z as f32)),
                Color::WHITE,
            );
             // Make it look "wired" by adding an inner cross or something if needed, but cuboid edges is what "wireframe" usually means.
             // To make it distinct, we can use the tetromino color for the wire.
             gizmos.cuboid(
                Transform::from_translation(Vec3::new(global_pos.x as f32, global_pos.y as f32, global_pos.z as f32)).with_scale(Vec3::splat(0.95)),
                tetromino.color,
            );
        }
    }
}

fn rotate_point(point: IVec3, axis: IVec3, forward: bool) -> IVec3 {
    // 90 degree rotation
    // Forward: +90 deg. Inverse: -90 deg.
    if axis.x == 1 {
        // X Axis: (x, y, z) -> (x, -z, y)
        if forward { IVec3::new(point.x, -point.z, point.y) }
        else { IVec3::new(point.x, point.z, -point.y) }
    } else if axis.y == 1 {
        // Y Axis: (x, y, z) -> (z, y, -x)
        if forward { IVec3::new(point.z, point.y, -point.x) }
        else { IVec3::new(-point.z, point.y, point.x) }
    } else { // z
        // Z Axis: (x, y, z) -> (-y, x, z)
        if forward { IVec3::new(-point.y, point.x, point.z) }
        else { IVec3::new(point.y, -point.x, point.z) }
    }
}

fn render_boundaries(mut gizmos: Gizmos) {
    // Draw the bounds of the game grid (0..5, 0..15, 0..5)
    // Center is (2.5, 7.5, 2.5)
    // Size is (5.0, 15.0, 5.0)
    gizmos.cuboid(
        Transform::from_xyz(2.5, 7.5, 2.5).with_scale(Vec3::new(5.0, 15.0, 5.0)),
        Color::srgb(0.5, 0.5, 0.5),
    );
}

fn try_rotate_with_kicks(
    tetromino: &mut Tetromino, 
    axis: IVec3, 
    forward: bool, 
    grid: &GameGrid
) {
    let new_positions: Vec<IVec3> = tetromino.positions.iter()
        .map(|p| rotate_point(*p, axis, forward))
        .collect();

    // Kick offsets to try: (0,0,0) then simple nudges
    let kicks = [
        IVec3::ZERO,
        IVec3::new(0, 1, 0), // Up (Floor kick)
        IVec3::new(0, 2, 0), // Up 2
        IVec3::new(1, 0, 0), 
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ];

    for kick in kicks {
        let test_pivot = tetromino.pivot + kick;
        if is_valid_rotation(&new_positions, test_pivot, grid) {
            tetromino.positions = new_positions;
            tetromino.pivot = test_pivot;
            return;
        }
    }
}

fn tetromino_movement(
    mut query: Query<&mut Tetromino, With<ActiveBlock>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_grid: Res<GameGrid>,
    time: Res<Time>,
    mut config: ResMut<GameConfig>,
) {
    // FIX: Use iter_mut().next() instead of get_single_mut()
    if let Some(mut tetromino) = query.iter_mut().next() {
        config.move_timer.tick(time.delta());
        
        if config.move_timer.is_finished() {
            let mut move_delta = IVec3::ZERO;
            if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
                move_delta.x -= 1;
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
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
                let mut valid = true;
                for pos in &tetromino.positions {
                    let new_pos = tetromino.pivot + *pos + move_delta;
                    if !game_grid.is_valid_pos(new_pos.x, new_pos.y, new_pos.z) || game_grid.is_occupied(new_pos.x, new_pos.y, new_pos.z) {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    tetromino.pivot += move_delta;
                }
            }
        }
        
        let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let forward = !shift;

        // Rotation (Immediate, not timer based for responsiveness)
        if keyboard_input.just_pressed(KeyCode::KeyQ) {
            // Rotate Y
            try_rotate_with_kicks(&mut tetromino, IVec3::Y, forward, &game_grid);
        }
        if keyboard_input.just_pressed(KeyCode::KeyE) {
             // Rotate X
             try_rotate_with_kicks(&mut tetromino, IVec3::X, forward, &game_grid);
        }
         if keyboard_input.just_pressed(KeyCode::KeyR) {
             // Rotate Z
             try_rotate_with_kicks(&mut tetromino, IVec3::Z, forward, &game_grid);
        }

        // Hard Drop
        if keyboard_input.just_pressed(KeyCode::Space) {
            loop {
                let mut valid = true;
                for pos in &tetromino.positions {
                    let new_pos = tetromino.pivot + *pos + IVec3::new(0, -1, 0);
                    if new_pos.y < 0 || game_grid.is_occupied(new_pos.x, new_pos.y, new_pos.z) {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    tetromino.pivot.y -= 1;
                } else {
                    break;
                }
            }
            // Lock immediately
            config.fall_timer.reset(); 
        }
    }
}

fn is_valid_rotation(positions: &Vec<IVec3>, pivot: IVec3, grid: &GameGrid) -> bool {
    for pos in positions {
        let global = pivot + *pos;
        if !grid.is_valid_pos(global.x, global.y, global.z) || grid.is_occupied(global.x, global.y, global.z) {
            return false;
        }
    }
    true
}

fn gravity_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Tetromino), With<ActiveBlock>>,
    mut game_grid: ResMut<GameGrid>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some((entity, mut tetromino)) = query.iter_mut().next() {
        let mut valid = true;
        for pos in &tetromino.positions {
            let new_pos = tetromino.pivot + *pos + IVec3::new(0, -1, 0);
            if new_pos.y < 0 || game_grid.is_occupied(new_pos.x, new_pos.y, new_pos.z) {
                valid = false;
                break;
            }
        }

        if valid {
            tetromino.pivot.y -= 1;
        } else {
            // Lock
            let mut game_over = false;
            for pos in &tetromino.positions {
                let global = tetromino.pivot + *pos;
                // If any block locks above the grid, it's game over
                if global.y >= GRID_HEIGHT {
                    game_over = true;
                }
                game_grid.set(global.x, global.y, global.z, tetromino.color);
            }
            commands.entity(entity).despawn();
            
            if game_over {
                next_state.set(GameState::GameOver);
            }
        }
    }
}

fn render_landed_blocks(
    mut commands: Commands,
    game_grid: Res<GameGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, With<LandedBlock>>, 
) {
    // Despawn all landed blocks to redraw
    for entity in &query {
        commands.entity(entity).despawn();
    }

    // Redraw
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            for z in 0..GRID_DEPTH {
                if let Some(color) = game_grid.get(x, y, z) {
                    // Shade by height (visual flair)
                    let _shade_factor = 0.5 + (y as f32 / GRID_HEIGHT as f32) * 0.5;
                    
                    commands.spawn(( 
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: color, 
                            ..default()
                        })),
                        Transform::from_xyz(x as f32, y as f32, z as f32),
                        LandedBlock(IVec3::new(x,y,z)),
                    ));
                }
            }
        }
    }
}

fn check_lines(
    mut game_grid: ResMut<GameGrid>,
    mut score: ResMut<Score>,
) {
    // Check 2D layers (y)
    let mut y = 0;
    while y < GRID_HEIGHT {
        let mut full = true;
        for x in 0..GRID_WIDTH {
            for z in 0..GRID_DEPTH {
                if game_grid.get(x, y, z).is_none() {
                    full = false;
                    break;
                }
            }
            if !full { break; }
        }

        if full {
            score.0 += 100;
            // Clear and move down
            // Shift everything above y down by 1
            for dy in y..(GRID_HEIGHT - 1) {
                for x in 0..GRID_WIDTH {
                    for z in 0..GRID_DEPTH {
                        let above = game_grid.get(x, dy + 1, z);
                        if let Some(c) = above {
                             game_grid.set(x, dy, z, c);
                        } else {
                             // clear
                             if let Some(idx) = GameGrid::index(x, dy, z) {
                                 game_grid.grid[idx] = None;
                             }
                        }
                    }
                }
            }
            // Clear top row
             for x in 0..GRID_WIDTH {
                for z in 0..GRID_DEPTH {
                     if let Some(idx) = GameGrid::index(x, GRID_HEIGHT-1, z) {
                         game_grid.grid[idx] = None;
                     }
                }
            }
            // Don't increment y, check this level again
        } else {
            y += 1;
        }
    }
}

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

fn ui_system(
    score: Res<Score>,
    next: Res<NextPiece>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<NextPieceText>) >,
    mut next_query: Query<&mut Text, With<NextPieceText>>,
) {
    for mut text in &mut score_query {
        text.0 = format!("Score: {}", score.0);
    }
    for mut text in &mut next_query {
        text.0 = format!("Next: {:?}", next.0);
    }
}

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

fn game_over_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut game_grid: ResMut<GameGrid>,
    mut score: ResMut<Score>,
    active_query: Query<Entity, With<ActiveBlock>>,
    landed_query: Query<Entity, With<LandedBlock>>,
    text_query: Query<Entity, With<GameOverText>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset Game
        *game_grid = GameGrid::new();
        score.0 = 0;
        
        // Cleanup entities
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
