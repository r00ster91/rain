use bevy::{
    diagnostic::*,
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
    window::WindowResized,
};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .insert_resource(SmallRng::from_entropy())
        .insert_resource(SpawnDropTimer(Timer::from_seconds(0.0001, true)))
        .insert_resource(MoveDropTimer(Timer::from_seconds(0.0001, true)))
        .add_startup_system(setup.system())
        .add_system(spawn_drop.system())
        .add_system(make_drops_drop.system())
        .add_system(despawn_drops.system())
        .add_system(update_background.system())
        .add_asset::<Uniforms>()
        .run();
}

const VERTEX_SHADER: &str = r#"
#version 460
layout(location = 0) in vec3 Vertex_Position;
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 460
layout(location = 0) out vec4 o_Target;
layout(set = 2, binding = 0) uniform Uniforms_size {
    vec2 size;
};
void main() {
    vec2 position = gl_FragCoord.xy / size;

    vec4 top = vec4(1., 1., 1., 1.);
    vec4 bottom = vec4(0., 1., 1., 1.);

    o_Target = vec4(mix(bottom, top, position.y));
}
"#;

struct SpawnDropTimer(Timer);
struct MoveDropTimer(Timer);
struct Drop;
struct Background;

#[derive(RenderResources, TypeUuid)]
#[uuid = "5cea8a14-f045-4884-b833-1e616ddf29ac"]
struct Uniforms {
    pub size: Vec2,
}

fn setup(
    commands: &mut Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    windows: Res<Windows>,
    mut uniforms: ResMut<Assets<Uniforms>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    commands.spawn(OrthographicCameraBundle::new_2d());

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    render_graph.add_system_node("size", AssetRenderResourcesNode::<Uniforms>::new(true));

    render_graph
        .add_node_edge("size", base::node::MAIN_PASS)
        .unwrap();

    let window = windows.get_primary().unwrap();

    let uniform = uniforms.add(Uniforms {
        size: Vec2::new(window.width() / 2., window.height() / 2.),
    });

    commands
        .spawn(SpriteBundle {
            sprite: Sprite::new(Vec2::new(window.width() / 2., window.height() / 2.)),
            render_pipelines: RenderPipelines::from_handles(&vec![pipeline_handle]),
            transform: Transform::from_scale(Vec3::new(
                window.width() / 2.,
                window.height() / 2.,
                0.,
            )),
            ..Default::default()
        })
        .with(uniform)
        .with(Background);
}

fn spawn_drop(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: ResMut<SpawnDropTimer>,
    time: Res<Time>,
    mut rng: ResMut<SmallRng>,
    windows: Res<Windows>,
) {
    if timer.0.tick(time.delta_seconds()).just_finished() {
        let window = windows.get_primary().unwrap();
        for _ in 0..5 {
            let x = rng.gen_range((-window.width() / 2.)..(window.width() / 2.));
            let drop_height = rng.gen_range(25.0..75.);
            commands
                .spawn(SpriteBundle {
                    material: materials.add(Color::rgb(0.3, 0.3, 0.75).into()),
                    sprite: Sprite::new(Vec2::new(2., drop_height)),
                    transform: Transform {
                        translation: Vec3::new(x, window.height() / 2., 0.),
                        rotation: Quat::from_rotation_z(-0.1),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Drop);
        }
    }
}

fn despawn_drops(
    commands: &mut Commands,
    drops: Query<(Entity, &Transform), With<Drop>>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    for (entity, transform) in drops.iter() {
        if transform.translation.y < -window.height() / 2. {
            commands.despawn(entity);
        }
    }
}

fn make_drops_drop(
    mut drops: Query<&mut Transform, With<Drop>>,
    mut timer: ResMut<MoveDropTimer>,
    time: Res<Time>,
) {
    if timer.0.tick(time.delta_seconds()).just_finished() {
        for mut drop in drops.iter_mut() {
            drop.translation.y -= 50.;
            drop.translation.x -= 5.;
        }
    }
}

fn update_background(
    mut window_resized_events: EventReader<WindowResized>,
    mut background_query: Query<&mut Transform, With<Background>>,
    mut uniforms: ResMut<Assets<Uniforms>>,
) {
    for event in window_resized_events.iter() {
        for mut background in background_query.iter_mut() {
            background.scale.x = event.width;
            background.scale.y = event.height;
        }
        let ids = uniforms.ids().collect::<Vec<_>>();
        for id in ids {
            uniforms.set(
                id,
                Uniforms {
                    size: Vec2::new(event.width, event.height),
                },
            );
        }
    }
}
