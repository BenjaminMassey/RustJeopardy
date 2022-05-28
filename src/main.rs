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
            width: 1422.0,
            height: 800.0,
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
        "GHIBLI JEOPARDY",
        Vec2::new((window.width() / 2.) - 350., y_values[0] - 60.), // arbitrary subtractions for positioning: BAD
        asset_server.load("korinan.ttf"),
        100.0,
        Color::YELLOW,
    );
    commands.spawn_bundle(title).insert(TextObj);

    // Make the categories
    let categories: Vec<&str> = vec![
        "General\nMovies",
        "Humans\n(Mostly)",
        "Creatures\n& Animals",
        "Historical\nKnowledge",
        "Scary\nVillains",
        "Random\n& Misc",
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
        sprite: Sprite::new(Vec2::new(225., 100.)),
        //sprite: Sprite::new(Vec2::new(250., 125.)),
        ..Default::default()
    };

    for i in 0..6 {
        for j in 1..7 {
            let mut new_box: SpriteBundle = blue_box.clone();
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
    mut box_query: Query<(Entity, &mut Transform, &Sprite, With<BoxObj>)>,
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
                /*
                if (text_iter < 7) {
                    // To keep categories + title unmoved
                    text_iter += 1;
                    continue;
                }
                */
                if (text_iter == 0) {
                    // To keep title at top, optional as well
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
            for (box_entity, mut box_tf, box_sprite, _) in box_query.iter_mut() {
                //println!("Box: {}", box_tf.translation);
                if ((i % 6) != 0
                    && mouse_pos.x < box_tf.translation.x + (box_sprite.size.x / 2.)
                    && mouse_pos.x > box_tf.translation.x - (box_sprite.size.x / 2.)
                    && mouse_pos.y < box_tf.translation.y + (box_sprite.size.y / 2.)
                    && mouse_pos.y > box_tf.translation.y - (box_sprite.size.y / 2.))
                {
                    // Move out of way rather than despawn because of future iteration
                    box_tf.translation = Vec3::new(9000., 9000., 15.);

                    let mut j: i32 = 0;
                    for (text_entity, mut text_style, _) in text_query.iter_mut() {
                        //println!("j{}", j);
                        if (i == text_to_box_coords(j - 1)) {
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
                        j += 1;
                    }

                    let mut clue_box = SpriteBundle {
                        material: materials.add((Color::MIDNIGHT_BLUE).into()),
                        sprite: Sprite::new(Vec2::new(1200., 620.)),
                        ..Default::default()
                    };
                    clue_box.transform = Transform {
                        translation: Vec3::new(0., -10., 15.),
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
                        /*
                        if (text_iter < 7) {
                            // To keep categories + title unmoved: genuinely optional, but I like it
                            text_iter += 1;
                            continue;
                        }
                        */
                        if (text_iter == 0) {
                            // To keep title at top, optional as well
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


// Categories are reversed: pay close attention to categories
fn get_clue(index: i32) -> &'static str {
    // https://docs.google.com/document/d/19mHqRAi4TFoegb3A0rNJBlHo1A9bFoo9O-p8CRMd24Q/edit?usp=sharing
    let mut clues: [&str; 36] = [
        "<<<Random and Misc>>>",
        "This “theme park” is set to open\non November 1st, 2022 at Aichi Earth Expo\nMemorial Park in Nagakute, Japan. It notably\nis experience based, and thus features no\nstandard rides.\n \n \n ",
        "This style of painting is often used\nin early sketches for Ghibli films\nby Hayao Miyazaki, and makes sense considering\nthe kind of art styles that Ghibli\nfilms go for.\n \n \n ",
        "Miyazaki announced his\nretirement from feature films shortly\nafter the release of this film.\n \n \n ",
        "This movie is the most recent\nStudio Ghibli venture, which has seen releases\nfrom 2020-2021. It was\na multiple company procedure\n- not just Studio Ghibli - and was directed by\nHayao Miyazaki’s son, Goro. It, rather unfortunately,\nuses 3D animation, and has stellar reviews like\n“With a story as uninspired as its animation, [it is a]\nnear-total—misfire for Studio Ghibli.”",
        "Disney began a partnership\nwith Studio Ghibli to aid\nin international releases in this year.\n \n \n ",
        "<<<Scary Villains>>>",
        "This large, elderly woman is primarily a villain\nbecause of her greed. She has a younger twin sister,\nloves her son Boh, and is the main antagonist\nin Spirited Away.\n \n \n ",
        "This woman is seen as the secondary\nantagonist to Madame Suliman, since her\nrole becomes more complicated\nafter the start of her story. That being said,\nshe notably curses the main character,\nSophie, to look like an elderly woman:\nbased on the aging magic she uses\non herself to keep her\nlooking young. She stars in Howls Moving Castle.",
        "The villain in this movie kidnaps the female\nlead, Sheeta, and holds her\ncaptive on an airship. It is\nfound that he is on the search for the kingdom\nof Laputa, which features magic that the\nmain protagonists end up having a connection to.\nThe main protagonist, Pazu, generally\nfoils his schemes.",
        "This scruffy old fellow kidnaps\nthe main character and transforms her into a cat.\nHe notably has a purple gem on top of his\nhead that looks like a false eye, and his\ntwo real eyes are different colors: red\nand blue. He stars in The Cat Returns.\n \n \n ",
        "This character is the father of the\nmain and title character, and is more complex \nand misguided than straight evil. He is a researcher,\nlives in an underwater harbor, and has had \nchildren with a sea goddess:\nexplaining key parts of the film.\nHis appearance is markedly whimsical\nand slender. He stars in Ponyo.",
        "<<<Historical Knowledge>>>",
        "Despite releasing just before Studio\nGhiblio fficially formed, this movie\nis generally cited as their first\nfilm, since it featured most of\nthe same creators (Hayao Miyazaki as\nthe director, for example), story themes, and art\nstyles found throughout the rest of Ghibli's portfolio.\n \n",
        "This movie is the first feature film\nofficially released by Studio Ghibli, in spite of\nmany considering it their second film.\n \n \n ",
        "This was the first movie directed\nby Hayao Miyazaki's son, Goro,\nwhich was famously hated by Hayao,\nand struggled at the box office and\nin reviews more so than\nother Ghibli works.\n \n ",
        "This Studio Ghibli film\nwas released in 2013.\n \n \n \n ",
        "This person helped found Studio\nGhibli as co-director with\nHayao Miyazaki, after working with Hayao and\nToshioSuzuki (a producer) at the\nanimation studio Topcraft.\n ",
        "<<<Creatures and Animals>>>",
        "This iconic black cat stars alongside\nthe main character witch in Kiki’s Delivery Service.\n \n \n ",
        "This creature may be the only one that\nis also amode of transportation, and is\nrather anti-American considering its\napproval of public transit. It is\nalso of note that it is felinen\nin nature, which fits the general theme of its\nfilm’s woodland creatures.\n ",
        "These tiny, black creatures \nare generally seen\ncarrying rocks or gems, and are seen in My\nNeighbor Totoro and Spirited Away.\nThey leave black dust as they walk\naround, and temporarily dissolve when crushed.\n \n \n ",
        "This lazy dog — inspired by a\nFrench hunting dog breed —\nis owned as the “errand dog”\n by the villain Suliman.\nHe is a character for the film\nHowl’s Moving Castle.\n ",
        "This pet “Fox Squirrel” was given as\na gift to the main character, who\ngives a little aggression before becoming friends.\nHe stars in Nausicaä of\nthe Valley of the Wind.\n \n \n \n ",
        "<<<Humans (Mostly)>>>",
        "This helpful young boy ensures the\nsuccess of the young girl\nprotagonist, but is later revealed\nto have a dragon form. He is a\ncharacter from the film Spirited Away.\n \n \n ",
        "This young boy saves a goldfish in his seaside town,\nwhich licks his cut finger to prove its mortality.\nHe is also the love interest, and co main character\nwith the film titled character Ponyo.\n \n \n ",
        "This 10 year old girl stars in My Neighbor Totoro,\nand has a 4 year old sister named Mei.\nShe struggles with her mother who is sick, and is\ntaken on some magical journeys with the movie’s\nwoodland creatures: primarily Totoro.\n \n ",
        "This young girl stars in The Cat Returns\nas the one taken by the feline prince for\nmarriage in the life of a cat.\n \n \n ",
        "This 16 year old girl is known as\n“the girl who raises red flags”, based\non the tragic death of her father in the\nKorean war. She lives in Coquelicot Manor, and is\nthe star of From Up on Poppy Hill.\n \n ",
        "<<<General Movies>>>",
        "This Ghibli film\nfamously features\na humanoid pig\nas its main character.\n \n \n ",
        "This Ghibli film stars a young girl\nwho stumbles upon a furry, woodland creature,\nwho helps her in rather magical ways.\n \n \n ",
        "This Ghibli film has a main location\nof a whimsical bathhouse, despite starting\non a simple car ride and stumbling\nupon a unique market-town.\n \n \n ",
        "This Ghibli film lands on the more serious\nside of Ghibli films, primarily because\nof it taking place in World War II, and features\nfirebombing, along with children struggling\nto survive on their own. \n ",
        "This Ghibli film has one of the most obvious\nenvironmentalist messages, features many\nanimal gods such as a boar, a deer,\nand wolves, and stars a warrior princess.\n \n \n ",
    ];
    println!("{} => {}", index, clues[index as usize]);
    return clues[index as usize];
}
