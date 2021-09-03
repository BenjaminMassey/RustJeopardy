#![allow(unused)]

use bevy::{
    input::mouse::{mouse_button_input_system, MouseButtonInput},
    prelude::*,
    reflect::TypeData,
    sprite::collide_aabb::collide,
    text,
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

struct ReadingClue(bool);

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Jeopardy".to_string(),
            width: 1800.0,
            height: 1012.0,
            ..Default::default()
        })
        .insert_resource(ReadingClue(false))
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
        Vec2::new((window.width() / 2.) - 350., y_values[0] - 60.), // arbitrary subtractions for positioning: BAD
        asset_server.load("korinan.ttf"),
        100.0,
        Color::YELLOW,
    );
    commands.spawn_bundle(title).insert(TextObj);

    // Make the categories
    let categories: Vec<&str> = vec![
        "Game\nWorlds",
        "Classic\nEnemies",
        "Before\n& After",
        "Koopa\nthe Quick",
        "Technical\nJunk",
        "Historical\nFacts",
    ];

    let mut index: usize = 0;
    for category in &categories {
        let mut x: f32 = x_values[index];
        let y: f32 = y_values[1];
        match index {
            // arbitrary addition for positioning: BAD
            0 | 2 => x += 20.,
            1 => x += 10.,
            _ => (),
        }
        let cat: TextBundle = gen_text(
            category,
            Vec2::new(x - 125., y - 50.), // arbitrary subtractions for positioning: BAD
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
                Vec2::new(x - 85., y - 20.), // arbitrary subtractions for positioning: BAD
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
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(pos.y), // - (size / 2.)), //Val::Px(5.0),
                right: Val::Px(pos.x), // - ((s.len() as f32 * (size / 2.)) / 2.)), //Val::Px(15.0),
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
    mut text_query: Query<(Entity, &mut Style, With<TextObj>)>,
    mut clue_box_query: Query<(Entity, With<ClueBox>)>,
    mut clue_text_query: Query<(Entity, With<ClueText>)>,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut reading: ResMut<ReadingClue>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if (reading.0) {
            for (clue_text_entity, _) in clue_text_query.iter_mut() {
                commands.entity(clue_text_entity).despawn();
            }
            for (clue_box_entity, _) in clue_box_query.iter_mut() {
                commands.entity(clue_box_entity).despawn();
            }
            let mut text_iter: i32 = 0;
            for (text_entity, mut text_style, _) in text_query.iter_mut() {
                if (text_iter < 7) {
                    // To keep categories + title unmoved
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
            for (box_entity, box_tf, box_sprite, _) in box_query.iter_mut() {
                //println!("Box: {}", box_tf.translation);
                if ((i % 6) != 0
                    && mouse_pos.x < box_tf.translation.x + (box_sprite.size.x / 2.)
                    && mouse_pos.x > box_tf.translation.x - (box_sprite.size.x / 2.)
                    && mouse_pos.y < box_tf.translation.y + (box_sprite.size.y / 2.)
                    && mouse_pos.y > box_tf.translation.y - (box_sprite.size.y / 2.))
                {
                    commands.entity(box_entity).despawn();
                    /* TODO: this just doesn't work as wanted
                    let mut j: i32 = 0;
                    for (text_entity, mut text_style, _) in text_query.iter_mut() {
                        //println!("j{}", j);
                        if (i == text_to_box_coords(j)) {
                            commands.entity(text_entity).despawn();
                            break;
                        }
                        j += 1;
                    }
                    */
                    let mut clue_box = SpriteBundle {
                        material: materials.add((Color::MIDNIGHT_BLUE).into()),
                        sprite: Sprite::new(Vec2::new(800., 600.)),
                        ..Default::default()
                    };
                    clue_box.transform = Transform {
                        translation: Vec3::new(0., -100., 15.),
                        ..Default::default()
                    };
                    commands.spawn_bundle(clue_box).insert(ClueBox);

                    let clue_text: &str = get_clue(i);
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
                    commands.spawn_bundle(clue).insert(ClueText);
                    let mut text_iter: i32 = 0;
                    for (text_entity, mut text_style, _) in text_query.iter_mut() {
                        if (text_iter < 7) {
                            // To keep categories + title unmoved
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

/* TODO: part of broken code above
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
*/

fn get_clue(index: i32) -> &'static str {
    // https://docs.google.com/document/d/1JXFZT8TP8WhSkEa_iMHrfNA1zcW5KImH34A5IyVG3NU/edit?usp=sharing
    let mut clues: [&str; 36] = [
        "<<<Historical Facts>>>",
        "Mario’s original name from his\ndebut in the arcade game\nDonkey Kong.\n \n \n \n ",
        "This game introduces Yoshi as\na character.\n \n \n \n \n ",
        "This Nintendo console featured the\nfirst entry in the Mario\nKart series.\n \n \n \n ",
        "The creator of Mario.\n \n \n \n \n \n ",
        "Super Mario Bros. was released on\nthis year for the Nintendo\nEntertainment System (NES).\n \n \n \n ",
        "<<<Technical Junk>>>",
        "While the Nintendo 64 gives away\nits amount of bits, this\nis the bit number for the Super\nNintendo (SNES).\n \n \n ",
        "Super Mario 64 features this gimmick\non startup, which was made\nas a technical demo of the\nNintendo 64’s advanced 3D capabilities.\n \n \n ",
        "The Nintendo GameCube was named\nafter this animal during its\ndevelopment at Nintendo. This\ncan be seen in reference in its\ncomponents, and is also the namesake\nof the popular GameCube and Wii\nEmulator.\n ",
        "In Super Mario 64, this boss character\nshares the same audio\nfile as Bowser, only with a\ndiffering playback speed.\n \n \n ",
        "This graphical technique for the Super\nNintendo was used by early\ngames like Super Mario Kart to\nsimulate 3D long before 3D became\na truly viable option.\n \n \n ",
        "<<<Koopa the Quick>>>",
        "The game version featuring this\nlanguage is the fastest for\nspeedrunning most speedrun categories\nin Super Mario 64.\n \n \n ",
        "During the primary boss battles\nin Super Mario 64, the player must\nperform this action on Bowser in\norder to defeat him. It is also a\nfrequent action taken by\nfamous runner Clint Stevens.\n ",
        "The fastest character for speedrunning\nin Super Mario Galaxy. Also a\nslippery and silly individual.\n \n \n \n ",
        "This category of Super Mario Odyssey\nspeedrunning involves runners\nattempting to capture enemies as\nlittle as physically possible. It\nis also a generic speedrunning\ncategory used in many different games.\n ",
        "This speedrunner is the current world\nrecord holder for the “120 Star”\ncategory on speedrun.com.\n \n \n \n ",
        "<<<BEFORE & AFTER>>>",
        "Bowser’s royal name, followed by a\nstandard shelled enemy.\n \n \n \n \n ",
        "The title of the original Mario game\nfor the Nintendo Entertainment System,\nfollowed by a mantra said\namong frat boys.\n \n \n ",
        "The RPG series title highlighting\nthe iconic plumber duo, followed by the\ngreen partner’s GameCube\ndebut title.\n \n \n ",
        "An obscure drawing game for the Super\nNintendo, followed by the main tool\nused by Bowser Jr.\n \n \n \n ",
        "The Nintendo Switch RPG collaboration\nwith the Rayman franchise, followed\nby the most popular\nSpongebob Squarepants 3D platformer game\n(recently remade for current\nera consoles).\n ",
        "<<<ENEMIES>>>",
        "This the most standard ground-based\nenemy in the Mario universe: iconic\nfor being grumpy, brown,\nand armless.\n \n \n ",
        "A lead villain in Super Mario Sunshine,\nthis character is famous for\ntheir flying vehicle and\ntheir infamous father.\n \n \n ",
        "Famous for being shy around any hero,\nthis enemy typically cannot be\nkilled by traditional means.\n \n \n \n ",
        "This magician often flies around and\ncauses mischief for the heroes.\nHe is also frequently portrayed\nas a right hand man to Bowser himself.\n \n \n ",
        "Debuting in Super Mario 64 but being\nseen in many future games, this\nunderwater creature is famous for\nscaring children and poking its long\nhead out of holes in the wall.\n \n ",
        "<<<GAME WORLDS>>>",
        "Vast in size and sub-areas, this Super\nMario Odyssey world features a\npyramid that floats into the air.\n \n \n \n ",
        "This hub world in Super Mario Sunshine\nfeatures a tropical paradise\nwith lively natives.\n \n \n \n ",
        "There are this many worlds in the\nfirst Mario game for the Nintendo\nEntertainment System.\n \n \n \n ",
        "This world is the first one\naccessible in Super Mario Galaxy.\n \n \n \n \n ",
        "This world in Super Mario 64 was\nso beloved that it was recreated\nin Super Mario Galaxy 2. It also features\na rather tall and skinny boss.\n \n \n ",
    ];
    //println!("{}", clues[index as usize]);
    return clues[index as usize];
}
