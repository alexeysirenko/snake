use bevy::prelude::*;
use bevy::time::common_conditions::*;
use bevy::window::PrimaryWindow;
use core::time::Duration;
use rand::random;

const SNAKE_HEAD_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const FOOD_COLOR: Color = Color::srgb(1.0, 0.0, 1.0);

const ARENA_HEIGHT: u32 = 20;
const ARENA_WIDTH: u32 = 20;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[warn(dead_code)]
#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct Food;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn spawn_snake(mut commands: Commands) {
    //println!("Spawning snake!");
    commands
        .spawn((
            Sprite::from_color(SNAKE_HEAD_COLOR, Vec2::ONE),
            Transform::default(), // Add explicitly
        ))
        .insert(SnakeHead {
            direction: Direction::Up,
        })
        .insert(Position {
            x: ARENA_WIDTH as i32 / 2,
            y: ARENA_HEIGHT as i32 / 2,
        })
        .insert(Size::square(0.8));
}

fn snake_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut head_positions: Query<&mut SnakeHead>,
) {
    if let Some(mut head) = head_positions.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::ArrowLeft) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::ArrowUp) {
            Direction::Up
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    } else {
        //println!("Head position not found");
    }
}

fn snake_movement(mut heads: Query<(&mut Position, &SnakeHead)>) {
    if let Some((mut head_pos, head)) = heads.iter_mut().next() {
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };
    }
}

fn size_scaling(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let tile_size_x = window.width() / ARENA_WIDTH as f32;
    let tile_size_y = window.height() / ARENA_HEIGHT as f32;
    let tile_size = tile_size_x.min(tile_size_y);

    for (sprite_size, mut transform) in q.iter_mut() {
        let scale = tile_size * sprite_size.width;
        transform.scale = Vec3::new(scale, scale, 1.0);
        //println!("Scaling entity: scale={}", scale);
    }
}

fn position_translation(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let Ok(window) = window_query.single() else {
        return;
    };
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width(), ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32),
            0.0,
        );
        /*
        println!(
            "Position: ({}, {}) -> Translation: {:?}",
            pos.x, pos.y, transform.translation
        );
        */
    }
}

fn food_spawner(mut commands: Commands) {
    commands
        .spawn((
            Sprite {
                color: FOOD_COLOR,
                custom_size: Some(Vec2::ONE),
                ..default()
            },
            Transform::default(), // Add this!
        ))
        .insert(Food)
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake!".to_string(), // <--
                resolution: (800, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_camera, spawn_snake))
        .add_systems(Update, snake_movement_input.before(snake_movement))
        .add_systems(
            FixedUpdate,
            (
                food_spawner.run_if(on_timer(Duration::from_secs(1))),
                snake_movement.run_if(on_timer(Duration::from_millis(500))),
            ),
        )
        .add_systems(PostUpdate, (position_translation, size_scaling))
        .run();
}
