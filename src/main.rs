use std::f32::consts::PI;

use bevy::{
    log::tracing_subscriber::{filter::targets, fmt::time},
    prelude::*,
    transform::{self, commands},
    utils::FloatOrd,
    window::WindowResolution,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Tower Defense".to_string(),
                resolution: WindowResolution::new(WIDTH, HEIGHT),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(WorldInspectorPlugin::new())
        .register_type::<Tower>()
        .register_type::<Lifetime>()
        .register_type::<Target>()
        .register_type::<Bullet>()
        .register_type::<Health>()
        .add_systems(Startup, spawn_basic_scene)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, asset_loading)
        .add_systems(Update, tower_shooting)
        .add_systems(Update, move_targets)
        .add_systems(Update, move_bullets)
        .add_systems(Update, bullet_despawn)
        .run();
}

fn asset_loading(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        bullet_scene: assets.load("Bullet.glb#Scene0"),
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn tower_shooting(
    mut commands: Commands,
    mut towers: Query<(Entity, &mut Tower, &GlobalTransform)>,
    targets: Query<&GlobalTransform, With<Target>>,
    bullet_assets: Res<GameAssets>,
    time: Res<Time>,
) {
    for (tower_ent, mut tower, transform) in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
            let bullet_spawn = transform.translation() + tower.bullet_offset;

            let direction = targets
                .iter()
                .min_by_key(|target_transform| {
                    FloatOrd(Vec3::distance(target_transform.translation(), bullet_spawn))
                })
                .map(|closest_target| closest_target.translation() - bullet_spawn);

            if let Some(direction) = direction {
                commands.entity(tower_ent).with_children(|commands| {
                    commands
                        .spawn(SceneBundle {
                            scene: bullet_assets.bullet_scene.clone(),
                            transform: Transform::from_translation(tower.bullet_offset),
                            ..Default::default()
                        })
                        .insert(Lifetime {
                            timer: Timer::from_seconds(1000.5, TimerMode::Once),
                        })
                        .insert(Bullet {
                            direction,
                            speed: 3.0,
                        });
                });
            }
        }
    }
}

fn spawn_basic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane {
                size: 5.0,
                subdivisions: 4,
            }),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..Default::default()
        })
        .insert(Name::new("Ground"));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.67, 0.84, 0.92)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Tower"))
        .insert(Tower {
            shooting_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            bullet_offset: Vec3::new(0.0, 0.2, 0.5),
        });

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1_500_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(Name::new("Light"));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.4, 0.4, 0.4)),
            material: materials.add(Color::rgb(0.67, 0.84, 0.92)),
            transform: Transform::from_xyz(-4.0, 0.2, 1.5),
            ..default()
        })
        .insert(Target { speed: 1.0 })
        .insert(Health { value: 3 })
        .insert(Name::new("Tagert"));
}

fn move_targets(mut targests: Query<(&Target, &mut Transform)>, time: Res<Time>) {
    for (target, mut transform) in &mut targests {
        transform.translation.x += target.speed * time.delta_seconds();
    }
}

fn bullet_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn move_bullets(mut bullets: Query<(&Bullet, &mut Transform)>, time: Res<Time>) {
    for (bullet, mut transform) in &mut bullets {
        transform.translation += bullet.direction.normalize() * bullet.speed * time.delta_seconds();
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    shooting_timer: Timer,
    bullet_offset: Vec3,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Target {
    speed: f32,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Health {
    value: i32,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Bullet {
    direction: Vec3,
    speed: f32,
}

#[derive(Resource)]
pub struct GameAssets {
    bullet_scene: Handle<Scene>,
}
