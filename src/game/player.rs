use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

use super::{
    K, MAX_ANGLE_ROTATION_FOR_ARM, MIN_ANGLE_ROTATION_FOR_ARM, SPEED_ROTATION, TARGET_ORIENTATION,
    TORQUE_ON_COLLIDE,
};

use super::{ball::Ball, Side};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                detect_player_collide_with_ground,
                apply_torque,
                jump_system,
                rotate_arms,
                detect_hand_collide_with_ball,
            )
                .chain(),
        );
    }
}

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Bundle)]
struct PlayerBundle {
    player: Player,
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
struct Hand;

#[derive(Debug, Component)]
struct Skeleton;

#[derive(Debug, Component, Default)]
struct IsOnGround(bool);

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    side: Side,
    color: Color,
    position: f32,
) {
    let player = PlayerBundle {
        player: Player,
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
        transform: TransformBundle::from(Transform::from_xyz(position, 400.0, 0.0)),
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

    let arm_x_position = if side == Side::LEFT { -1.0 } else { 1.0 };

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
            Hand,
            Collider::ball(15.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ColliderMassProperties::Mass(0.0),
            TransformBundle::from(Transform::from_xyz(0.0, -40.0, 0.0)),
        ))
        .id();

    commands.entity(arm).add_child(sensor);
}

fn apply_torque(mut rigid_bodies: Query<(&Transform, &mut ExternalForce), With<RigidBody>>) {
    for (transform, mut force) in rigid_bodies.iter_mut() {
        force.torque = K * (TARGET_ORIENTATION - transform.rotation.z);
    }
}

fn jump_system(
    keyboard_inputs: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&Transform, &mut ExternalImpulse, &mut IsOnGround), With<Player>>,
) {
    for (bodie, mut impulse, mut is_on_ground) in players.iter_mut() {
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

fn detect_hand_collide_with_ball(
    mut collision_events: EventReader<CollisionEvent>,
    hands_q: Query<Entity, With<Hand>>,
    balls_q: Query<Entity, With<Ball>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(hand_c, ball_c, _) = collision_event {
            // TODO: fix la balle au bras du joueur et en conséquence choisir le panier target du joueur
            let hand = if hands_q.get(*hand_c).is_ok() {
                Some(hand_c)
            } else if hands_q.get(*ball_c).is_ok() {
                Some(ball_c)
            } else {
                None
            };

            let ball = if balls_q.get(*hand_c).is_ok() {
                Some(hand_c)
            } else if balls_q.get(*ball_c).is_ok() {
                Some(ball_c)
            } else {
                None
            };

            if let (Some(hand), Some(ball)) = (hand, ball) {
                println!(
                    "Collision entre une main {:?} et une balle {:?}",
                    hand, ball
                );
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
