use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use float_cmp::approx_eq;

const GRID_WIDTH: f32 = 50.0;
const GRID_HEIGHT: f32 = 50.0;
const PIXEL_SIZE: f32 = 15.0;

#[derive(Component)]
struct Cell(bool); // bool for is alive or dead

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct NeighborCount {
    count: u8,
}

#[derive(Resource)]
struct EpisodeTimer(Timer);

#[derive(Resource)]
struct CellsUpdated(bool);

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_systems(Update, make_visible)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: ((GRID_WIDTH * PIXEL_SIZE), (GRID_HEIGHT * PIXEL_SIZE)).into(),
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
            CellsPlugin))
        .add_systems(Update, (draw_gizmos, config_gizmos, update_camera))
        .run();
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 3 {
        // At this point the gpu is ready to show the app so we can make the window visible.
        // Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.single_mut().visible = true;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_camera(mut camera: Query <&mut Transform, With<Camera2d>>) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };
    let direction = Vec3::new(GRID_WIDTH * PIXEL_SIZE / 2.0, GRID_HEIGHT * PIXEL_SIZE / 2.0, camera.translation.z);
    camera.translation = direction;
}

fn initialize_cells(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    
    let color = Color::hsl(225.0, 0.95, 0.0);

    for x in 0..GRID_WIDTH as u32 {
        for y in 0.. GRID_HEIGHT as u32 {
            
            let pixel = Mesh2dHandle(meshes.add(Rectangle::new(PIXEL_SIZE, PIXEL_SIZE)));
            commands.spawn((
                Cell(false), 
                Position {x: x as f32, y: y as f32}, 
                NeighborCount{count : 0},
                MaterialMesh2dBundle{
                    mesh: pixel,
                    material: materials.add(color),
                    transform: Transform::from_xyz(x as f32 * PIXEL_SIZE + PIXEL_SIZE / 2.0, y  as f32 * PIXEL_SIZE + PIXEL_SIZE / 2.0, 0.0),
                    ..default()
                })
            );
        }
    }
}

fn start_episode(time: Res<Time>, mut timer: ResMut<EpisodeTimer>, mut cells_updated: ResMut<CellsUpdated>){
    if timer.0.tick(time.delta()).just_finished(){
        println!("NEW EPISODE");
        cells_updated.0 = false;
    }
}

fn update_cells(mut cells_updated: ResMut<CellsUpdated>,mut query: Query<(&mut Cell, &mut NeighborCount, &Position)>, all_positions: Query<&Position>) {
    
    if cells_updated.0 { return };
    
    // Update neighbors first
    for (_cell, mut neighbor_count, position) in &mut query {
        let mut count: u8 = 0;
        for other_position in &all_positions {
            if position.x == other_position.x && position.y == other_position.y {
                continue
            }
            if position.x == other_position.x {
                if position.y == (other_position.y + 1.0) {count += 1};
                if !approx_eq!(f32, other_position.y, position.y, epsilon = 1e-10) {count += 1};
            }
            if position.y == other_position.y {
                if position.x == (other_position.x + 1.0) {count += 1};
                if !approx_eq!(f32, other_position.x, position.x, epsilon = 1e-10) && position.x == (other_position.x - 1.0) {count += 1};
            }
        }
        neighbor_count.count = count;
    }
    
    // Update cells 
    for (mut cell, neighbor_count, _position) in &mut query{
        if neighbor_count.count == 3 { cell.0 = true }
        else if neighbor_count.count < 2 || neighbor_count.count > 3 { cell.0 = false }
    }
    
    cells_updated.0 = true;
}

struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CellsUpdated(true));
        app.insert_resource(EpisodeTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, initialize_cells);
        app.add_systems(Update, (start_episode, update_cells).chain());
    }
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Vec2::ZERO,
            0.0,
            UVec2::new(GRID_WIDTH as u32 * 2, GRID_HEIGHT as u32 * 2),
            Vec2::new(PIXEL_SIZE, PIXEL_SIZE),
            // Dark gray
            LinearRgba::gray(0.15),
        )
        .outer_edges();
}

fn config_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line_width = 1.;
}