use std::io;

use crossterm::{cursor, QueueableCommand};

pub trait Printable {
  fn print(&self) -> io::Result<()>;

  fn print_at(&self, row: u16, col: u16) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::MoveTo(col, row))?;
    self.print()?;
    Ok(())
  }
}
