use minifb::{Key, Window, WindowOptions};
use std::{thread, time};

const INITIAL_WIDTH: usize = 100;
const INITIAL_HEIGHT: usize = 100;
const INITIAL_SCALE: usize = 5;

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Dead,
    Alive(u8), // Edad
}

struct Config {
    width: usize,
    height: usize,
    scale: usize,
    fps: u64,
    color_mode: bool,
}

impl Config {
    fn window_size(&self) -> (usize, usize) {
        (self.width * self.scale, self.height * self.scale)
    }
}

fn get_color(cell: Cell, color_mode: bool) -> u32 {
    match cell {
        Cell::Dead => 0x000000,
        Cell::Alive(age) => {
            if color_mode {
                let age = age as u32;
                let r = 50 + 20 * age;
                let g = 255u32.saturating_sub(15 * age);
                let b = 50 + 5 * age;
                (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
            } else {
                0xFFFFFF
            }
        }
    }
}


fn point(buffer: &mut Vec<u32>, config: &Config, x: usize, y: usize, color: u32) {
    for dy in 0..config.scale {
        for dx in 0..config.scale {
            let px = x * config.scale + dx;
            let py = y * config.scale + dy;
            if px < config.width * config.scale && py < config.height * config.scale {
                buffer[py * config.width * config.scale + px] = color;
            }
        }
    }
}

fn render(buffer: &mut Vec<u32>, grid: &Vec<Vec<Cell>>, config: &Config) {
    for y in 0..config.height {
        for x in 0..config.width {
            point(buffer, config, x, y, get_color(grid[y][x], config.color_mode));
        }
    }
}

fn count_neighbors(grid: &Vec<Vec<Cell>>, x: usize, y: usize, config: &Config) -> usize {
    let mut count = 0;
    for dy in [-1, 0, 1] {
        for dx in [-1, 0, 1] {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = ((x as isize + dx).rem_euclid(config.width as isize)) as usize;
            let ny = ((y as isize + dy).rem_euclid(config.height as isize)) as usize;

            if let Cell::Alive(_) = grid[ny][nx] {
                count += 1;
            }
        }
    }
    count
}

fn next_generation(grid: &Vec<Vec<Cell>>, config: &Config) -> Vec<Vec<Cell>> {
    let mut new_grid = grid.clone();
    for y in 0..config.height {
        for x in 0..config.width {
            let neighbors = count_neighbors(grid, x, y, config);
            let cell = grid[y][x];

            new_grid[y][x] = match (cell, neighbors) {
                (Cell::Alive(age), 2 | 3) => Cell::Alive(age.saturating_add(1)),
                (Cell::Alive(_), _) => Cell::Dead,
                (Cell::Dead, 3) => Cell::Alive(1),
                _ => Cell::Dead,
            };
        }
    }
    new_grid
}

fn ultra_complex_initial_pattern(config: &Config) -> Vec<Vec<Cell>> {
    let mut grid = vec![vec![Cell::Dead; config.width]; config.height];

    let place = |cells: &[(usize, usize)], ox: usize, oy: usize, grid: &mut Vec<Vec<Cell>>| {
        for &(x, y) in cells {
            let nx = ox + x;
            let ny = oy + y;
            if nx < config.width && ny < config.height {
                grid[ny][nx] = Cell::Alive(1);
            }
        }
    };

    // 1) Dos Gosper Glider Guns separadas
    let gosper = [
        (1, 5), (1, 6), (2, 5), (2, 6),
        (11, 5), (11, 6), (11, 7),
        (12, 4), (12, 8),
        (13, 3), (13, 9),
        (14, 3), (14, 9),
        (15, 6),
        (16, 4), (16, 8),
        (17, 5), (17, 6), (17, 7),
        (18, 6),
        (21, 3), (21, 4), (21, 5),
        (22, 3), (22, 4), (22, 5),
        (23, 2), (23, 6),
        (25, 1), (25, 2), (25, 6), (25, 7),
        (35, 3), (35, 4), (36, 3), (36, 4),
    ];
    place(&gosper, 2, 2, &mut grid);
    place(&gosper, 60, 60, &mut grid);

    // 2) Gliders (5)
    let glider = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    place(&glider, 10, 85, &mut grid);
    place(&glider, 15, 80, &mut grid);
    place(&glider, 20, 75, &mut grid);
    place(&glider, 80, 10, &mut grid);
    place(&glider, 85, 15, &mut grid);

    // 3) LWSS (Lightweight Spaceships) (3)
    let lwss = [(1,0),(4,0),(0,1),(0,2),(4,2),(0,3),(1,3),(2,3),(3,3)];
    place(&lwss, 40, 30, &mut grid);
    place(&lwss, 50, 40, &mut grid);
    place(&lwss, 55, 45, &mut grid);

    // 4) MWSS (Middleweight Spaceship)
    let mwss = [
        (1,0),(4,0),(5,1),(0,2),(5,2),(0,3),(1,3),(2,3),(3,3),(4,3),
    ];
    place(&mwss, 70, 50, &mut grid);

    // 5) HWSS (Heavyweight Spaceship)
    let hwss = [
        (2,0),(5,0),(6,1),(0,2),(6,2),(0,3),(1,3),(2,3),(3,3),(4,3),(5,3),
    ];
    place(&hwss, 75, 55, &mut grid);

    // 6) Oscillators

    // Pulsar
    let pulsar_offsets = [
        (2,0),(3,0),(4,0),(8,0),(9,0),(10,0),
        (0,2),(5,2),(7,2),(12,2),
        (0,3),(5,3),(7,3),(12,3),
        (0,4),(5,4),(7,4),(12,4),
        (2,5),(3,5),(4,5),(8,5),(9,5),(10,5),
        (2,7),(3,7),(4,7),(8,7),(9,7),(10,7),
        (0,8),(5,8),(7,8),(12,8),
        (0,9),(5,9),(7,9),(12,9),
        (0,10),(5,10),(7,10),(12,10),
        (2,12),(3,12),(4,12),(8,12),(9,12),(10,12),
    ];
    place(&pulsar_offsets, 10, 10, &mut grid);

    // Pentadecathlon (oscillator periodo 15)
    let pentadecathlon = [
        (2,0),(3,0),(4,0),(5,0),(6,0),(7,0),(8,0),(9,0),(10,0),(11,0),
        (3,1),(10,1),
    ];
    place(&pentadecathlon, 20, 20, &mut grid);

    // Traffic light oscillator
    let traffic_light = [
        (1,0),(2,0),
        (0,1),(3,1),
        (1,2),(2,2),
    ];
    place(&traffic_light, 25, 25, &mut grid);

    // 7) Still lifes (estáticos)

    // Block
    let block = [(0, 0), (1, 0), (0, 1), (1, 1)];
    place(&block, 5, 5, &mut grid);

    // Boat
    let boat = [(0, 0), (1, 0), (0, 1), (2, 1), (1, 2)];
    place(&boat, 95, 10, &mut grid);

    // Beehive
    let beehive = [(1,0), (2,0), (0,1), (3,1), (1,2), (2,2)];
    place(&beehive, 90, 90, &mut grid);

    // 8) Diehard pattern (muy raro)
    let diehard = [(6,0),(0,1),(1,1),(1,2),(5,2),(6,2),(7,2)];
    place(&diehard, 80, 80, &mut grid);

    // 9) Acorn (otro raro y famoso)
    let acorn = [(1,0),(3,1),(0,2),(1,2),(4,2),(5,2),(6,2)];
    place(&acorn, 70, 80, &mut grid);

    // 10) Giant Diehard (más grande y lento)
    let giant_diehard = [
        (6,1),(0,2),(1,2),(1,3),(5,3),(6,3),(7,3),
        (0,10),(1,11),(2,11),(3,11),(4,11),(5,11),(6,11),
    ];
    place(&giant_diehard, 20, 80, &mut grid);

    grid
}


fn main() {
    let mut config = Config {
        width: INITIAL_WIDTH,
        height: INITIAL_HEIGHT,
        scale: INITIAL_SCALE,
        fps: 10,
        color_mode: true,
    };

    let (win_w, win_h) = config.window_size();
    let mut window = Window::new(
        "Conway's Game of Life - Patrón Complejo",
        win_w,
        win_h,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| panic!("{}", e));

    let mut buffer = vec![0; win_w * win_h];
    let mut grid = ultra_complex_initial_pattern(&config);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        render(&mut buffer, &grid, &config);
        window
            .update_with_buffer(&buffer, config.width * config.scale, config.height * config.scale)
            .unwrap();

        buffer = vec![0; config.width * config.scale * config.height * config.scale];
        grid = next_generation(&grid, &config);
        thread::sleep(time::Duration::from_millis(1000 / config.fps));
    }
}
