extern crate rand;
extern crate sdl2;
extern crate serde;

use rand::Rng;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

pub static MAP_WIDTH: i32 = 15;
pub static MAP_HEIGHT: i32 = 15;
pub static TILE_SIZE: i32 = 50;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub textures: Textures,
    pub sounds: Sounds,
}

#[derive(Serialize, Deserialize)]
pub struct Sounds {
    pub found_treasure: String,
    pub next_stage: String,
    pub next_game: String,
}

#[derive(Serialize, Deserialize)]
pub struct Textures {
    pub ground: String,
    pub wall: String,
    pub player: Player,
    pub shadow: Shadow,
    pub title: String,
    pub regen_title: String,
    pub font: String,
    pub treasure: String,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub right: String,
    pub left: String,
}

#[derive(Serialize, Deserialize)]
pub struct Shadow {
    pub light: String,
    pub dark: String,
}

pub struct Sprite {
    pub x: i32,
    pub y: i32,
    // distance to move
    pub dx: i32,
    pub dy: i32,
}

impl Sprite {
    pub fn d_move(&mut self) {
        let d = TILE_SIZE / 5;
        if self.dx > 0 {
            self.x += d;
            self.dx -= d;
        } else if self.dx < 0 {
            self.x -= d;
            self.dx += d;
        }
        if self.dy > 0 {
            self.y += d;
            self.dy -= d;
        } else if self.dy < 0 {
            self.y -= d;
            self.dy += d;
        }
    }
}

impl Settings {
    pub fn new(path: &Path) -> Settings {
        let mut file = match File::open(path) {
            Err(_) => panic!("Failed to open file: {}", path.display()),
            Ok(file) => file,
        };
        let mut json_str = String::new();
        match file.read_to_string(&mut json_str) {
            Err(_) => panic!("Failed to read file: {}", path.display()),
            Ok(_) => 0,
        };
        let val: Settings = serde_json::from_str(&json_str).unwrap();
        val
    }
}

pub fn play_wav(path: &String) {
    Command::new("aplay")
        .arg(path)
        .spawn()
        .expect("failed to play wav file");
}

pub fn edit_map(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ground: &sdl2::render::Texture,
    wall: &sdl2::render::Texture,
    treasure: &sdl2::render::Texture,
    map: &Vec<Vec<i32>>,
) {
    let mut texture: &sdl2::render::Texture;
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            texture = match map[y as usize][x as usize] {
                1 => wall,
                2 => treasure,
                0 | _ => ground,
            };
            draw(canvas, texture, x * TILE_SIZE, y * TILE_SIZE);
        }
    }
}
pub fn edit_shadow(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    shadow_d: &sdl2::render::Texture,
    shadow_l: &sdl2::render::Texture,
    s_player: &Sprite,
) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_HEIGHT {
            if (x - s_player.x / TILE_SIZE).abs() > 2 || (y - s_player.y / TILE_SIZE).abs() > 2 {
                draw(canvas, shadow_d, x * TILE_SIZE, y * TILE_SIZE);
            } else if (x - s_player.x / TILE_SIZE).abs() > 1
                || (y - s_player.y / TILE_SIZE).abs() > 1
            {
                draw(canvas, shadow_l, x * TILE_SIZE, y * TILE_SIZE);
            }
        }
    }
}
pub fn draw(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &sdl2::render::Texture,
    x: i32,
    y: i32,
) {
    let dest = Rect::new(
        x as i32,
        y as i32,
        texture.query().width,
        texture.query().height,
    );
    canvas.copy(&texture, None, dest).unwrap();
}

pub fn draw_text(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    font: &sdl2::ttf::Font,
    text: String,
    x: i32,
    y: i32,
) {
    let tex_creator = canvas.texture_creator();
    let surface = font
        .render(&text)
        .blended(Color::RGB(255, 255, 255))
        .unwrap();
    let tex = tex_creator.create_texture_from_surface(&surface).unwrap();
    draw(canvas, &tex, x, y);
}

pub fn movable(x: i32, y: i32, map: &Vec<Vec<i32>>) -> bool {
    if map[as_tile(y)][as_tile(x)] != 1 {
        true
    } else {
        false
    }
}

pub fn as_tile(n: i32) -> usize {
    (n / TILE_SIZE).try_into().unwrap()
}

pub fn gen_dungeon() -> Vec<Vec<i32>> {
    let mut map = vec![vec![0; MAP_WIDTH as usize]; MAP_HEIGHT as usize];
    // generate outer wall
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if (y == 0 || y == MAP_HEIGHT - 1) || (x == 0 || x == MAP_WIDTH - 1) {
                map[y as usize][x as usize] = 1;
            }
        }
    }
    // generate inner wall
    let mut x = 2;
    let mut y = 2;
    while y < MAP_HEIGHT - 2 {
        while x < MAP_WIDTH - 2 {
            map[y as usize][x as usize] = 1;
            loop {
                match rnd_n(4) {
                    0 => {
                        if x == 2 {
                            map[(y - 1) as usize][x as usize] = 1;
                            break;
                        }
                    }
                    1 => {
                        if map[(y + 1) as usize][x as usize] == 0 {
                            map[(y + 1) as usize][x as usize] = 1;
                            break;
                        }
                    }
                    2 => {
                        if map[y as usize][(x - 1) as usize] == 0 {
                            map[y as usize][(x - 1) as usize] = 1;
                            break;
                        }
                    }
                    3 => {
                        if map[y as usize][(x + 1) as usize] == 0 {
                            map[y as usize][(x + 1) as usize] = 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            x += 2;
        }
        y += 2;
        x = 2;
    }
    // generate treasures
    for _i in 0..3 {
        let mut x = rnd_n(MAP_WIDTH as u32);
        let mut y = rnd_n(MAP_HEIGHT as u32);
        while map[y as usize][x as usize] != 0 {
            x = rnd_n(MAP_WIDTH as u32);
            y = rnd_n(MAP_HEIGHT as u32);
        }
        map[y as usize][x as usize] = 2;
    }
    map
}

pub fn rnd_n(n: u32) -> u32 {
    let mut rng = rand::thread_rng();
    let r: u32 = rng.gen();
    r % n
}
