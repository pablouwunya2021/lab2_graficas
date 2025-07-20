use minifb::{Key, Window, WindowOptions};
use std::{thread, time};
use gif::{Encoder, Frame, Repeat, SetParameter};
use std::fs::File;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;
const SCALE: usize = 5;
const WINDOW_WIDTH: usize = WIDTH * SCALE;
const WINDOW_HEIGHT: usize = HEIGHT * SCALE;

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Alive,
    Dead,
}

fn get_color(cell: Cell) -> u32 {
    match cell {
        Cell::Alive => 0xFFFFFF, // Blanco
        Cell::Dead => 0x000000,  // Negro
    }
}

fn count_neighbors(grid: &Vec<Vec<Cell>>, x: usize, y: usize) -> usize {
    let mut count = 0;
    for dy in [-1, 0, 1].iter() {
        for dx in [-1, 0, 1].iter() {
            if *dx == 0 && *dy == 0 {
                continue;
            }

            let nx = (x as isize + dx).rem_euclid(WIDTH as isize) as usize;
            let ny = (y as isize + dy).rem_euclid(HEIGHT as isize) as usize;

            if grid[ny][nx] == Cell::Alive {
                count += 1;
            }
        }
    }
    count
}

fn next_generation(grid: &Vec<Vec<Cell>>) -> Vec<Vec<Cell>> {
    let mut new_grid = grid.clone();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let neighbors = count_neighbors(grid, x, y);
            let cell = grid[y][x];

            new_grid[y][x] = match (cell, neighbors) {
                (Cell::Alive, 2 | 3) => Cell::Alive,
                (Cell::Alive, _) => Cell::Dead,
                (Cell::Dead, 3) => Cell::Alive,
                (_, _) => Cell::Dead,
            };
        }
    }

    new_grid
}

fn point(buffer: &mut Vec<u32>, x: usize, y: usize, color: u32) {
    for dy in 0..SCALE {
        for dx in 0..SCALE {
            let px = x * SCALE + dx;
            let py = y * SCALE + dy;
            if px < WINDOW_WIDTH && py < WINDOW_HEIGHT {
                buffer[py * WINDOW_WIDTH + px] = color;
            }
        }
    }
}

fn render(buffer: &mut Vec<u32>, grid: &Vec<Vec<Cell>>) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            point(buffer, x, y, get_color(grid[y][x]));
        }
    }
}

fn initial_pattern() -> Vec<Vec<Cell>> {
    let mut grid = vec![vec![Cell::Dead; WIDTH]; HEIGHT];

    // Glider
    let glider = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    for (x, y) in glider {
        grid[y + 10][x + 10] = Cell::Alive;
    }

    // Blinker
    let blinker = [(0, 0), (1, 0), (2, 0)];
    for (x, y) in blinker {
        grid[y + 30][x + 30] = Cell::Alive;
    }

    // Toad
    let toad = [(1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1)];
    for (x, y) in toad {
        grid[y + 60][x + 60] = Cell::Alive;
    }

    // Beacon
    let beacon = [(0, 0), (1, 0), (0, 1), (3, 2), (2, 3), (3, 3)];
    for (x, y) in beacon {
        grid[y + 15][x + 50] = Cell::Alive;
    }

    // Lightweight spaceship (LWSS)
    let lwss = [(1, 0), (4, 0), (0, 1), (0, 2), (4, 2), (0, 3), (1, 3), (2, 3), (3, 3)];
    for (x, y) in lwss {
        grid[y + 40][x + 5] = Cell::Alive;
    }

    grid
}

fn export_gif(frames: &Vec<Vec<u32>>) {
    let mut image = File::create("output.gif").unwrap();
    let mut encoder = Encoder::new(&mut image, WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, &[]).unwrap();
    encoder.set(Repeat::Infinite).unwrap();

    for frame_buf in frames {
        let mut pixel_data = Vec::with_capacity(WINDOW_WIDTH * WINDOW_HEIGHT * 3);
        for pixel in frame_buf {
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;

            pixel_data.push(r);
            pixel_data.push(g);
            pixel_data.push(b);
        }

        let mut frame = Frame::from_rgb(WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, &pixel_data);
        frame.delay = 5; // 5 * 10ms = 50ms por frame
        encoder.write_frame(&frame).unwrap();
    }
}

fn main() {
    let mut window = Window::new(
        "Conway's Game of Life - ESC to quit",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    let mut buffer: Vec<u32> = vec![0; WINDOW_WIDTH * WINDOW_HEIGHT];
    let mut grid = initial_pattern();

    let mut gif_frames: Vec<Vec<u32>> = Vec::new();
    let max_frames = 50;
    let mut frame_count = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) && frame_count < max_frames {
        render(&mut buffer, &grid);
        window.update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();

        gif_frames.push(buffer.clone()); // Guardar frame para GIF
        grid = next_generation(&grid);
        frame_count += 1;

        thread::sleep(time::Duration::from_millis(100));
    }

    export_gif(&gif_frames);
}
