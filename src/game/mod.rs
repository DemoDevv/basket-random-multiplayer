use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use ball::BallPlugin;
use iyes_perf_ui::PerfUiCompleteBundle;
use level::LevelPlugin;
use player::{spawn_player, PlayerPlugin};

mod ball;
mod level;
mod player;
mod ui;

pub const TARGET_ORIENTATION: f32 = 0.0;
pub const K: f32 = 200_000_000.0;
pub const TORQUE_ON_COLLIDE: f32 = 30_000_000.0;
pub const MAX_ANGLE_ROTATION_FOR_ARM: f32 = 155.0;
pub const MIN_ANGLE_ROTATION_FOR_ARM: f32 = 0.0;
pub const SPEED_ROTATION: f32 = 300.0;
pub const GRAVITE_SCALE_BALL: f32 = 0.4;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins((LevelPlugin, PlayerPlugin, BallPlugin))
            .insert_state(GameState::Playing)
            .add_systems(Startup, setup_graphics)
            .add_systems(
                OnEnter(GameState::Playing),
                (setup_scores, spawn_teams).chain(),
            );
    }
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(PerfUiCompleteBundle::default());
}

#[derive(States, Clone, Debug, Eq, Hash, PartialEq)]
pub enum GameState {
    Paused,
    Playing,
}

#[derive(Debug, Component, Clone, PartialEq)]
pub enum Side {
    LEFT,
    RIGHT,
}

#[derive(Resource, Default)]
struct TeamScore {
    left: u8,
    right: u8,
}

fn setup_scores(mut commands: Commands) {
    commands.insert_resource(TeamScore::default());
}

fn spawn_teams(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_player(
        &mut commands,
        &mut meshes,
        &mut materials,
        Side::LEFT,
        Color::BLUE,
        2. * -150.,
    );
    spawn_player(
        &mut commands,
        &mut meshes,
        &mut materials,
        Side::LEFT,
        Color::BLUE,
        1. * -150.,
    );

    spawn_player(
        &mut commands,
        &mut meshes,
        &mut materials,
        Side::RIGHT,
        Color::RED,
        1. * 150.,
    );
    spawn_player(
        &mut commands,
        &mut meshes,
        &mut materials,
        Side::RIGHT,
        Color::RED,
        2. * 150.,
    );
}
