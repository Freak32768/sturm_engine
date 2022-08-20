extern crate sdl2;
extern crate serde;

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::path::Path;
use std::{thread, time};
use sturm_engine::sturm;

static WINDOW_WIDTH: i32 = sturm::MAP_WIDTH * sturm::TILE_SIZE;
static WINDOW_HEIGHT: i32 = sturm::MAP_HEIGHT * sturm::TILE_SIZE;
static FPS: u32 = 60;

pub fn main() {
    // initializing SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG);
    let ttf_context = sdl2::ttf::init().unwrap();
    let window = video_subsystem
        .window("sturm_engine", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    // initializing sturm_engine
    let s = sturm::Settings::new(Path::new("sample/settings.json"));
    let tex_creator = canvas.texture_creator();
    // textures
    let t_ground = tex_creator
        .load_texture(Path::new(&s.textures.ground))
        .unwrap();
    let t_wall = tex_creator
        .load_texture(Path::new(&s.textures.wall))
        .unwrap();
    let t_player_r = tex_creator
        .load_texture(Path::new(&s.textures.player.right))
        .unwrap();
    let t_player_l = tex_creator
        .load_texture(Path::new(&s.textures.player.left))
        .unwrap();
    let t_shadow_d = tex_creator
        .load_texture(Path::new(&s.textures.shadow.dark))
        .unwrap();
    let t_shadow_l = tex_creator
        .load_texture(Path::new(&s.textures.shadow.light))
        .unwrap();
    let t_title = tex_creator
        .load_texture(Path::new(&s.textures.title))
        .unwrap();
    let t_regen = tex_creator
        .load_texture(Path::new(&s.textures.regen_title))
        .unwrap();
    let t_treasure = tex_creator
        .load_texture(Path::new(&s.textures.treasure))
        .unwrap();
    // font
    let mut font = ttf_context
        .load_font(Path::new(&s.textures.font), 36)
        .unwrap();
    font.set_style(sdl2::ttf::FontStyle::NORMAL);
    'main_loop: loop {
        // 0: ground, 1: wall
        let mut map = sturm::gen_dungeon();
        // player pos
        let mut s_player = sturm::Sprite {
            x: sturm::TILE_SIZE,
            y: sturm::TILE_SIZE,
            dx: 0,
            dy: 0,
        };
        // other variables
        let mut t_player: &sdl2::render::Texture = &t_player_r;
        let mut c_regen = 0;
        let mut time = 30 * FPS;
        let mut score = 0;
        // title rendering
        sturm::draw(&mut canvas, &t_title, 0, 0);
        sturm::draw_text(
            &mut canvas,
            &font,
            "[Esc]: restart, [arrow keys]: move".to_string(),
            120,
            400,
        );
        canvas.present();
        thread::sleep(time::Duration::from_secs(5));
        'game_loop: loop {
            // check
            if time <= 0 {
                break 'game_loop;
            } else {
                time -= 1;
            }
            // player action
            s_player.d_move();
            for event in event_pump.poll_iter() {
                if s_player.dx == 0 && s_player.dy == 0 {
                    match event {
                        Event::Quit { .. } => break 'main_loop,
                        Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'game_loop,
                        Event::KeyDown {
                            keycode: Some(Keycode::Up),
                            ..
                        } => {
                            if sturm::movable(s_player.x, s_player.y - sturm::TILE_SIZE, &map) {
                                s_player.dy -= sturm::TILE_SIZE;
                            }
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Down),
                            ..
                        } => {
                            if sturm::movable(s_player.x, s_player.y + sturm::TILE_SIZE, &map) {
                                s_player.dy += sturm::TILE_SIZE;
                            }
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Right),
                            ..
                        } => {
                            if sturm::movable(s_player.x + sturm::TILE_SIZE, s_player.y, &map) {
                                s_player.dx += sturm::TILE_SIZE;
                                t_player = &t_player_r;
                            }
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Left),
                            ..
                        } => {
                            if sturm::movable(s_player.x - sturm::TILE_SIZE, s_player.y, &map) {
                                s_player.dx -= sturm::TILE_SIZE;
                                t_player = &t_player_l;
                            }
                        }
                        _ => {}
                    }
                }
            }
            // regenerating map if necessary
            if s_player.x / sturm::TILE_SIZE == 13 && s_player.y / sturm::TILE_SIZE == 13 {
                s_player.x = sturm::TILE_SIZE;
                s_player.y = sturm::TILE_SIZE;
                map = sturm::gen_dungeon();
                c_regen = 2 * 60;
                score += 10;
                time += 300;
                // play se
                sturm::play_wav(&s.sounds.next_stage);
            }
            // add score if player got a treasure
            if map[sturm::as_tile(s_player.y)][sturm::as_tile(s_player.x)] == 2 {
                score += sturm::rnd_n(4) + 1;
                map[sturm::as_tile(s_player.y)][sturm::as_tile(s_player.x)] = 0;
                // play se
                sturm::play_wav(&s.sounds.found_treasure);
            }
            // redrawing
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();
            sturm::edit_map(&mut canvas, &t_ground, &t_wall, &t_treasure, &map);
            sturm::edit_shadow(&mut canvas, &t_shadow_d, &t_shadow_l, &s_player);
            sturm::draw(&mut canvas, t_player, s_player.x, s_player.y);
            sturm::draw_text(
                &mut canvas,
                &font,
                format!("time: {}   score: {}", time / 60, score),
                400,
                0,
            );
            if c_regen > 0 {
                sturm::draw(&mut canvas, &t_regen, 0, 0);
                c_regen -= 1;
            }
            canvas.present();
            thread::sleep(time::Duration::new(0, 1_000_000_000 / FPS));
        }
        // end game and move to next loop
        sturm::draw(&mut canvas, &t_title, 0, 0);
        sturm::draw_text(
            &mut canvas,
            &font,
            format!("score: {}   [Esc]: restart", score),
            200,
            400,
        );
        canvas.present();
        'wait: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main_loop,
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'wait,
                    _ => {}
                }
            }
        }
        // play se
        sturm::play_wav(&s.sounds.next_game);
    }
}
