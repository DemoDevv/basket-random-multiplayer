use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // create the ground
    commands
        .spawn(Collider::cuboid(500.0, 50.0))
        .insert(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(1000.0, 100.0))),
            material: materials.add(Color::WHITE),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));

    // create the bouncing ball
    for i in 0..3 {
        let color = Color::hsl(360. * i as f32 / 3.0, 0.95, 0.7);

        commands
            .spawn(RigidBody::Dynamic)
            .insert(Collider::ball(50.0))
            .insert(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
                material: materials.add(color),
                ..default()
            })
            .insert(Restitution::coefficient(0.7))
            .insert(TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)));
    }
}
