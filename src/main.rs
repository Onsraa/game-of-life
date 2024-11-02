use bevy::{
    core::FrameCount,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::prelude::*;

use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};

const GRID_WIDTH: usize = 50;
const GRID_HEIGHT: usize = 50;
const PIXEL_SIZE: f32 = 12.0;
const SPAWN_RATE: f64 = 0.5;
const EPISODE_REFRESH_RATE: f32 = 0.2;

#[derive(Component)]
struct Cell(bool);

#[derive(Component)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct EpisodeTimer(Timer);

#[derive(Resource)]
struct CellsUpdated(bool);

#[derive(Resource)]
struct NeighborCounts(Vec<Vec<u8>>);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (GRID_WIDTH  as f32 * PIXEL_SIZE, GRID_HEIGHT as f32 * PIXEL_SIZE).into(),
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextStyle {
                        font_size: 10.0,
                        color: Color::srgb(0.0, 1.0, 0.0),
                        font: default(),
                    },
                },
            },
            CellsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (make_visible, update_camera))
        .run();
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 3 {
        window.single_mut().visible = true;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_camera(mut camera: Query<&mut Transform, With<Camera2d>>) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };
    let direction = Vec3::new(GRID_WIDTH as f32 * PIXEL_SIZE / 2.0, GRID_HEIGHT as f32 * PIXEL_SIZE / 2.0, camera.translation.z);
    camera.translation = direction;
}

fn initialize_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    let color_alive = Color::hsl(225.0, 0.95, 1.0);
    let color_dead = Color::hsl(225.0, 0.95, 0.0);

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let is_alive = rng.gen::<f64>() <= SPAWN_RATE;
            let color = if is_alive { color_alive } else { color_dead };

            let pixel = Mesh2dHandle(meshes.add(Rectangle::new(PIXEL_SIZE, PIXEL_SIZE)));
            commands.spawn((
                Cell(is_alive),
                Position { x, y },
                MaterialMesh2dBundle {
                    mesh: pixel,
                    material: materials.add(color),
                    transform: Transform::from_xyz(
                        x as f32 * PIXEL_SIZE + PIXEL_SIZE / 2.0,
                        y as f32 * PIXEL_SIZE + PIXEL_SIZE / 2.0,
                        0.0,
                    ),
                    ..default()
                },
            ));
        }
    }
}

fn start_episode(
    time: Res<Time>,
    mut timer: ResMut<EpisodeTimer>,
    mut cells_updated: ResMut<CellsUpdated>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        cells_updated.0 = false;
    }
}

fn count_neighbors(
    mut neighbor_counts: ResMut<NeighborCounts>,
    cells: Query<(&Position, &Cell)>,
) {
    neighbor_counts.0.iter_mut().for_each(|row| row.fill(0));

    for (pos, cell) in cells.iter() {
        if !cell.0 {
            continue;
        }
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = pos.x as isize + dx;
                let ny = pos.y as isize + dy;
                if nx >= 0 && ny >= 0 && nx < GRID_WIDTH as isize && ny < GRID_HEIGHT as isize {
                    neighbor_counts.0[nx as usize][ny as usize] += 1;
                }
            }
        }
    }
}

fn update_cells(
    mut cells_updated: ResMut<CellsUpdated>,
    neighbor_counts: Res<NeighborCounts>,
    mut cells: Query<(&mut Cell, &Position, &mut Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if cells_updated.0 {
        return;
    }

    let color_alive = Color::hsl(225.0, 0.95, 1.0);
    let color_dead = Color::hsl(225.0, 0.95, 0.0);

    for (mut cell, pos, material_handle) in cells.iter_mut() {
        let count = neighbor_counts.0[pos.x][pos.y];
        let was_alive = cell.0;
        cell.0 = match count {
            3 => true,
            2 if cell.0 => true,
            _ => false,
        };

        if cell.0 != was_alive {
            let color = if cell.0 { color_alive } else { color_dead };
            if let Some(material) = materials.get_mut(&*material_handle) {
                material.color = color;
            }
        }
    }

    cells_updated.0 = true;
}

struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NeighborCounts(vec![vec![0; GRID_HEIGHT]; GRID_WIDTH]))
            .insert_resource(CellsUpdated(true))
            .insert_resource(EpisodeTimer(Timer::from_seconds(
                EPISODE_REFRESH_RATE,
                TimerMode::Repeating,
            )))
            .add_systems(Startup, initialize_cells)
            .add_systems(Update, (start_episode, count_neighbors, update_cells, draw_gizmos, config_gizmos).chain());
    }
}


fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Vec2::ZERO,
            0.0,
            UVec2::new(GRID_WIDTH as u32 * 2, GRID_HEIGHT as u32 * 2),
            Vec2::new(PIXEL_SIZE, PIXEL_SIZE),
            LinearRgba::gray(0.15),
        )
        .outer_edges();
}

fn config_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line_width = 1.0;
}
