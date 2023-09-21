#![allow(non_snake_case)]

use std::{f32::consts::TAU, iter};

use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
use rand::prelude::*;

#[derive(Component)]
struct Creature;

#[derive(Component)]
struct Parameters {
    mu_k: f32,
    sigma_k: f32,
    w_k: f32,
    
    mu_g: f32,
    sigma_g: f32,

    c_rep: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            mu_k: 4.0,
            sigma_k: 1.0,
            w_k: 0.022,
            
            mu_g: 0.6,
            sigma_g: 0.15,
            
            c_rep: 1.0,
        }
    }    
}

#[derive(Component)]
struct Particle;

#[derive(Component)]
struct Fields {
    R_val: f32,
    R_grad: Vec3,

    U_val: f32,
    U_grad: Vec3,

    E_grad: Vec3,
}

impl Default for Fields {
    fn default() -> Self {
        Self {
            R_val: 0.0,
            R_grad: Vec3::ZERO,

            U_val: 0.0,
            U_grad: Vec3::ZERO,

            E_grad: Vec3::ZERO,
        }
    }
}

#[derive(Bundle)]
struct CreatureBundle {
    spatial: SpatialBundle,
    parameters: Parameters,
    creature: Creature,
}

#[derive(Bundle)]
struct ParticleBundle {
    materialmesh2d: MaterialMesh2dBundle<ColorMaterial>,
    fields: Fields,
    particle: Particle,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_creature))
        .add_systems(Update, reset_fields.before(calculate_fields))
        .add_systems(Update, (bevy::window::close_on_esc, calculate_fields))
        .add_systems(Update, (update_position, update_size).after(calculate_fields))
        .run();
}

fn setup(
    mut commands: Commands,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 1.0 / 24.0;
    commands.spawn(camera);
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
        parameters: Parameters::default(),
        creature: Creature,
    }).with_children(|parent| {
        for _ in 0..199 {
            let r = 10.0 * rng.gen::<f32>().sqrt();
            let theta = rng.gen::<f32>() * TAU;

            parent.spawn(ParticleBundle {
                materialmesh2d: MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(0.5).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(r * theta.cos(), r * theta.sin(), 0.0)),
                    ..default()
                },
                fields: Fields::default(),
                particle: Particle,
            });
        }
    });
}

fn repulsion_field(r: f32, c_rep: f32) -> (f32, f32) {
    let t = f32::max(0.0, 1.0 - r);
    (0.5 * c_rep * t * t, -c_rep * t)
}

fn radial_field(x: f32, mu: f32, sigma: f32, w: f32) -> (f32, f32) {
    let t = (x - mu) / sigma;
    let y = w / (t * t).exp();
    (y, -2.0 * t * y / sigma)
}

fn reset_fields(
    mut particle_query: Query<(&Parent, &mut Fields), With<Particle>>,
    creature_query: Query<&Parameters, With<Creature>>,
) {
    particle_query.par_iter_mut().for_each_mut(|(parent, mut fields)| {
        let parameters = creature_query.get(parent.get()).unwrap();
        fields.R_val = repulsion_field(0.0, parameters.c_rep).0;
        fields.R_grad = Vec3::ZERO;
            
        fields.U_val = radial_field(0.0, parameters.mu_k, parameters.sigma_k, parameters.w_k).0;
        fields.U_grad = Vec3::ZERO;

        fields.E_grad = Vec3::ZERO;
    });
}

fn calculate_fields(
    creature_query: Query<(&Parameters, &Children), With<Creature>>,
    mut particle_query: Query<(&Transform, &mut Fields), With<Particle>>,
) {
    for (parameters, children) in creature_query.iter() {
        for (child_i, child_j) in children.iter()
            .enumerate()
            .flat_map(|(index, child)| iter::zip(
                iter::repeat(child),
                children.iter().skip(index + 1),
            ))
        {
            let [(transform_i, mut fields_i), (transform_j, mut fields_j)] = particle_query.get_many_mut([*child_i, *child_j]).unwrap();
            
            let r = transform_i.translation.distance(transform_j.translation);
            let r_grad = (transform_i.translation - transform_j.translation) / r;

            if r < 1.0 {
                let (R, dR) = repulsion_field(r, parameters.c_rep);
                fields_i.R_val += R;
                fields_j.R_val += R;
                fields_i.R_grad += r_grad * dR;
                fields_j.R_grad -= r_grad * dR;
            }

            let (K, dK) = radial_field(r, parameters.mu_k, parameters.sigma_k, parameters.w_k);
            fields_i.U_val += K;
            fields_j.U_val += K;
            fields_i.U_grad += r_grad * dK;
            fields_j.U_grad -= r_grad * dK;
        }

        for child in children.iter() {
            let (_, mut fields) = particle_query.get_mut(*child).unwrap();
            let (_, dG) = radial_field(fields.U_val, parameters.mu_g, parameters.sigma_g, 1.0);
            fields.E_grad = fields.R_grad - (dG * fields.U_grad);
        }
    }
}

fn update_position(
    time: Res<Time>,
    mut particle_query: Query<(&mut Transform, &Fields), With<Particle>>,
) {
    particle_query.par_iter_mut().for_each_mut(|(mut transform, fields)| {
        transform.translation += 0.1 * (-fields.E_grad);
    });
}

fn update_size(
    mut meshes: ResMut<Assets<Mesh>>,
    creature_query: Query<(&Parameters, &Children), With<Creature>>,
    particle_query: Query<(&Mesh2dHandle, &Fields), With<Particle>>,
) {
    for (parameters, children) in creature_query.iter() {
        for child in children.iter() {
            let (mesh, fields) = particle_query.get(*child).unwrap();
            let r = parameters.c_rep / (fields.R_val * 5.0);
            let _ = meshes.set(&mesh.0, shape::Circle::new(r).into());
        }
    }
}