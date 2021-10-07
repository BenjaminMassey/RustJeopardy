mod quiz;
use quiz::*;

use bevy::prelude::*;

struct TextObj;
struct BoxObj;
struct ClueBox;
struct ClueText;
struct ReadingClue(bool);

fn main() {
    let quiz = Quiz::new("assets/quiz.xml").unwrap();
    App::build()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Jeopardy".to_string(),
            width: 1800.0,
            height: 1012.0,
            ..Default::default()
        })
        .insert_resource(ReadingClue(false))
        .insert_resource(quiz)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(user_click.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    quiz: Res<Quiz>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
) {
    // Window setup
    let window = windows.get_primary_mut().unwrap();
    window.set_position(IVec2::new(0, 0));

    // Cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Set up coordinate values
    let mut x_values: Vec<f32> = vec![0., 0., 0., 0., 0., 0.];
    let nx = x_values.len() as f32;
    let n = nx - 0.35;
    for (i, x) in x_values.iter_mut().enumerate() {
        *x = (n - i as f32) * (window.width() / nx);
    }

    let mut y_values: Vec<f32> = vec![0., 0., 0., 0., 0., 0., 0.];
    let ny = y_values.len() as f32;
    let n = ny - 0.35;
    for (i, y) in y_values.iter_mut().enumerate() {
        *y = (n - i as f32) * (window.height() / ny);
    }

    // Make the title
    let title = gen_text(
        (&quiz.name).as_deref().unwrap_or("Quiz!"),
        // arbitrary subtractions for positioning: BAD
        Vec2::new((window.width() / 2.) - 350., y_values[0] - 60.),
        asset_server.load("korinan.ttf"),
        100.0,
        Color::YELLOW,
    );
    commands.spawn_bundle(title).insert(TextObj);

    for (index, category) in quiz.category.iter().enumerate() {
        let mut x: f32 = x_values[index];
        let y: f32 = y_values[1];
        match index {
            // arbitrary addition for positioning: BAD
            0 | 2 => x += 20.,
            1 => x += 10.,
            _ => (),
        }
        let cat: TextBundle = gen_text(
            &category.name,
            // arbitrary subtractions for positioning: BAD
            Vec2::new(x - 125., y - 50.),
            asset_server.load("korinan.ttf"),
            50.,
            Color::WHITE,
        );
        commands.spawn_bundle(cat).insert(TextObj);
    }

    let amounts: Vec<i32> = vec![200, 400, 600, 800, 1000];
    let mut y_index: usize = 2;
    for amount in &amounts {
        for &x in &x_values {
            let y: f32 = y_values[y_index];
            let text = format!("${}", amount);
            let a: TextBundle = gen_text(
                &text.to_string(),
                // arbitrary subtractions for positioning: BAD
                Vec2::new(x - 85., y - 20.),
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

    for &x in &x_values {
        for &y in &y_values[1..] {
            let mut new_box: SpriteBundle = blue_box.clone();
            new_box.transform = Transform {
                translation: Vec3::new(
                    x - (window.width() / 1.9), // idk why 1.9, just seems to work
                    y - (window.height() / 2.),
                    10.,
                ),
                ..Default::default()
            };
            commands.spawn_bundle(new_box).insert(BoxObj);
        }
    }
}

fn gen_text(s: &str, pos: Vec2, font: Handle<Font>, font_size: f32, color: Color) -> TextBundle {
    TextBundle {
        style: Style {
            align_self: AlignSelf::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(pos.y),
                right: Val::Px(pos.x),
                ..Default::default()
            },
            ..Default::default()
        },

        text: Text::with_section(
            s,
            TextStyle {
                font,
                font_size,
                color,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
            },
        ),
        ..Default::default()
    }
}

#[allow(clippy::too_many_arguments)]
fn user_click(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mut box_query: Query<(Entity, &mut Transform, &Sprite, With<BoxObj>)>,
    mut text_query: Query<(Entity, &mut Style, With<TextObj>)>,
    mut clue_box_query: Query<(Entity, With<ClueBox>)>,
    mut clue_text_query: Query<(Entity, With<ClueText>)>,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    quiz: Res<Quiz>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut reading: ResMut<ReadingClue>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if reading.0 {
            for (clue_text_entity, _) in clue_text_query.iter_mut() {
                commands.entity(clue_text_entity).despawn();
            }
            for (clue_box_entity, _) in clue_box_query.iter_mut() {
                commands.entity(clue_box_entity).despawn();
            }
            let mut text_iter: i32 = 0;
            for (_, mut text_style, _) in text_query.iter_mut() {
                if text_iter < 7 {
                    // To keep category + title unmoved
                    text_iter += 1;
                    continue;
                }
                let new_bottom: Val = text_style.position.bottom + (-5000.);
                let new_right: Val = text_style.position.right + (-5000.);
                text_style.position = Rect {
                    bottom: new_bottom,
                    right: new_right,
                    ..Default::default()
                }
            }
            reading.0 = !reading.0;
        } else {
            let win = windows.get_primary().expect("No Window");
            let mouse_pos_raw: Vec2 = win.cursor_position().expect("No Mouse Pos");
            let mouse_pos: Vec2 = Vec2::new(
                mouse_pos_raw.x - (win.width() / 2.),
                mouse_pos_raw.y - (win.height() / 2.),
            );
            //println!("{}, {}", mouse_pos.x, mouse_pos.y);
            let mut i: i32 = 0;
            for (_, mut box_tf, box_sprite, _) in box_query.iter_mut() {
                //println!("Box: {}", box_tf.translation);
                if (i % 6) != 0
                    && mouse_pos.x < box_tf.translation.x + (box_sprite.size.x / 2.)
                    && mouse_pos.x > box_tf.translation.x - (box_sprite.size.x / 2.)
                    && mouse_pos.y < box_tf.translation.y + (box_sprite.size.y / 2.)
                    && mouse_pos.y > box_tf.translation.y - (box_sprite.size.y / 2.)
                {
                    // Move out of way rather than despawn because of future iteration
                    box_tf.translation = Vec3::new(9000., 9000., 15.);

                    for (j, (_, mut text_style, _)) in text_query.iter_mut().enumerate() {
                        //println!("j{}", j);
                        if Some(i) == text_to_box_coords(j as i32 - 1) {
                            // Move out of way rather than despawn because of future iteration
                            let new_bottom: Val = text_style.position.bottom + 5000.;
                            let new_right: Val = text_style.position.right + 5000.;
                            text_style.position = Rect {
                                bottom: new_bottom,
                                right: new_right,
                                ..Default::default()
                            };
                            break;
                        }
                    }

                    let mut clue_box = SpriteBundle {
                        material: materials.add((Color::MIDNIGHT_BLUE).into()),
                        sprite: Sprite::new(Vec2::new(800., 320.)),
                        ..Default::default()
                    };
                    clue_box.transform = Transform {
                        translation: Vec3::new(0., -10., 15.),
                        ..Default::default()
                    };
                    commands.spawn_bundle(clue_box).insert(ClueBox);

                    let clue_text: &str = quiz.get_clue(i as usize);
                    let clue: TextBundle = gen_text(
                        &clue_text,
                        Vec2::new(
                            (win.width() / 2.) - 350.,
                            ((win.height() / 2.) - 80.) - 125.,
                        ), // arbitrary subtractions for positioning: BAD
                        asset_server.load("korinan.ttf"),
                        50.,
                        Color::WHITE,
                    );
                    commands.spawn_bundle(clue).insert(ClueText);
                    let mut text_iter: i32 = 0;
                    for (_, mut text_style, _) in text_query.iter_mut() {
                        if text_iter < 7 {
                            // To keep category + title unmoved:
                            // genuinely optional, but I like it
                            text_iter += 1;
                            continue;
                        }
                        let new_bottom: Val = text_style.position.bottom + 5000.;
                        let new_right: Val = text_style.position.right + 5000.;
                        text_style.position = Rect {
                            bottom: new_bottom,
                            right: new_right,
                            ..Default::default()
                        };
                    }

                    reading.0 = !reading.0;

                    break;
                }
                i += 1;
            }
        }
    }
}

fn text_to_box_coords(n: i32) -> Option<i32> {
    if (0..=35).contains(&n) {
        Some(6 * (5 - n % 6) + n / 6)
    } else {
        None
    }
}
