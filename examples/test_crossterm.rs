use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::char::from_u32;
use std::io::{self, Write};
use std::{thread, time};

const CONSOLE_WIDTH: u16 = 160;
const CONSOLE_HEIGHT: u16 = 65;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let bloc_id = 0x2500;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    for x in 0..CONSOLE_WIDTH {
        for y in 0..CONSOLE_HEIGHT {
            stdout
                .queue(cursor::MoveTo(x, y))?
                .queue(style::PrintStyledContent("░".magenta()))?;
        }
    }
    stdout.flush()?;
    let ten_millis = time::Duration::from_millis(1000);
    thread::sleep(ten_millis);
    for i in 0x80..0x9F {
        for y in 0..20 {
            for x in 0..20 {
                stdout.execute(cursor::MoveTo(
                    x + CONSOLE_WIDTH / 2 - 10,
                    y + CONSOLE_HEIGHT / 2 - 10,
                ))?;
                stdout.execute(style::PrintStyledContent(
                    format!("{}", from_u32(bloc_id + i).unwrap()).red(),
                ))?;
            }
        }
        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);
    }
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;
    Ok(())
}
