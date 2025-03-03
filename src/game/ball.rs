use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

use super::{level::Hoop, player::Hand, GameState, GRAVITE_SCALE_BALL};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_ball)
            .add_systems(Update, make_shoot)
            .add_systems(FixedUpdate, follow_hand);
    }
}

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct BallPossession {
    pub user: Entity,
}

fn setup_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Ball)
        .insert(Collider::ball(17.0))
        .insert(Restitution::coefficient(1.1))
        .insert(GravityScale(GRAVITE_SCALE_BALL))
        .insert(Velocity::linear(Vec2::new(0.0, 0.0)))
        .insert(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 17.0 })),
            material: materials.add(Color::ORANGE),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 400.0, 1.0)));
}

fn follow_hand(
    mut q_ball: Query<(&mut Transform, Option<&BallPossession>), With<Ball>>,
    q_players: Query<&GlobalTransform, (With<Hand>, Without<Ball>)>,
) {
    let (mut ball_transform, ball_possession) = q_ball.single_mut();
    if let Some(ball_possession) = ball_possession {
        let user = q_players.get(ball_possession.user).unwrap();
        ball_transform.translation = user.compute_transform().translation;
    }
}

fn make_shoot(
    mut commands: Commands,
    keyboard_inputs: Res<ButtonInput<KeyCode>>,
    q_hoops: Query<&Transform, With<Hoop>>,
    mut q_ball: Query<(Entity, &Transform, &mut Velocity, Option<&BallPossession>), With<Ball>>,
) {
    if !keyboard_inputs.just_released(KeyCode::Space) {
        return;
    }

    let ball = q_ball.single();
    let is_possessed = ball.3.is_some();

    if !is_possessed {
        return;
    }

    // remove the ball possession component
    commands.entity(ball.0).remove::<BallPossession>();

    // get the player transform and the hoop
    let ball_transform = ball.1;
    let hoop = q_hoops.iter().next().unwrap();

    let first_distance = Vec2::new(
        hoop.translation.x - ball_transform.translation.x,
        hoop.translation.y - ball_transform.translation.y,
    );

    // use the direction of the first distance to know if the player is on the left or right of the hoop
    let direction = first_distance.normalize();

    // calculate the target position because if the player is too close to the hoop, the ball will be shooted from the side
    let target_position = if first_distance.x.abs() < 200.0 {
        if direction.x > 0.0 {
            hoop.translation + Vec3::Y * 70.0 - Vec3::X * 20.0
        } else {
            hoop.translation + Vec3::Y * 70.0 + Vec3::X * 20.0
        }
    } else {
        hoop.translation
    };

    // calculate the distance between the player and the hoop
    let distance = Vec2::new(
        target_position.x - ball_transform.translation.x,
        target_position.y - ball_transform.translation.y,
    );

    let dx = distance.x;
    let dy = distance.y;

    let angle = dy.atan2(dx) / 2.0 + std::f32::consts::FRAC_PI_4;
    let tan_angle = angle.tan();

    // calculate the speed of the ball with equation of motion
    let speed = ((9.81 * GRAVITE_SCALE_BALL) * (dx).powi(2) * (tan_angle.powi(2) + 1.0)
        / (2.0 * (dx * tan_angle - dy)))
        .sqrt()
        * 14.6;

    let mut velocity = q_ball.single_mut().2;
    velocity.linvel = Vec2::new(speed * angle.cos(), speed * angle.sin());
}
