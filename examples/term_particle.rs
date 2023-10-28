use crossterm::{
    cursor, execute,
    style::{self, StyledContent, Stylize},
    terminal, QueueableCommand,
};
use multimap::MultiMap;
use rand::{distributions::Standard, prelude::Distribution, random, Rng};
use std::io::{self, Error, ErrorKind, Write};
use std::{thread, time};

fn background() -> StyledContent<char> {
    '░'.blue()
}

type Position = (i16, i16);

#[allow(dead_code)]
enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    fn get_mvt(&self) -> Position {
        match self {
            Direction::N => (0, -1),
            Direction::NE => (1, -1),
            Direction::E => (1, 0),
            Direction::SE => (1, 1),
            Direction::S => (0, 1),
            Direction::SW => (-1, 1),
            Direction::W => (-1, 0),
            Direction::NW => (-1, -1),
        }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..8) {
            0 => Direction::N,
            1 => Direction::NE,
            2 => Direction::E,
            3 => Direction::SE,
            4 => Direction::S,
            5 => Direction::SW,
            6 => Direction::W,
            _ => Direction::NW,
        }
    }
}

struct Particle {
    position: Position,
    direction: Position,
}

impl Particle {
    fn new(position: Position, direction: Position) -> Self {
        Self {
            position,
            direction,
        }
    }

    fn print(&self, stdout: &mut dyn Write, c: StyledContent<char>) -> io::Result<()> {
        let (x, y) = self.position;
        let (console_width, console_height) = terminal::size()?;
        if let (Ok(x_), Ok(y_)) = (x.try_into(), y.try_into()) {
            if x_ < console_width && y_ < console_height {
                stdout.queue(cursor::MoveTo(x_, y_))?;
                stdout.queue(style::PrintStyledContent(c))?;
                return Ok(());
            }
        }
        Err(Error::new(
            ErrorKind::Other,
            format!("out of bound {}, {}", x, y),
        ))
    }

    fn draw(&self, stdout: &mut dyn Write) -> io::Result<()> {
        if self.position.0 % 2 == self.position.1 % 2 {
            self.print(stdout, '▚'.green())
        } else {
            self.print(stdout, '▞'.green())
        }
    }

    fn erase(&self, stdout: &mut dyn Write) -> io::Result<()> {
        self.print(stdout, background())
    }

    fn update(&mut self) {
        self.position = (
            self.position.0 + self.direction.0,
            self.position.1 + self.direction.1,
        );
    }
}

fn compute_world(mut particles: Vec<Particle>, limits: (u16, u16)) -> Vec<Particle> {
    for particle in particles.iter_mut() {
        particle.update();
    }
    let mut map = particles.into_iter().fold(MultiMap::new(), |mut map, p| {
        map.insert(p.position, p);
        map
    });
    map.iter_all_mut()
        .fold(Vec::new(), |mut result, (pos, particles_here)| {
            if pos.0 >= 0
                && pos.0 < limits.0.try_into().unwrap()
                && pos.1 >= 0
                && pos.1 < limits.1.try_into().unwrap()
            {
                if particles_here.len() == 1 {
                    result.push(particles_here.pop().unwrap());
                } else if particles_here.len() == 2 {
                    let p1 = particles_here.pop().unwrap();
                    let p2 = particles_here.pop().unwrap();
                    let new_part_dir = (
                        (p1.direction.0 + p2.direction.0) % 2,
                        (p1.direction.1 + p2.direction.1) % 2,
                    );
                    if new_part_dir != (0, 0) {
                        result.push(Particle::new((pos.0, pos.1), new_part_dir));
                        while let Some(p) = particles_here.pop() {
                            result.push(p);
                        }
                    }
                }
            }
            result
        })
}

fn erase_particles(particles: &[Particle], stdout: &mut dyn Write) -> io::Result<()> {
    for particle in particles.iter() {
        let _ = particle.erase(stdout);
    }
    Ok(())
}

fn draw_particles(particles: &[Particle], stdout: &mut dyn Write) -> io::Result<()> {
    for particle in particles.iter() {
        particle.draw(stdout)?;
    }
    stdout.flush()
}

macro_rules! init_term {
    ( $stdout: ident ) => {{
        execute!(
            $stdout,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
        )?;
    }};
}

macro_rules! reset_term {
    ( $stdout: ident ) => {{
        execute!(
            $stdout,
            cursor::Show,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
        )?;
    }};
}

fn draw_world(stdout: &mut dyn Write, console_width: u16, console_height: u16) -> io::Result<()> {
    for x in 0..console_width {
        for y in 0..console_height {
            stdout
                .queue(cursor::MoveTo(x, y))?
                .queue(style::PrintStyledContent(background()))?;
        }
    }
    stdout.flush()
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let (console_width, console_height) = terminal::size()?;
    init_term!(stdout);
    if let Err(e) = draw_world(&mut stdout, console_width, console_height) {
        reset_term!(stdout);
        eprintln!("{}", e);
    }
    let mut particles = Vec::<Particle>::new();
    for _ in 0..30_000 {
        particles.push(Particle::new(
            (
                (random::<u16>() % console_width) as i16,
                (random::<u16>() % console_height) as i16,
            ),
            random::<Direction>().get_mvt(),
        ));
    }
    particles = compute_world(particles, (console_width, console_height));
    if let Err(e) = draw_particles(&particles, &mut stdout) {
        reset_term!(stdout);
        eprintln!("{}", e);
    }
    thread::sleep(time::Duration::from_millis(1000));
    while !particles.is_empty() {
        let _ = erase_particles(&particles, &mut stdout);
        particles = compute_world(particles, (console_width, console_height));
        if let Err(e) = draw_particles(&particles, &mut stdout) {
            reset_term!(stdout);
            eprintln!("{}", e);
        }
        if let Err(e) = stdout.flush() {
            reset_term!(stdout);
            eprintln!("{}", e);
        }
        thread::sleep(time::Duration::from_millis(50));
    }
    reset_term!(stdout);
    Ok(())
}
