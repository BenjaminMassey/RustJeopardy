#![allow(unused)]

use bevy::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;

struct WinSize {
    w: f32,
    h: f32,
}

struct TextObj;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 1.0)))
        .insert_resource(WindowDescriptor {
            title: "Jeopardy".to_string(),
            width: 1800.0,
            height: 1012.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
) {
    // Window setup
    let mut window = windows.get_primary_mut().unwrap();
    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height(),
    });
    window.set_position(IVec2::new(0, 0));

    // Cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Font
    let main_font = asset_server.load("korinan.ttf");

    // Make the title
    let title = gen_text(
        "JEOPARDY",
        Vec2::new(window.width() / 2., window.height() - 65.),
        main_font,
        100.0,
    );
    commands.spawn_bundle(title).insert(TextObj);
}

fn gen_text(s: &str, pos: Vec2, font: Handle<Font>, size: f32) -> TextBundle {
    return TextBundle {
        style: Style {
            align_self: AlignSelf::Center,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(pos.y - (size / 2.)), //Val::Px(5.0),
                right: Val::Px(pos.x - ((s.len() as f32 * (size / 2.)) / 2.)), //Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },

        text: Text::with_section(
            s,
            TextStyle {
                font: font,
                font_size: size,
                color: Color::WHITE,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
                ..Default::default()
            },
        ),
        ..Default::default()
    };
}
