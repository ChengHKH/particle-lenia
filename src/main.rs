use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::prelude::*;

#[derive(Component)]
struct Creature;

#[derive(Component)]
struct Particle;

#[derive(Bundle)]
struct CreatureBundle {
    spatial: SpatialBundle,
    creature: Creature,
}

#[derive(Bundle)]
struct ParticleBundle {
    materialmesh2d: MaterialMesh2dBundle<ColorMaterial>,
    particle: Particle,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_creature))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_creature(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();

    commands.spawn(CreatureBundle {
        spatial: SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        creature: Creature,
    }).with_children(|parent| {
        for _ in 0..200 {
            let r = 150.0 * (rng.gen_range(0.0..=1.0) as f32).sqrt();
            let theta = (rng.gen_range(0.0..=1.0) as f32) * 2.0 * PI;

            parent.spawn(ParticleBundle {
                materialmesh2d: MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(5.).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(r * theta.cos(), r * theta.sin(), 0.0)),
                    ..default()
                },
                particle: Particle,
            });
        }
    });
}