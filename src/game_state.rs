//! Demonstrates how to use the fly camera

extern crate amethyst;

use amethyst::assets::{Loader, Handle};
use amethyst::config::Config;
use amethyst::controls::{FlyControlBundle, FlyControlTag};
use amethyst::core::cgmath::{Deg, Vector3};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle, Parent};
use amethyst::ecs::{World, Component, NullStorage, Fetch, System, ReadStorage, WriteStorage, Join};
use amethyst::input::InputBundle;
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, ElementState, Event,
                         KeyboardInput, Light, Material, MaterialDefaults, MeshHandle, ObjFormat,
                         Pipeline, PointLight, PosNormTex, Projection, RenderBundle, Rgba, Stage,
                         VirtualKeyCode, WindowEvent, TextureHandle};
use amethyst::ui::{Anchor, Anchored, FontAsset, UiText, UiTransform, TtfFormat, UiBundle,
    Stretched, Stretch, DrawUi, UiImage};
use amethyst::utils::fps_counter::{FPSCounterBundle, FPSCounter};
use amethyst::{Application, Error, State, Trans};

struct ExampleState;

#[derive(Debug, Default)]
struct DebugDisplay;

impl Component for DebugDisplay {
    type Storage = NullStorage<Self>;
}

struct DebugSystem;

impl<'a> System<'a> for DebugSystem {
    type SystemData = (ReadStorage<'a, DebugDisplay>,
                       WriteStorage<'a, UiText>,
                       Fetch<'a, FPSCounter>);

    fn run(&mut self, (flag, mut text, counter): Self::SystemData) {
        for (_, text) in (&flag, &mut text).join() {
            text.text = format!("fps: {:.2}", counter.sampled_fps())
        }
    }
}

impl State for ExampleState {
    fn on_start(&mut self, world: &mut World) {
        initialise_camera(world);

        let assets = load_assets(&world);

        // Add cube to scene
        let mut trans = Transform::default();
        trans.translation = Vector3::new(0.0, 0.0, -5.0);
        world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.red.clone())
            .with(trans)
            .with(GlobalTransform::default())
            .build();
        // debug data
        let debug_background = world
            .create_entity()
            .with(UiTransform::new(
                "debug_background".to_string(),
                0.0,
                0.0,
                0.0,
                200.0,
                300.0,
                0,
            ))
            .with(UiImage {
                texture: assets.debug_background.clone(),
            })
            .with(Anchored::new(Anchor::TopLeft))
            .build();
        world
            .create_entity()
            .with(
                UiTransform::new("debug_data".to_string(), 100.0, 150.0, -1.0, 200.0, 300.0, 0)
                    .as_percent(),
            )
            .with(Stretched::new(Stretch::XY, 5.0, 5.0))
            .with(Parent {
                entity: debug_background
            })
            .with(UiText::new(
                assets.debug_font.clone(),
                "Hello world!".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                12.,
            ))
            .with(DebugDisplay)
            .build();

        let directional_light: Light = PointLight {
            center: [0.0; 3],
            color: Rgba::red(),
            ..Default::default()
        }.into();
        world.create_entity().with(directional_light).build();

        world.add_resource(AmbientColor(Rgba::from([0.1; 3])));
    }

    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match virtual_keycode {
                    Some(VirtualKeyCode::Escape) => return Trans::Quit,
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
        Trans::None
    }

    fn update(&mut self, world: &mut World) -> Trans {
        Trans::None
    }
}

struct Assets {
    cube: MeshHandle,
    red: Material,
    debug_background: TextureHandle,
    debug_font: Handle<FontAsset>,
}

fn load_assets(world: &World) -> Assets {
    let mesh_storage = world.read_resource();
    let tex_storage = world.read_resource();
    let font_storage = world.read_resource();
    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let red = loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), (), &tex_storage);
    let red = Material {
        albedo: red,
        ..mat_defaults.0.clone()
    };
    let debug_font = loader.load(
        "DroidSansMono.ttf",
        TtfFormat,
        Default::default(),
        (),
        &font_storage,
    );

    let debug_background = loader.load_from_data([0.0, 0.0, 0.0, 0.6].into(), (), &tex_storage);
    let cube = loader.load(
        "mesh/person_man_muscular.obj",
        ObjFormat,
        (),
        (),
        &mesh_storage,
    );

    Assets { cube, red, debug_background, debug_font }
}

const CARGO_MANIFEST_DIR: &'static str = env!("CARGO_MANIFEST_DIR");

/// Wrapper around the main, so we can return errors easily.
pub fn run() -> Result<(), Error> {
    use std::path::Path;

    let root_dir = if ::std::env::var("CARGO").is_ok() {
        Path::new(CARGO_MANIFEST_DIR)
    } else {
        Path::new(".")
    };
    let resources_directory = root_dir.join("resources");
    let display_config_path = resources_directory.join("display_config.ron");
    let key_bindings_path = resources_directory.join("input.ron");
    let display_config = DisplayConfig::load(display_config_path);


    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(DrawUi::new())
    );
    let mut game = Application::build(resources_directory, ExampleState)?
        .register::<DebugDisplay>()
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
        .with_bundle(FlyControlBundle::<String, String>::new(
            Some(String::from("move_x")),
            Some(String::from("move_y")),
            Some(String::from("move_z")),
        ))?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path),
        )?
        .with_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(FPSCounterBundle::default())?
        .with(DebugSystem, "debug_system", &[])
        .build()?;
    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    let local = Transform::default();

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(local)
        .with(GlobalTransform::default())
        .with(FlyControlTag)
        .build();
}
