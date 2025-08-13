use std::io;

use crossterm::{cursor, QueueableCommand};

pub trait Printable {
  type Context;

  fn print(&self, ctx: Self::Context) -> io::Result<()>;

  fn print_at(&self, ctx: Self::Context, row: u16, col: u16) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.queue(cursor::MoveTo(col, row))?;
    self.print(ctx)?;
    Ok(())
  }
}
