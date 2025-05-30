use bevy::{
    math::{vec2, vec4},
    prelude::*,
    render::{camera::ScalingMode, render_resource::AsBindGroup},
    sprite::{Material2d, Material2dPlugin},
};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use noisy_bevy::{NoisyShaderPlugin, simplex_noise_2d_seeded};

fn main() {
    App::new()
        .register_type::<AsteroidParams>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins,
            NoisyShaderPlugin,
            PanCamPlugin,
            Material2dPlugin::<AsteroidBackgroundMaterial>::default(),
            // WorldInspectorPlugin::new()
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, expand_asteroids)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 70.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        PanCam::default(),
    ));

    commands.spawn(AsteroidBundle::default());
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct AsteroidBackgroundMaterial {
    #[uniform(0)]
    params: Vec4,
}

impl Material2d for AsteroidBackgroundMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "examples/asteroid_background.wgsl".into()
    }
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "examples/asteroid_background.wgsl".into()
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
struct AsteroidParams {
    frequency_scale: f32,
    amplitude_scale: f32,
    radius: f32,
    seed: u32,
}

impl Default for AsteroidParams {
    fn default() -> Self {
        Self {
            frequency_scale: 0.1,
            amplitude_scale: 2.8,
            radius: 14.0,
            seed: 0,
        }
    }
}

#[derive(Bundle)]
struct AsteroidBundle {
    name: Name,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    view_visibility: ViewVisibility,
    inherited_visibility: InheritedVisibility,
    params: AsteroidParams,
}

impl Default for AsteroidBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Asteroid"),
            transform: default(),
            global_transform: default(),
            visibility: default(),
            params: default(),
            view_visibility: default(),
            inherited_visibility: default(),
        }
    }
}

// turns compact model representation into something we can see on screen
fn expand_asteroids(
    changed_asteroids: Query<(Entity, &AsteroidParams), Changed<AsteroidParams>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut asteroid_materials: ResMut<Assets<AsteroidBackgroundMaterial>>,
) {
    for (asteroid_entity, params) in changed_asteroids.iter() {
        let max_half_size = params.radius as i32 + 1;

        commands
            .entity(asteroid_entity)
            .despawn_related::<Children>();
        commands.entity(asteroid_entity).with_children(|asteroid| {
            for x in -max_half_size..=max_half_size {
                for y in -max_half_size..=max_half_size {
                    let p = vec2(x as f32, y as f32);
                    let o = simplex_noise_2d_seeded(p * params.frequency_scale, params.seed as f32)
                        * params.amplitude_scale;
                    // let o = noisy_bevy::fbm_simplex_2d(p * params.frequency_scale, 3, 2., 0.5)
                    //     * params.amplitude_scale;
                    if ((x * x + y * y) as f32) < (params.radius + o).powi(2) {
                        asteroid.spawn((
                            Sprite {
                                color: Color::WHITE.with_luminance(0.2),
                                custom_size: Some(Vec2::splat(1.)),
                                ..default()
                            },
                            Transform::from_translation(Vec3::new(x as f32, y as f32, 100.)),
                        ));
                    }
                }
            }

            // we are making a new material each time we make an asteroid
            // this doesn't really scale well, but works fine for an example
            let material_handle = asteroid_materials.add(AsteroidBackgroundMaterial {
                params: vec4(
                    params.frequency_scale,
                    params.amplitude_scale,
                    params.radius,
                    params.seed as f32,
                ),
            });

            let quad_handle = meshes.add(Mesh::from(Rectangle::from_size(Vec2::new(100.0, 100.0))));

            asteroid.spawn((
                Mesh2d(quad_handle),
                MeshMaterial2d(material_handle),
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
            ));
        });
    }
}
