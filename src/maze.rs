use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).expect("Could not open maze file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| {
            line.expect("Could not read line from maze file")
                .chars()
                .collect()
        })
        .collect()
}

pub fn validate_maze(maze: &Maze) -> bool {
    if maze.is_empty() || maze.iter().any(|row| row.is_empty()) {
        return false;
    }

    let width = maze[0].len();

    if !maze.iter().all(|row| row.len() == width) {
        return false;
    }

    let player_count = count_char(maze, 'p');
    let goal_count = count_char(maze, 'g');

    player_count == 1 && goal_count == 1 && has_wall_borders(maze)
}

pub fn find_char(maze: &Maze, target: char) -> Option<(usize, usize)> {
    for (y, row) in maze.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if *tile == target {
                return Some((x, y));
            }
        }
    }

    None
}

pub fn is_walkable(maze: &Maze, x: f32, y: f32, block_size: usize) -> bool {
    if block_size == 0 || !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return false;
    }

    let column = (x / block_size as f32).floor() as usize;
    let row = (y / block_size as f32).floor() as usize;

    match maze.get(row).and_then(|maze_row| maze_row.get(column)) {
        Some(' ' | 'p' | 'g') => true,
        _ => false,
    }
}

fn count_char(maze: &Maze, target: char) -> usize {
    maze.iter()
        .flatten()
        .filter(|tile| **tile == target)
        .count()
}

fn has_wall_borders(maze: &Maze) -> bool {
    let height = maze.len();
    let width = maze[0].len();

    for x in 0..width {
        if maze[0][x] != '#' || maze[height - 1][x] != '#' {
            return false;
        }
    }

    for row in maze {
        if row[0] != '#' || row[width - 1] != '#' {
            return false;
        }
    }

    true
}
