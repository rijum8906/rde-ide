use domain::error::{IdeError, Result};
use tui::ui::Ui;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Initialize Ratatui terminal abstraction
    let mut terminal = ratatui::init();

    // Create UI instance
    let mut ui = match Ui::new() {
        Ok(u) => u,
        Err(e) => {
            ratatui::restore();
            return Err(IdeError::UiError(e.to_string()));
        }
    };

    // Run application loop
    let result = ui.run(&mut terminal);

    // Restore terminal state safely on exit
    ratatui::restore();

    result?;
    Ok(())
}
