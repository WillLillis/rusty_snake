mod snake;

use std::io::Write;

use console::Term;
use snake::play;

#[allow(dead_code)]
fn main_menu(mut term: Term) -> anyhow::Result<()> {
    let (height, width) = term.size();

    term.move_cursor_to(width as usize / 2, height as usize / 2)?;
    term.write_all("SNAKE".as_bytes())?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    term.hide_cursor()?;
    // main_menu(&mut term);
    play(term.clone())?;

    Ok(())
}
