use bevy::prelude::*;
use bevy::time::common_conditions::*;
use bevy::window::PrimaryWindow;
use core::time::Duration;
use rand::random;

const SNAKE_HEAD_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const FOOD_COLOR: Color = Color::srgb(1.0, 0.0, 1.0);
const SNAKE_SEGMENT_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

const ARENA_HEIGHT: u32 = 20;
const ARENA_WIDTH: u32 = 20;

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Deref, DerefMut, Resource)]
struct SnakeSegments(Vec<Entity>);

#[derive(Message)]
struct GrowthEvent;

#[derive(Message)]
struct GameOverEvent;

#[derive(Default, Resource)]
struct LastTailPosition(Option<Position>);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    #[warn(dead_code)]
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

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    *segments = SnakeSegments(vec![
        commands
            .spawn((
                Sprite::from_color(SNAKE_HEAD_COLOR, Vec2::ONE),
                Transform::default(),
            ))
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn((
            Sprite::from_color(SNAKE_SEGMENT_COLOR, Vec2::ONE),
            Transform::default(),
        ))
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
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

fn snake_movement(
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: MessageWriter<GameOverEvent>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
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
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.write(GameOverEvent);
        }
        if segment_positions.contains(&head_pos) {
            game_over_writer.write(GameOverEvent);
        }
        segment_positions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
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

fn food_spawner(
    mut commands: Commands,
    segments: ResMut<SnakeSegments>,
    mut positions: Query<&mut Position>,
) {
    let food_position = Position {
        x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
        y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
    };

    if !segments
        .iter()
        .map(|e| *positions.get_mut(*e).unwrap())
        .any(|segment_position| segment_position == food_position)
    {
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
            .insert(food_position)
            .insert(Size::square(0.8));
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: MessageWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.write(GrowthEvent);
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: MessageReader<GrowthEvent>,
) {
    if growth_reader.read().next().is_some() {
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: MessageReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.read().next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }
        spawn_snake(commands, segments_res);
    }
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
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_message::<GrowthEvent>()
        .add_message::<GameOverEvent>()
        .add_systems(Startup, (setup_camera, spawn_snake))
        .add_systems(Update, snake_movement_input.before(snake_movement))
        .add_systems(Update, snake_eating.after(snake_movement))
        .add_systems(Update, snake_growth.after(snake_eating))
        .add_systems(Update, game_over.after(snake_movement))
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
