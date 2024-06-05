use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;
use iyes_perf_ui::{PerfUiCompleteBundle, PerfUiPlugin};

const NUMBER_OF_PLAYERS: i8 = 2;

const TARGET_ORIENTATION: f32 = 0.0;
const K: f32 = 200_000_000.0;
const TORQUE_ON_COLLIDE: f32 = 30_000_000.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .add_systems(
            Update,
            (detect_player_collide_with_ground, apply_torque, jump_system).chain(),
        )
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(PerfUiCompleteBundle::default());
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // create the ground
    let _ground = commands.spawn(Ground {
        collider: Collider::cuboid(500.0, 50.0),
        friction: Friction {
            coefficient: 0.20,
            combine_rule: CoefficientCombineRule::Min,
        },
        transform: TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)),
    });
    // .insert(MaterialMesh2dBundle {
    //     mesh: Mesh2dHandle(meshes.add(Rectangle::new(1000.0, 100.0))),
    //     material: materials.add(Color::WHITE),
    //     ..default()
    // })

    // create the bouncing ball
    for i in 0..NUMBER_OF_PLAYERS {
        let color = Color::hsl(360. * i as f32 / 3.0, 0.95, 0.7);

        let player = Player {
            rigid_bodie: RigidBody::Dynamic,
            collider: Collider::capsule_y(40.0, 15.0),
            external_impulse: ExternalImpulse { ..default() },
            externel_force: ExternalForce {
                force: Vec2::new(0.0, 0.0),
                torque: 0.0,
            },
            restitution: Restitution::coefficient(0.7),
            damping: Damping {
                linear_damping: 1.0,
                angular_damping: 0.0000000000001,
            },
            gravity_scale: GravityScale(0.30),
            active_events: ActiveEvents::COLLISION_EVENTS,
            transform: TransformBundle::from(Transform::from_xyz(
                (i + 1) as f32 * -150.0,
                400.0,
                0.0,
            )),
            is_on_ground: IsOnGround::default(),
        };

        let _entity = commands.spawn(player).insert(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Capsule2d {
                radius: 15.0,
                half_length: 40.0,
            })),
            material: materials.add(color),
            transform: Transform::from_xyz((i + 1) as f32 * -150.0, 400.0, 0.0),
            ..default()
        });
    }

    // faire apparaitre la balle
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(17.0))
        .insert(Restitution::coefficient(1.1))
        .insert(GravityScale(0.5))
        .insert(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 17.0 })),
            material: materials.add(Color::rgb(0.7, 0.2, 0.3)),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)));
}

fn apply_torque(mut rigid_bodies: Query<(&Transform, &mut ExternalForce), With<RigidBody>>) {
    for (transform, mut force) in rigid_bodies.iter_mut() {
        force.torque = K * (TARGET_ORIENTATION - transform.rotation.z);
    }
}

enum Team {
    RED(Side),
    BLEU(Side),
}

enum Side {
    LEFT,
    RIGHT,
}

#[derive(Debug, Bundle)]
struct Ground {
    collider: Collider,
    friction: Friction,
    transform: TransformBundle,
}

#[derive(Debug, Bundle)]
struct Player {
    rigid_bodie: RigidBody,
    collider: Collider,
    external_impulse: ExternalImpulse,
    externel_force: ExternalForce,
    restitution: Restitution,
    damping: Damping,
    gravity_scale: GravityScale,
    active_events: ActiveEvents,
    transform: TransformBundle,
    is_on_ground: IsOnGround,
}

#[derive(Debug, Component, Default)]
struct IsOnGround(bool);

fn detect_player_collide_with_ground(
    mut collision_events: EventReader<CollisionEvent>,
    grounds_q: Query<Entity, With<Friction>>,
    mut entities_q: Query<
        (Entity, &Transform, &mut ExternalImpulse, &mut IsOnGround),
        With<RigidBody>,
    >,
) {
    for collision_event in collision_events.read() {
        // FIXME: check if the collison is started with the ground and not another guy
        if let CollisionEvent::Started(ground_c, player, _) = collision_event {
            for ground in grounds_q.iter() {
                if ground != *ground_c {
                    // ce n'est pas une collision avec le sol
                    return;
                }
            }

            // on verifie si il est déjà au sol
            for (entity, _, _, is_on_ground) in entities_q.iter_mut() {
                if entity.index() == player.index() {
                    if is_on_ground.0 {
                        return;
                    }
                }
            }

            for (entity, transform, mut external_impulse, mut is_on_ground) in entities_q.iter_mut()
            {
                if entity.index() == player.index() {
                    let direction;

                    if transform.rotation.to_axis_angle().0.z == 0.0 {
                        direction = 1.0;
                    } else {
                        direction = transform.rotation.to_axis_angle().0.z
                    }

                    external_impulse.torque_impulse = TORQUE_ON_COLLIDE * direction;
                    is_on_ground.0 = true;
                }
            }
        }
    }
}

fn jump_system(
    keyboard_inputs: Res<ButtonInput<KeyCode>>,
    mut rigid_bodies: Query<(&Transform, &mut ExternalImpulse, &mut IsOnGround), With<RigidBody>>,
) {
    for (bodie, mut impulse, mut is_on_ground) in rigid_bodies.iter_mut() {
        let up_bodie_direction = bodie.rotation * Vec3::new(0.0, 0.1, 0.0);
        let up_direction = Vec3::new(0.0, 0.1, 0.0);
        let dot_product = up_bodie_direction.dot(up_direction);
        let mag_up_bodie = up_bodie_direction.length();
        let mag_up = up_direction.length();
        let cos_theta = dot_product / (mag_up * mag_up_bodie);
        let theta = cos_theta.acos();
        let angle = theta.to_degrees();
        if keyboard_inputs.just_pressed(KeyCode::Space) && is_on_ground.0 && angle <= 80.0 {
            let up_direction_2d = Vec2::new(up_bodie_direction.x, up_bodie_direction.y);

            let force = up_direction_2d * 17_000_000.0;
            impulse.impulse = force;

            is_on_ground.0 = false;
        }
    }
}
