#![allow(unused)]

mod quiz;

use std::{path::PathBuf, process::ExitCode};

use quiz::Quiz;

use bevy::{
    input::mouse::{mouse_button_input_system, MouseButtonInput},
    prelude::*,
    reflect::TypeData,
    sprite::collide_aabb::collide,
    text,
    window::{PrimaryWindow, WindowPosition},
};

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Resource)]
struct WinSize {
    w: f32,
    h: f32,
}

#[derive(Component)]
/// The box index for board text, or `None` for the title.
struct TextObj(Option<i32>);
#[derive(Component)]
struct BoxObj(usize);
#[derive(Component)]
struct ClueBox;

#[derive(Component)]
struct ClueText;

#[derive(Resource)]
struct ReadingClue(bool);

const DEFAULT_QUIZ: &str = "assets/gibli-quiz.xml";
const USAGE: &str = "Usage: jeopardy [QUIZ.xml]";

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let quiz_path = match (args.next(), args.next()) {
        (None, _) => PathBuf::from(DEFAULT_QUIZ),
        (Some(arg), None) if arg == "--help" || arg == "-h" => {
            println!("{USAGE}\nDefault quiz: {DEFAULT_QUIZ}");
            return ExitCode::SUCCESS;
        }
        (Some(arg), None) => PathBuf::from(arg),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    let quiz = match Quiz::load(&quiz_path) {
        Ok(quiz) => quiz,
        Err(err) => {
            eprintln!("Could not load quiz {}: {err}", quiz_path.display());
            return ExitCode::FAILURE;
        }
    };
    App::new()
        .insert_resource(quiz)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ReadingClue(false))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Jeopardy".to_string(),
                resolution: (1422.0_f32, 800.0_f32).into(),
                position: WindowPosition::At(IVec2::ZERO),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_startup_system(setup)
        .add_system(user_click)
        .run();
    ExitCode::SUCCESS
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
    quiz: Res<Quiz>,
) {
    // Window setup
    let window = windows.single();
    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height(),
    });

    // Camera (renders both sprites and UI)
    commands.spawn(Camera2dBundle::default());

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
        &quiz.name,
        Vec2::new((window.width() / 2.) - 350., y_values[0] - 60.), // arbitrary subtractions for positioning: BAD
        asset_server.load("korinan.ttf"),
        100.0,
        Color::YELLOW,
    );
    commands.spawn(title).insert(TextObj(None));

    // Make the categories
    let box_size = Vec2::new(225., 100.);
    let board_x_offset = window.width() / 1.9;
    for (index, category) in quiz.categories.iter().enumerate() {
        // Categories are in left-to-right order; boxes are right-to-left.
        let center_x = x_values[5 - index] - board_x_offset + window.width() / 2.;
        let right = window.width() - center_x - box_size.x / 2.;
        let bottom = y_values[1] - box_size.y / 2.;
        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        right: Val::Px(right),
                        bottom: Val::Px(bottom),
                        ..Default::default()
                    },
                    size: Size::new(Val::Px(box_size.x), Val::Px(box_size.y)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(TextObj(Some(text_to_box_coords(index as i32))))
            .with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(
                        &category.name,
                        TextStyle {
                            font: asset_server.load("korinan.ttf"),
                            font_size: 50.,
                            color: Color::WHITE,
                        },
                    )
                    .with_text_alignment(TextAlignment::Center),
                );
            });
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
                Vec2::new(x - 85., y - 20.), // arbitrary subtractions for positioning: BAD
                asset_server.load("korinan.ttf"),
                50.,
                Color::ORANGE,
            );
            let box_index = text_to_box_coords(((y_index - 1) * 6 + i) as i32);
            commands.spawn(a).insert(TextObj(Some(box_index)));
        }
        y_index += 1;
    }

    let blue_box: SpriteBundle = SpriteBundle {
        sprite: Sprite {
            color: Color::BLUE,
            custom_size: Some(box_size),
            ..Default::default()
        },
        ..Default::default()
    };

    for i in 0..6 {
        for j in 1..7 {
            let mut new_box: SpriteBundle = blue_box.clone();
            new_box.transform = Transform {
                translation: Vec3::new(
                    x_values[i] - board_x_offset, // idk why 1.9, just seems to work
                    y_values[j] - (window.height() / 2.),
                    10.,
                ),
                ..Default::default()
            };
            commands.spawn(new_box).insert(BoxObj(i * 6 + j - 1));
        }
    }
}

fn gen_text(s: &str, pos: Vec2, font: Handle<Font>, size: f32, color: Color) -> TextBundle {
    return TextBundle {
        style: Style {
            align_self: AlignSelf::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(pos.y), // - (size / 2.)), //Val::Px(5.0),
                right: Val::Px(pos.x), // - ((s.len() as f32 * (size / 2.)) / 2.)), //Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },

        text: Text::from_section(
            s,
            TextStyle {
                font: font,
                font_size: size,
                color: color,
            },
        )
        .with_alignment(TextAlignment::Center),
        ..Default::default()
    };
}

fn user_click(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mut box_query: Query<(&BoxObj, &mut Transform, &Sprite)>,
    mut text_query: Query<(&mut Style, &TextObj)>,
    clue_box_query: Query<Entity, With<ClueBox>>,
    clue_text_query: Query<Entity, With<ClueText>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut reading: ResMut<ReadingClue>,
    quiz: Res<Quiz>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if (reading.0) {
            for clue_text_entity in clue_text_query.iter() {
                commands.entity(clue_text_entity).despawn();
            }
            for clue_box_entity in clue_box_query.iter() {
                commands.entity(clue_box_entity).despawn();
            }
            for (mut text_style, text_obj) in text_query.iter_mut() {
                if text_obj.0.is_none() {
                    // Keep the title at the top.
                    continue;
                }
                text_style
                    .position
                    .bottom
                    .try_sub_assign(Val::Px(5000.))
                    .unwrap();
                text_style
                    .position
                    .right
                    .try_sub_assign(Val::Px(5000.))
                    .unwrap();
            }
            reading.0 = !reading.0;
        } else {
            let Ok(win) = windows.get_single() else {
                return;
            };
            let Some(mouse_pos_raw) = win.cursor_position() else {
                return;
            };
            let mouse_pos: Vec2 = Vec2::new(
                mouse_pos_raw.x - (win.width() / 2.),
                mouse_pos_raw.y - (win.height() / 2.),
            );
            //println!("{}, {}", mouse_pos.x, mouse_pos.y);
            for (box_obj, mut box_tf, box_sprite) in box_query.iter_mut() {
                let Some(clue_text) = quiz.clue_for_box(box_obj.0) else {
                    continue;
                };
                let i = box_obj.0 as i32;
                //println!("Box: {}", box_tf.translation);
                let box_size = box_sprite.custom_size.unwrap();
                if (mouse_pos.x < box_tf.translation.x + (box_size.x / 2.)
                    && mouse_pos.x > box_tf.translation.x - (box_size.x / 2.)
                    && mouse_pos.y < box_tf.translation.y + (box_size.y / 2.)
                    && mouse_pos.y > box_tf.translation.y - (box_size.y / 2.))
                {
                    // Move out of way rather than despawn because of future iteration
                    box_tf.translation = Vec3::new(9000., 9000., 15.);

                    for (mut text_style, text_obj) in text_query.iter_mut() {
                        if text_obj.0 == Some(i) {
                            // Move out of way rather than despawn because of future iteration
                            text_style
                                .position
                                .bottom
                                .try_add_assign(Val::Px(5000.))
                                .unwrap();
                            text_style
                                .position
                                .right
                                .try_add_assign(Val::Px(5000.))
                                .unwrap();
                            break;
                        }
                    }

                    let mut clue_box = SpriteBundle {
                        sprite: Sprite {
                            color: Color::MIDNIGHT_BLUE,
                            custom_size: Some(Vec2::new(1200., 620.)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    clue_box.transform = Transform {
                        translation: Vec3::new(0., -10., 15.),
                        ..Default::default()
                    };
                    commands.spawn(clue_box).insert(ClueBox);

                    let clue: TextBundle = gen_text(
                        clue_text,
                        Vec2::new(
                            (win.width() / 2.) - 350.,
                            ((win.height() / 2.) - 80.) - 125.,
                        ), // arbitrary subtractions for positioning: BAD
                        asset_server.load("korinan.ttf"),
                        50.,
                        Color::WHITE,
                    );
                    commands.spawn(clue).insert(ClueText);
                    for (mut text_style, text_obj) in text_query.iter_mut() {
                        if text_obj.0.is_none() {
                            // Keep the title at the top.
                            continue;
                        }
                        text_style
                            .position
                            .bottom
                            .try_add_assign(Val::Px(5000.))
                            .unwrap();
                        text_style
                            .position
                            .right
                            .try_add_assign(Val::Px(5000.))
                            .unwrap();
                    }

                    reading.0 = !reading.0;

                    break;
                }
            }
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
