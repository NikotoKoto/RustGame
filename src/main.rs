use ggez::{conf, event, graphics, Context, GameResult};
use noise::{NoiseFn, Perlin};
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

#[derive(Clone, Debug, PartialEq)]
enum Cell {
    Empty,
    Obstacle,
    Energy,
    Crystal,
    Base,
    ReservedEnergy,
    ReservedCrystal,
}

#[derive(Clone, Debug)]
struct Robot {
    x: usize,
    y: usize,
    role: Role,
    resource_coords: Option<(usize, usize)>,
    carrying: Option<Cell>,
    speed: usize,
    move_counter: usize,
}

impl Robot {
    fn default_speed() -> usize {
        1
    }

    fn increased_speed() -> usize {
        4
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Role {
    Explorer,
    Extractor,
}

#[derive(Eq, PartialEq)]
struct Node {
    cost: usize,
    position: (usize, usize),
    priority: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct GameState {
    map: Vec<Vec<Cell>>,
    map_width: usize,
    map_height: usize,
    base_position: (usize, usize),
    robots: Vec<Robot>,
    crystal_score: u32,
    energy_score: u32,
    game_over: bool,
    discovered: Vec<Vec<bool>>,
}

impl GameState {
    fn new(_ctx: &mut Context, seed: u64) -> GameResult<GameState> {
        let mut rng = StdRng::seed_from_u64(seed);
        let noise = Perlin::new();
        let map_width = 40;
        let map_height = 30;
        let noise_threshold = 0.5;
        let mut map = vec![vec![Cell::Empty; map_width]; map_height];
        let discovered = vec![vec![false; map_width]; map_height];

        // Generation d'obstacle aléatoire
        for y in 0..map_height {
            for x in 0..map_width {
                let nx = x as f64 / map_width as f64;
                let ny = y as f64 / map_height as f64;
                let val = noise.get([nx * 10.0, ny * 10.0, rng.gen::<f64>()]);
                if val > noise_threshold {
                    map[y][x] = Cell::Obstacle;
                }
            }
        }

        // creation d'obstacles pour la bordure de map
        for x in 0..map_width {
            map[0][x] = Cell::Obstacle;
            map[map_height - 1][x] = Cell::Obstacle;
        }
        for y in 0..map_height {
            map[y][0] = Cell::Obstacle;
            map[y][map_width - 1] = Cell::Obstacle;
        }

        // Place la base à une position random
        let base_position = place_randomly(&mut map, &mut rng, Cell::Base, 1).unwrap();

        // enleve les obstacles autour de la base
        for dy in -3..=3 {
            for dx in -3..=3 {
                let (x, y) = (
                    (base_position.0 as isize + dx).clamp(0, map_width as isize - 1) as usize,
                    (base_position.1 as isize + dy).clamp(0, map_height as isize - 1) as usize,
                );
                map[y][x] = Cell::Empty;
            }
        }

        // Place aleatoirement les ressources
        place_randomly(&mut map, &mut rng, Cell::Energy, 5);
        place_randomly(&mut map, &mut rng, Cell::Crystal, 10);

        // S'assurer que la base à une bonne position
        map[base_position.1][base_position.0] = Cell::Base;

        // Initialisation des 3 robots
        let robots = vec![
            Robot {
                x: base_position.0,
                y: base_position.1,
                role: Role::Explorer,
                resource_coords: None,
                carrying: None,
                speed: Robot::default_speed(),
                move_counter: 0,
            },
            Robot {
                x: base_position.0,
                y: base_position.1,
                role: Role::Explorer,
                resource_coords: None,
                carrying: None,
                speed: Robot::default_speed(),
                move_counter: 0,
            },
            Robot {
                x: base_position.0,
                y: base_position.1,
                role: Role::Explorer,
                resource_coords: None,
                carrying: None,
                speed: Robot::default_speed(),
                move_counter: 0,
            },
        ];

        Ok(GameState {
            map,
            map_width,
            map_height,
            base_position,
            robots,
            crystal_score: 0,
            energy_score: 0,
            game_over: false,
            discovered,
        })
    }

    fn wrap_position(&self, x: usize, y: usize) -> (usize, usize) {
        let wrapped_x = (x + self.map_width) % self.map_width;
        let wrapped_y = (y + self.map_height) % self.map_height;
        (wrapped_x, wrapped_y)
    }

    fn a_star_pathfinding(
        &self,
        start: (usize, usize),
        goal: (usize, usize),
        avoid_fog: bool,
    ) -> Option<Vec<(usize, usize)>> {
        let mut open_set = BinaryHeap::new();
        let mut came_from = std::collections::HashMap::new();
        let mut g_score = std::collections::HashMap::new();
        let mut f_score = std::collections::HashMap::new();
        let mut closed_set = HashSet::new();

        open_set.push(Node {
            cost: 0,
            position: start,
            priority: Self::heuristic(start, goal),
        });

        g_score.insert(start, 0);
        f_score.insert(start, Self::heuristic(start, goal));

        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        while let Some(Node { position, .. }) = open_set.pop() {
            if position == goal {
                let mut path = vec![position];
                let mut current = position;

                while let Some(&prev) = came_from.get(&current) {
                    current = prev;
                    path.push(current);
                }

                path.reverse();
                return Some(path);
            }

            closed_set.insert(position);

            for (dx, dy) in directions.iter() {
                let new_x = (position.0 as isize + dx).max(0) as usize;
                let new_y = (position.1 as isize + dy).max(0) as usize;
                let wrapped_pos = self.wrap_position(new_x, new_y);

                if closed_set.contains(&wrapped_pos)
                    || matches!(self.map[wrapped_pos.1][wrapped_pos.0], Cell::Obstacle)
                    || (avoid_fog && !self.discovered[wrapped_pos.1][wrapped_pos.0])
                {
                    continue;
                }

                let tentative_g_score = g_score.get(&position).unwrap_or(&usize::MAX) + 1;

                if tentative_g_score < *g_score.get(&wrapped_pos).unwrap_or(&usize::MAX) {
                    came_from.insert(wrapped_pos, position);
                    g_score.insert(wrapped_pos, tentative_g_score);
                    f_score.insert(
                        wrapped_pos,
                        tentative_g_score + Self::heuristic(wrapped_pos, goal),
                    );

                    open_set.push(Node {
                        cost: tentative_g_score,
                        position: wrapped_pos,
                        priority: tentative_g_score + Self::heuristic(wrapped_pos, goal),
                    });
                }
            }
        }

        None
    }

    fn heuristic(a: (usize, usize), b: (usize, usize)) -> usize {
        let dx = (a.0 as isize - b.0 as isize).abs() as usize;
        let dy = (a.1 as isize - b.1 as isize).abs() as usize;
        dx + dy
    }

    fn move_robot_towards_target(
        &self,
        x: usize,
        y: usize,
        target_x: usize,
        target_y: usize,
        avoid_fog: bool,
    ) -> (usize, usize) {
        let valid_positions = |pos: (usize, usize)| {
            !matches!(self.map[pos.1][pos.0], Cell::Obstacle)
                && (!avoid_fog || self.discovered[pos.1][pos.0])
        };

        if let Some(path) = self.a_star_pathfinding((x, y), (target_x, target_y), avoid_fog) {
            if path.len() > 1 && valid_positions(path[1]) {
                return path[1];
            }
        }
        (x, y)
    }

    fn move_robot_randomly(
        x: usize,
        y: usize,
        directions: &[(isize, isize)],
        map: &Vec<Vec<Cell>>,
        map_width: usize,
        map_height: usize,
    ) -> (usize, usize) {
        if let Some(&(dx, dy)) = directions.choose(&mut rand::thread_rng()) {
            let new_x = (x as isize + dx).max(0) as usize % map_width;
            let new_y = (y as isize + dy).max(0) as usize % map_height;
            if matches!(
                map[new_y][new_x],
                Cell::Empty | Cell::Energy | Cell::Crystal | Cell::Base
            ) {
                return (new_x, new_y);
            }
        }
        (x, y)
    }

    fn update_robot(&mut self, robot: &mut Robot) {
        if robot.move_counter < robot.speed {
            robot.move_counter += 1;
            return;
        }
        robot.move_counter = 0;

        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        match robot.role {
            Role::Explorer => {
                if let Some(resource_coords) = robot.resource_coords {
                    let (new_x, new_y) = self.move_robot_towards_target(
                        robot.x,
                        robot.y,
                        self.base_position.0,
                        self.base_position.1,
                        false,
                    );
                    robot.x = new_x;
                    robot.y = new_y;

                    if (robot.x, robot.y) == self.base_position {
                        // Robot explorateur passe à robot extracteur et va chercher la ressource
                        robot.role = Role::Extractor;
                        println!(
                            "Déploiement du Robot extracteur, il part chercher la ressource {:?}",
                            resource_coords
                        );
                    }
                } else {
                    // Explore la map et se rappelle de la postion des ressources
                    let (new_x, new_y) = GameState::move_robot_randomly(
                        robot.x,
                        robot.y,
                        &directions,
                        &self.map,
                        self.map_width,
                        self.map_height,
                    );
                    robot.x = new_x;
                    robot.y = new_y;

                    // Marque la position actuelle comme découverte
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let (disc_x, disc_y) = (
                                (robot.x as isize + dx).clamp(0, self.map_width as isize - 1)
                                    as usize,
                                (robot.y as isize + dy).clamp(0, self.map_height as isize - 1)
                                    as usize,
                            );
                            self.discovered[disc_y][disc_x] = true;
                        }
                    }

                    // Check si la ressource est autour du robot
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let (check_x, check_y) = (
                                (robot.x as isize + dx).clamp(0, self.map_width as isize - 1)
                                    as usize,
                                (robot.y as isize + dy).clamp(0, self.map_height as isize - 1)
                                    as usize,
                            );
                            if matches!(self.map[check_y][check_x], Cell::Crystal | Cell::Energy) {
                                let resource_type = self.map[check_y][check_x].clone();
                                robot.resource_coords = Some((check_x, check_y));
                                self.map[check_y][check_x] = match resource_type {
                                    Cell::Crystal => Cell::ReservedCrystal,
                                    Cell::Energy => Cell::ReservedEnergy,
                                    _ => unreachable!(),
                                };
                                println!("Robot explorateur à trouver une ressource {:?}, retour à la base", (check_x, check_y));
                                break;
                            }
                        }
                    }
                }
            }
            Role::Extractor => {
                if let Some(resource_coords) = robot.resource_coords {
                    if robot.carrying.is_none() {
                        // Va chercher la ressource
                        let (new_x, new_y) = self.move_robot_towards_target(
                            robot.x,
                            robot.y,
                            resource_coords.0,
                            resource_coords.1,
                            true,
                        );
                        robot.x = new_x;
                        robot.y = new_y;

                        if (robot.x, robot.y) == resource_coords {
                            // Collecter la ressource
                            robot.speed = Robot::increased_speed();
                            if matches!(
                                self.map[robot.y][robot.x],
                                Cell::ReservedCrystal | Cell::ReservedEnergy
                            ) {
                                robot.carrying = Some(match self.map[robot.y][robot.x] {
                                    Cell::ReservedCrystal => Cell::Crystal,
                                    Cell::ReservedEnergy => Cell::Energy,
                                    _ => unreachable!(),
                                });
                                self.map[robot.y][robot.x] = Cell::Empty;
                                println!("Robot extracteur a récupéré la ressource {:?}, retour à la base", (robot.x, robot.y));
                            }
                        }
                    } else {
                        // REtourne à la base apres avoir extrait
                        let (new_x, new_y) = self.move_robot_towards_target(
                            robot.x,
                            robot.y,
                            self.base_position.0,
                            self.base_position.1,
                            true,
                        );
                        robot.x = new_x;
                        robot.y = new_y;

                        if (robot.x, robot.y) == self.base_position {
                            match robot.carrying {
                                Some(Cell::Crystal) => {
                                    self.crystal_score += 1;
                                    println!(
                                        "Cristal déposé à la base. Score: {}",
                                        self.crystal_score
                                    );
                                }
                                Some(Cell::Energy) => {
                                    self.energy_score += 1;
                                    println!(
                                        "Energie déposée à la base. Score: {}",
                                        self.energy_score
                                    );
                                }
                                _ => {}
                            }
                            // Passe de l'extracteur à l'explorateur
                            robot.role = Role::Explorer;
                            robot.speed = Robot::default_speed();
                            robot.carrying = None;
                            robot.resource_coords = None;
                            println!("Envoie du robot explorateur");
                        }
                    }
                }
            }
        }
    }

    fn check_game_over(&self) -> bool {
        !self.map.iter().any(|row| {
            row.iter().any(|cell| {
                matches!(
                    cell,
                    Cell::Crystal | Cell::Energy | Cell::ReservedCrystal | Cell::ReservedEnergy
                )
            })
        })
    }
}

fn place_randomly(
    map: &mut Vec<Vec<Cell>>,
    rng: &mut StdRng,
    cell_type: Cell,
    quantity: usize,
) -> Option<(usize, usize)> {
    let map_height = map.len();
    let map_width = map[0].len();

    let mut placed_positions = Vec::new();

    for _ in 0..quantity {
        let mut placed = false;
        while !placed {
            let x = rng.gen_range(0..map_width);
            let y = rng.gen_range(0..map_height);
            if matches!(map[y][x], Cell::Empty) {
                map[y][x] = cell_type.clone();
                placed_positions.push((x, y));
                placed = true;
            }
        }
    }

    if !placed_positions.is_empty() {
        Some(placed_positions[0])
    } else {
        None
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.game_over {
            return Ok(());
        }

        // Récuperer les mouvements séparément
        let mut updated_robots = self.robots.clone();

        for robot in &mut updated_robots {
            self.update_robot(robot);
        }

        self.robots = updated_robots;

        // Check si le jeu est finis
        self.game_over = self.check_game_over();

        if self.game_over {
            println!(
                "Fin du jeu! Score final - Cristaux: {}, Energies: {}",
                self.crystal_score, self.energy_score
            );
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));
        let cell_size = 20.0;
        for (y, row) in self.map.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let color = if self.discovered[y][x] {
                    match cell {
                        Cell::Obstacle => graphics::Color::new(0.5, 0.5, 0.5, 1.0),
                        Cell::Energy => graphics::Color::new(1.0, 1.0, 0.0, 1.0),
                        Cell::Crystal => graphics::Color::new(0.5, 0.0, 0.5, 1.0),
                        Cell::Base => graphics::Color::new(1.0, 0.0, 0.0, 1.0),
                        Cell::ReservedEnergy => graphics::Color::new(1.0, 1.0, 0.5, 1.0),
                        Cell::ReservedCrystal => graphics::Color::new(0.75, 0.0, 0.75, 1.0),
                        Cell::Empty => graphics::Color::new(0.0, 0.8, 0.0, 1.0),
                    }
                } else {
                    graphics::Color::new(0.0, 0.0, 0.0, 1.0)
                };

                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new(
                        x as f32 * cell_size,
                        y as f32 * cell_size,
                        cell_size,
                        cell_size,
                    ),
                    color,
                )?;

                graphics::draw(ctx, &rectangle, graphics::DrawParam::default())?;
            }
        }

        for robot in &self.robots {
            let color = match robot.role {
                Role::Explorer => graphics::Color::new(0.0, 0.0, 1.0, 1.0), //Robot explorateur
                Role::Extractor => graphics::Color::new(1.0, 0.65, 0.0, 1.0), // Robot extracteur
            };
            let robot_rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    robot.x as f32 * cell_size,
                    robot.y as f32 * cell_size,
                    cell_size,
                    cell_size,
                ),
                color,
            )?;
            graphics::draw(ctx, &robot_rectangle, graphics::DrawParam::default())?;
        }

        //Affiche le score
        let score_text = format!(
            "Cistaux: {} | Energies: {}",
            self.crystal_score, self.energy_score
        );
        let score_display = graphics::Text::new((score_text, graphics::Font::default(), 18.0));
        graphics::draw(
            ctx,
            &score_display,
            graphics::DrawParam::default().dest([10.0, 10.0]),
        )?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let seed = rand::thread_rng().gen();
    let cb = ggez::ContextBuilder::new("Rust Game", "ggez")
        .window_setup(conf::WindowSetup::default().title("Création de la map"))
        .window_mode(conf::WindowMode::default().dimensions(800.0, 600.0));
    let (mut ctx, event_loop) = cb.build()?;
    let state = GameState::new(&mut ctx, seed)?;
    event::run(ctx, event_loop, state)
}
