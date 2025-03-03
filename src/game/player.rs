use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

use crate::game::ball::BallPossession;

use super::{
    K, MAX_ANGLE_ROTATION_FOR_ARM, MIN_ANGLE_ROTATION_FOR_ARM, SPEED_ROTATION, TARGET_ORIENTATION,
    TORQUE_ON_COLLIDE,
};

use super::{ball::Ball, Side};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                apply_torque,
                jump_system,
                rotate_arms,
                detect_player_collide_with_ground,
                detect_hand_collide_with_ball,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
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

#[derive(Component)]
struct Arm;

#[derive(Component)]
pub struct Hand;

#[derive(Component)]
struct Skeleton;

#[derive(Component, Default)]
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
            Arm,
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

        let cos_theta = up_bodie_direction.y / up_bodie_direction.length();
        let angle = cos_theta.acos().to_degrees();

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
    mut skeletons: Query<(&mut Transform, &Side), With<Skeleton>>,
) {
    for (mut transform, side) in skeletons.iter_mut() {
        let direction = match side {
            Side::LEFT => 1.0,
            Side::RIGHT => -1.0,
        };

        let mut angle = transform
            .rotation
            .to_euler(EulerRot::XYZ)
            .2
            .to_degrees()
            .abs();

        let rotation_amount = SPEED_ROTATION * time.delta_seconds();

        if keyboard_inputs.pressed(KeyCode::Space) {
            angle = (angle + rotation_amount)
                .clamp(MIN_ANGLE_ROTATION_FOR_ARM, MAX_ANGLE_ROTATION_FOR_ARM);
        } else {
            angle = (angle - rotation_amount)
                .clamp(MIN_ANGLE_ROTATION_FOR_ARM, MAX_ANGLE_ROTATION_FOR_ARM);
        }

        transform.rotation = Quat::from_rotation_z((angle * direction).to_radians());
    }
}

fn detect_hand_collide_with_ball(
    mut commands: Commands,
    keyboard_inputs: Res<ButtonInput<KeyCode>>,
    mut collision_events: EventReader<CollisionEvent>,
    hands_q: Query<Entity, With<Hand>>,
    balls_q: Query<Entity, With<Ball>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(hand_c, ball_c, _) = collision_event {
            let hand = hands_q.get(*hand_c).ok();
            let ball = balls_q.get(*ball_c).ok();

            let hand_alt = balls_q.get(*ball_c).ok();
            let ball_alt = hands_q.get(*hand_c).ok();

            if let Some(hand) = hand.or(hand_alt) {
                if let Some(ball) = ball.or(ball_alt) {
                    if !keyboard_inputs.pressed(KeyCode::Space) {
                        return;
                    }

                    // Ajouter la possession de la balle
                    commands.entity(ball).insert(BallPossession { user: hand });
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
