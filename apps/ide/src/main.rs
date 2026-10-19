use domain::error::{IdeError, Result};
use rde_ide::app::App;

fn main() -> Result<()> {
    // Initialize Ratatui terminal abstraction
    let mut terminal = ratatui::init();

    // Create UI instance
    let mut app = match App::new() {
        Ok(u) => u,
        Err(e) => {
            ratatui::restore();
            return Err(IdeError::AppError(e.to_string()));
        }
    };

    // Run application loop
    let result = app.run(&mut terminal);

    // Restore terminal state safely on exit
    ratatui::restore();

    result?;
    Ok(())
}
