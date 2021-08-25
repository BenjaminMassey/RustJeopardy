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
    //let mut main_font: Handle<Font> = asset_server.load("korinan.ttf");

    // Set up coordinate values
    let mut x_values: Vec<f32> = vec![0., 0., 0., 0., 0.];
    let mut n: f32 = 5. - 0.35;
    for i in 0..5 {
        x_values[i] = n as f32 * (window.width() / 5.);
        n -= 1.
    }

    let mut y_values: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0.];
    let mut n: f32 = 7. - 0.35;
    for i in 0..7 {
        y_values[i] = n as f32 * (window.height() / 7.);
        n -= 1.
    }

    // Make the title
    let title = gen_text(
        "JEOPARDY",
        Vec2::new(window.width() / 2., y_values[0]),
        asset_server.load("korinan.ttf"),
        100.0,
        Color::YELLOW,
    );
    commands.spawn_bundle(title).insert(TextObj);

    // Make the categories
    let categories: Vec<&str> = vec![
        "Category 1",
        "Category 2",
        "Category 3",
        "Category 4",
        "Category 5",
    ];

    let mut index: usize = 0;
    for category in &categories {
        let x: f32 = x_values[index];
        let y: f32 = y_values[1];
        let cat: TextBundle = gen_text(
            category,
            Vec2::new(x, y),
            asset_server.load("korinan.ttf"),
            50.,
            Color::WHITE,
        );
        commands.spawn_bundle(cat).insert(TextObj);
        index += 1;
    }

    let amounts: Vec<&str> = vec!["$400", "$800", "$1200", "$1600", "$2000"];
    let mut y_index: usize = 2;
    for amount in &amounts {
        for i in 0..5 {
            let x: f32 = x_values[i];
            let y: f32 = y_values[y_index];
            let a: TextBundle = gen_text(
                amount,
                Vec2::new(x, y),
                asset_server.load("korinan.ttf"),
                50.,
                Color::ORANGE,
            );
            commands.spawn_bundle(a).insert(TextObj);
        }
        y_index += 1;
    }
}

fn gen_text(s: &str, pos: Vec2, font: Handle<Font>, size: f32, color: Color) -> TextBundle {
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
                color: color,
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
