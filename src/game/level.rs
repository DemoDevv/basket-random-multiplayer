use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

use super::GameState;
use super::Side;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_level);
    }
}

#[derive(Debug, Bundle)]
struct Ground {
    collider: Collider,
    friction: Friction,
    transform: TransformBundle,
}

#[derive(Debug, Bundle)]
struct Wall {
    collider: Collider,
    transform: TransformBundle,
}

#[derive(Debug, Component)]
pub struct Hoop;

#[derive(Debug, Bundle)]
struct HoopBundle {
    hoop: Hoop,
    collider: Collider,
    sensor: Sensor,
    side: Side,
    transform: TransformBundle,
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // create the ground
    commands
        .spawn(Ground {
            collider: Collider::cuboid(500.0, 50.0),
            friction: Friction {
                // FIXME: don't add friction with the ball
                coefficient: 0.20,
                combine_rule: CoefficientCombineRule::Min,
            },
            transform: TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)),
        })
        .insert(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(1000.0, 100.0))),
            material: materials.add(Color::DARK_GRAY),
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            ..default()
        });

    // create the two walls on the side but transparent
    commands.spawn(Wall {
        collider: Collider::cuboid(50.0, 200.0),
        transform: TransformBundle::from(Transform::from_xyz(-550.0, 0.0, 0.0)),
    });

    commands.spawn(Wall {
        collider: Collider::cuboid(50.0, 200.0),
        transform: TransformBundle::from(Transform::from_xyz(550.0, 0.0, 0.0)),
    });

    // create the two hoops on the two sides
    for i in 0..2 {
        let hoop = commands
            .spawn(HoopBundle {
                hoop: Hoop,
                collider: Collider::cuboid(30.0, 10.0),
                side: if i == 0 { Side::LEFT } else { Side::RIGHT },
                sensor: Sensor,
                transform: TransformBundle::from(Transform::from_xyz(
                    if i == 0 { -400.0 } else { 400.0 },
                    200.0,
                    0.0,
                )),
            })
            .id();

        let hoop_back_x_position = if i == 0 { -1.0 } else { 1.0 };

        // make the back of the hoop
        let hoop_back = commands
            .spawn((
                TransformBundle::from(Transform::from_xyz(hoop_back_x_position * 37.0, 40.0, 0.0)),
                Collider::cuboid(7.0, 40.0),
            ))
            .id();

        commands.entity(hoop).add_child(hoop_back);
    }
}
