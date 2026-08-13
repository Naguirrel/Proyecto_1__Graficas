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
    matches!(
        cell_at_world_position(maze, x, y, block_size),
        Some(' ' | 'p' | 'g')
    )
}

fn is_wall(cell: char) -> bool {
    matches!(cell, '#' | '+' | '%' | '@' | '&' | '!')
}

pub fn cell_at_world_position(maze: &Maze, x: f32, y: f32, block_size: usize) -> Option<char> {
    if block_size == 0 || !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }

    let column = (x / block_size as f32).floor() as usize;
    let row = (y / block_size as f32).floor() as usize;

    maze.get(row)
        .and_then(|maze_row| maze_row.get(column))
        .copied()
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

    let top = &maze[0];
    let bottom = &maze[height - 1];

    if top
        .iter()
        .zip(bottom.iter())
        .any(|(top, bottom)| !is_wall(*top) || !is_wall(*bottom))
    {
        return false;
    }

    for row in maze {
        if !is_wall(row[0]) || !is_wall(row[width - 1]) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    fn maze_from_text(text: &str) -> Maze {
        text.lines().map(|line| line.chars().collect()).collect()
    }

    fn path_exists(maze: &Maze, start: (usize, usize), goal: (usize, usize)) -> bool {
        let mut visited = vec![vec![false; maze[0].len()]; maze.len()];
        let mut queue = VecDeque::from([start]);
        visited[start.1][start.0] = true;

        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == goal {
                return true;
            }

            for (dx, dy) in [(1_isize, 0_isize), (-1, 0), (0, 1), (0, -1)] {
                let next_x = x as isize + dx;
                let next_y = y as isize + dy;

                if next_x < 0 || next_y < 0 {
                    continue;
                }

                let next_x = next_x as usize;
                let next_y = next_y as usize;

                if next_y >= maze.len() || next_x >= maze[next_y].len() || visited[next_y][next_x] {
                    continue;
                }

                if matches!(maze[next_y][next_x], ' ' | 'p' | 'g') {
                    visited[next_y][next_x] = true;
                    queue.push_back((next_x, next_y));
                }
            }
        }

        false
    }

    fn route_exists_through(maze: &Maze, waypoint: (usize, usize)) -> bool {
        let player = find_char(maze, 'p').expect("test maze must contain p");
        let goal = find_char(maze, 'g').expect("test maze must contain g");

        path_exists(maze, player, waypoint) && path_exists(maze, waypoint, goal)
    }

    fn goal_has_yellow_wall(maze: &Maze) -> bool {
        let (goal_x, goal_y) = find_char(maze, 'g').expect("test maze must contain g");

        [(1_isize, 0_isize), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .any(|(dx, dy)| {
                let x = goal_x as isize + dx;
                let y = goal_y as isize + dy;

                x >= 0
                    && y >= 0
                    && maze
                        .get(y as usize)
                        .and_then(|row| row.get(x as usize))
                        .is_some_and(|cell| *cell == '!')
            })
    }

    fn wall_characters(maze: &Maze) -> Vec<char> {
        let mut characters = maze
            .iter()
            .flatten()
            .filter(|cell| is_wall(**cell) && **cell != '!')
            .copied()
            .collect::<Vec<_>>();

        characters.sort_unstable();
        characters.dedup();
        characters
    }

    fn repeated_adjacent_wall_color(maze: &Maze) -> Option<char> {
        for (y, row) in maze.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if !is_wall(*cell) {
                    continue;
                }

                if x + 1 < row.len() && row[x + 1] == *cell {
                    return Some(*cell);
                }

                if y + 1 < maze.len() && maze[y + 1][x] == *cell {
                    return Some(*cell);
                }
            }
        }

        None
    }

    fn yellow_walls_are_only_next_to_goal(maze: &Maze) -> bool {
        let (goal_x, goal_y) = find_char(maze, 'g').expect("test maze must contain g");

        maze.iter().enumerate().all(|(y, row)| {
            row.iter().enumerate().all(|(x, cell)| {
                *cell != '!'
                    || ((x as isize - goal_x as isize).abs() + (y as isize - goal_y as isize).abs())
                        == 1
            })
        })
    }

    #[test]
    fn project_maze_is_valid_and_connects_player_to_goal() {
        let maze = maze_from_text(include_str!("../maze.txt"));

        assert!(validate_maze(&maze));
        assert_eq!(maze.len(), 13);
        assert!(maze.iter().all(|row| row.len() == 19));

        let player = find_char(&maze, 'p').expect("test maze must contain p");
        let goal = find_char(&maze, 'g').expect("test maze must contain g");

        assert_eq!(player, (1, 1));
        assert_eq!(goal, (16, 11));
        assert!(path_exists(&maze, player, goal));
    }

    #[test]
    fn project_maze_has_three_routes_to_goal() {
        let maze = maze_from_text(include_str!("../maze.txt"));

        assert!(route_exists_through(&maze, (5, 1)));
        assert!(route_exists_through(&maze, (1, 7)));
        assert!(route_exists_through(&maze, (7, 11)));
    }

    #[test]
    fn project_maze_has_yellow_wall_next_to_goal() {
        let maze = maze_from_text(include_str!("../maze.txt"));

        assert!(goal_has_yellow_wall(&maze));
        assert!(yellow_walls_are_only_next_to_goal(&maze));
    }

    #[test]
    fn project_maze_uses_five_wall_colors_without_repeated_neighbors() {
        let maze = maze_from_text(include_str!("../maze.txt"));

        assert_eq!(wall_characters(&maze), vec!['#', '%', '&', '+', '@']);
        assert_eq!(repeated_adjacent_wall_color(&maze), None);
    }

    #[test]
    fn is_walkable_accepts_only_known_walkable_cells() {
        let maze = maze_from_text(
            "#####\n\
             #p g#\n\
             # X #\n\
             #####",
        );

        assert!(is_walkable(&maze, 15.0, 15.0, 10));
        assert!(is_walkable(&maze, 20.0, 15.0, 10));
        assert!(is_walkable(&maze, 30.0, 15.0, 10));
        assert!(!is_walkable(&maze, 20.0, 25.0, 10));
        assert!(!is_walkable(&maze, -1.0, 15.0, 10));
        assert!(!is_walkable(&maze, 100.0, 15.0, 10));
        assert!(!is_walkable(&maze, f32::NAN, 15.0, 10));
        assert!(!is_walkable(&maze, 15.0, 15.0, 0));
    }

    #[test]
    fn validation_rejects_non_rectangular_and_missing_markers() {
        let non_rectangular = maze_from_text("###\n#p#\n####");
        assert!(!validate_maze(&non_rectangular));

        let missing_goal = maze_from_text("###\n#p#\n###");
        assert!(!validate_maze(&missing_goal));

        let missing_player = maze_from_text("###\n#g#\n###");
        assert!(!validate_maze(&missing_player));
    }
}
