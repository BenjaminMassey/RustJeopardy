#![allow(unused)]

use bevy::{
    input::mouse::{mouse_button_input_system, MouseButtonInput},
    prelude::*,
    reflect::TypeData,
    sprite::collide_aabb::collide,
};

const TIME_STEP: f32 = 1.0 / 60.0;

struct WinSize {
    w: f32,
    h: f32,
}

struct TextObj;
struct BoxObj;
struct ClueBox;

struct ClueText;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Jeopardy".to_string(),
            width: 1800.0,
            height: 1012.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(user_click.system())
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
    let mut x_values: Vec<f32> = vec![0., 0., 0., 0., 0., 0.];
    let mut n: f32 = 6. - 0.35;
    for i in 0..6 {
        x_values[i] = n as f32 * (window.width() / 6.);
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
        "MARIO JEOPARDY",
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
        "Category 6",
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

    let amounts: Vec<i32> = vec![200, 400, 600, 800, 1000];
    let mut y_index: usize = 2;
    for amount in &amounts {
        for i in 0..6 {
            let x: f32 = x_values[i];
            let y: f32 = y_values[y_index];
            let text = format!("${}", amount);
            let a: TextBundle = gen_text(
                &text.to_string(),
                Vec2::new(x, y),
                asset_server.load("korinan.ttf"),
                50.,
                Color::ORANGE,
            );
            commands.spawn_bundle(a).insert(TextObj);
        }
        y_index += 1;
    }

    let blue_box: SpriteBundle = SpriteBundle {
        material: materials.add((Color::BLUE).into()),
        sprite: Sprite::new(Vec2::new(250., 125.)),
        ..Default::default()
    };

    for i in 0..6 {
        for j in 1..7 {
            let mut new_box: SpriteBundle = blue_box.clone();

            //println!("{}", (window.width() / 2.));
            new_box.transform = Transform {
                translation: Vec3::new(
                    x_values[i] - (window.width() / 1.9), // idk why 1.9, just seems to work
                    y_values[j] - (window.height() / 2.),
                    10.,
                ),
                ..Default::default()
            };
            commands.spawn_bundle(new_box).insert(BoxObj);
        }
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

fn user_click(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mut box_query: Query<(Entity, &Transform, &Sprite, With<BoxObj>)>,
    mut text_query: Query<(Entity, With<TextObj>)>,
    mut clue_text_query: Query<(Entity, With<ClueText>)>,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let win = windows.get_primary().expect("No Window");
        let mouse_pos_raw: Vec2 = win.cursor_position().expect("No Mouse Pos");
        let mouse_pos: Vec2 = Vec2::new(
            mouse_pos_raw.x - (win.width() / 2.),
            mouse_pos_raw.y - (win.height() / 2.),
        );
        //println!("{}, {}", mouse_pos.x, mouse_pos.y);
        let mut i: i32 = 0;
        for (box_entity, box_tf, box_sprite, _) in box_query.iter_mut() {
            //println!("Box: {}", box_tf.translation);
            if ((i % 6) != 0
                && mouse_pos.x < box_tf.translation.x + (box_sprite.size.x / 2.)
                && mouse_pos.x > box_tf.translation.x - (box_sprite.size.x / 2.)
                && mouse_pos.y < box_tf.translation.y + (box_sprite.size.y / 2.)
                && mouse_pos.y > box_tf.translation.y - (box_sprite.size.y / 2.))
            {
                commands.entity(box_entity).despawn();
                /*
                let mut j: i32 = 1;
                for (text_entity, _) in text_query.iter_mut() {
                    //println!("j{}", j);
                    if (i == text_to_box_coords(j - 2)) {
                        commands.entity(text_entity).despawn();
                        break;
                    }
                    j += 1;
                }
                */
                for (clue_text_entity, _) in clue_text_query.iter_mut() {
                    commands.entity(clue_text_entity).despawn();
                }
                let clue_text: &str = get_clue(i);
                let clue: TextBundle = gen_text(
                    clue_text,
                    Vec2::new(win.width() / 2., win.height() / 2.),
                    asset_server.load("korinan.ttf"),
                    200.,
                    Color::WHITE,
                );
                commands.spawn_bundle(clue).insert(ClueText);
                break;
            }
            i += 1;
        }
    }
}

fn text_to_box_coords(n: i32) -> i32 {
    if (n < 0 || n > 35) {
        return -1;
    };
    let nums: [i32; 36] = [
        30, 24, 18, 12, 6, 0, 31, 25, 19, 13, 7, 1, 32, 26, 20, 14, 8, 2, 33, 27, 21, 15, 9, 3, 34,
        28, 22, 16, 10, 4, 35, 29, 23, 17, 11, 5,
    ];
    return nums[n as usize];
}

fn get_clue(index: i32) -> &'static str {
    let mut clues: [&str; 36] = [
        "CATEGORY 6",
        "C6 1",
        "C6 2",
        "C6 3",
        "C6 4",
        "C6 5",
        "CATEGORY 5",
        "C5 1",
        "C5 2",
        "C5 3",
        "C5 4",
        "C5 5",
        "CATEGORY 4",
        "C4 1",
        "C4 2",
        "C4 3",
        "C4 4",
        "C4 5",
        "CATEGORY 3",
        "C3 1",
        "C3 2",
        "C3 3",
        "C3 4",
        "C3 5",
        "CATEGORY 2",
        "C2 1",
        "C2 2",
        "C2 3",
        "C2 4",
        "C2 5",
        "CATEGORY 1",
        "C1 1",
        "C1 2",
        "C1 3",
        "C1 4",
        "C1 5",
    ];
    println!("{}", clues[index as usize]);
    return clues[index as usize];
}
