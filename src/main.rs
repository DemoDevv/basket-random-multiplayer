use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;
use iyes_perf_ui::{PerfUiCompleteBundle, PerfUiPlugin};

const NUMBER_OF_TEAMS: i8 = 2; // if you set more than 2 teams, you will encounter some bugs with the spawn position of the players
const NUMBER_OF_PLAYERS: i8 = 2;

const TARGET_ORIENTATION: f32 = 0.0;
const K: f32 = 200_000_000.0;
const TORQUE_ON_COLLIDE: f32 = 30_000_000.0;
const MAX_ANGLE_ROTATION_FOR_ARM: f32 = 155.0;
const MIN_ANGLE_ROTATION_FOR_ARM: f32 = 1.0;
const SPEED_ROTATION: f32 = 6.3;

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
            (
                detect_player_collide_with_ground,
                detect_player_collide_with_player,
                apply_torque,
                jump_system,
                rotate_arms,
            )
                .chain(),
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
    commands.insert_resource(TeamScore::default());

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
        commands.spawn(Hoop {
            collider: Collider::cuboid(30.0, 10.0),
            side: if i == 0 { Side::LEFT } else { Side::RIGHT },
            sensor: Sensor,
            transform: TransformBundle::from(Transform::from_xyz(
                if i == 0 { -400.0 } else { 400.0 },
                200.0,
                0.0,
            )),
        });
    }

    for i in 0..NUMBER_OF_TEAMS {
        let color = Color::hsl(360. * i as f32 / 3.0, 0.95, 0.7);

        let side = if i == 0 { Side::LEFT } else { Side::RIGHT };

        for y in 0..NUMBER_OF_PLAYERS {
            // posibly optimize this
            let position = if i == 0 {
                (y + 1) as f32 * -150.0
            } else {
                (y + 1) as f32 * 150.0
            };

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
                    position, // multiplier par -1 pour inverser la position
                    400.0, 0.0,
                )),
                is_on_ground: IsOnGround::default(),
                side: side.clone(),
            };

            let entity = commands
                .spawn(player)
                .insert(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Capsule2d {
                        radius: 15.0,
                        half_length: 40.0,
                    })),
                    material: materials.add(color),
                    transform: Transform::from_xyz(position, 400.0, 0.0),
                    ..default()
                })
                .id();

            let arm_x_position = if i == 0 { -1.0 } else { 1.0 };

            let squeleton_arm_entity = commands
                .spawn((
                    TransformBundle::from(Transform::from_xyz(7.0 * arm_x_position, 17.0, 0.0)),
                    Skeleton,
                    InheritedVisibility::VISIBLE,
                    side.clone(),
                ))
                .id();

            commands.entity(entity).add_child(squeleton_arm_entity);

            let arm = commands
                .spawn((
                    Arm {
                        angle: 0.0,
                        length: 40.0,
                    },
                    Collider::cuboid(7.0, 40.0),
                    Sensor,
                    TransformBundle::from(Transform::from_xyz(0.0, -30.0, 0.0)),
                    ColliderMassProperties::Mass(0.0),
                ))
                .insert(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle {
                        half_size: Vec2::new(7.0, 40.0),
                    })),
                    material: materials.add(color),
                    transform: Transform::from_xyz(0.0, -30.0, 0.0),
                    ..default()
                })
                .id();

            commands.entity(squeleton_arm_entity).add_child(arm);

            let sensor = commands
                .spawn((
                    Collider::ball(15.0),
                    Sensor,
                    ColliderMassProperties::Mass(0.0),
                    TransformBundle::from(Transform::from_xyz(0.0, -40.0, 0.0)),
                ))
                .id();

            commands.entity(arm).add_child(sensor);
        }
    }

    // faire apparaitre la balle
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(17.0))
        .insert(Restitution::coefficient(1.1))
        .insert(GravityScale(0.5))
        .insert(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 17.0 })),
            material: materials.add(Color::ORANGE),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)));
}

fn apply_torque(mut rigid_bodies: Query<(&Transform, &mut ExternalForce), With<RigidBody>>) {
    for (transform, mut force) in rigid_bodies.iter_mut() {
        force.torque = K * (TARGET_ORIENTATION - transform.rotation.z);
    }
}

#[derive(Debug, Component, Clone)]
enum Side {
    LEFT,
    RIGHT,
}

#[derive(Resource, Default)]
struct TeamScore {
    left: u8,
    right: u8,
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

#[derive(Debug, Bundle)]
struct Hoop {
    collider: Collider,
    sensor: Sensor,
    side: Side,
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
    side: Side,
}

#[derive(Debug, Component)]
struct Arm {
    angle: f32,
    length: f32,
}

#[derive(Debug, Component)]
struct Skeleton;

#[derive(Debug, Component, Default)]
struct IsOnGround(bool);

// fonctionne pas car il faudrait pouvoir tester cela lors du saut
fn detect_player_collide_with_player(
    mut collision_events: EventReader<CollisionEvent>,
    mut entities_q: Query<(Entity, &Transform, &mut IsOnGround), With<RigidBody>>,
) {
    // vérifier si un joueur est en collision avec un autre joueur et perpendiculaire à l'autre joueur.
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(player1, player2, _) = collision_event {
            let up_direction_player_1 = entities_q.iter().find_map(|(entity, transform, _)| {
                if entity == *player1 {
                    Some(transform.rotation * Vec3::Y)
                } else {
                    None
                }
            });
            let up_direction_player_2 = entities_q.iter().find_map(|(entity, transform, _)| {
                if entity == *player2 {
                    Some(transform.rotation * Vec3::Y)
                } else {
                    None
                }
            });

            if up_direction_player_1.is_none() || up_direction_player_2.is_none() {
                continue;
            }

            let dot_product = up_direction_player_1
                .unwrap()
                .dot(up_direction_player_2.unwrap());

            if dot_product < f32::EPSILON && dot_product > -f32::EPSILON {
                // set is_on_ground to true
                // pour le joueur perpendiculaire à l'autre joueur
                for (entity, _, mut is_on_ground) in entities_q.iter_mut() {
                    if entity == *player1 && !is_on_ground.0 {
                        is_on_ground.0 = true;
                    }
                }
            }
        }
    }
}

fn detect_player_collide_with_ground(
    mut collision_events: EventReader<CollisionEvent>,
    grounds_q: Query<Entity, With<Friction>>,
    mut entities_q: Query<
        (
            Entity,
            &Transform,
            &mut ExternalImpulse,
            &mut IsOnGround,
            &Side,
        ),
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
            for (entity, _, _, is_on_ground, _) in entities_q.iter_mut() {
                if entity.index() == player.index() {
                    if is_on_ground.0 {
                        return;
                    }
                }
            }

            for (entity, transform, mut external_impulse, mut is_on_ground, side) in
                entities_q.iter_mut()
            {
                if entity.index() == player.index() {
                    let direction;

                    if transform.rotation.to_axis_angle().0.z == 0.0 {
                        direction = match side {
                            Side::LEFT => -1.0,
                            Side::RIGHT => 1.0,
                        }
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

// FIXME: verifier le coté du bras avec le vec3 suivant la team des joueurs
fn rotate_arms(
    time: Res<Time>,
    keyboard_inputs: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Side), With<Skeleton>>,
) {
    for (mut transform, side) in query.iter_mut() {
        let direction;

        match side {
            Side::LEFT => direction = 1.0,
            Side::RIGHT => direction = -1.0,
        }

        if keyboard_inputs.pressed(KeyCode::Space)
            && transform.rotation.to_axis_angle().1.to_degrees() < MAX_ANGLE_ROTATION_FOR_ARM
        {
            // lever le bras
            transform.rotate(Quat::from_rotation_z(
                SPEED_ROTATION * time.delta_seconds() * direction,
            ));
        } else if !keyboard_inputs.pressed(KeyCode::Space)
            && transform.rotation.to_axis_angle().1.to_degrees() > MIN_ANGLE_ROTATION_FOR_ARM
        {
            // baisser le bras
            transform.rotate(Quat::from_rotation_z(
                -SPEED_ROTATION * time.delta_seconds() * direction,
            ));
        }
    }
}
