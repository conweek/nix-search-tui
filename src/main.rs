mod app;
mod ui;

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::{App, CurrScreen};
use ui::ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;

    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(do_print) = res {
        if do_print {
            app.print_results();
        }
    } else if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            // Global quit
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(false);
            }

            match app.current_screen {
                CurrScreen::Searching => match key.code {
                    KeyCode::Tab => {
                        app.cycle_tab();
                    }
                    KeyCode::Enter => {
                        app.results.clear();
                        let _ = app.search();
                        app.selected_result = 0;
                        app.current_screen = CurrScreen::DisplayResults;
                    }
                    KeyCode::Esc => {
                        return Ok(false);
                    }
                    KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Pop off the last word in the search
                        if let Some((rest, _last)) = app.search_option.rsplit_once(' ') {
                            app.search_option = rest.to_string();
                        } else {
                            app.search_option = String::new();
                        }
                    }
                    KeyCode::Backspace => {
                        app.search_option.pop();
                    }
                    KeyCode::Char(c) => {
                        app.search_option.push(c);
                    }
                    _ => {}
                },
                CurrScreen::DisplayResults => match key.code {
                    KeyCode::Char('s') | KeyCode::Char('/') => {
                        app.current_screen = CurrScreen::Both;
                    }
                    KeyCode::Enter => {
                        match app.search_choice {
                            app::Search::Configuration | app::Search::HomeConfiguration => {
                                if app.results_from_config {
                                    let _ = app.fetch_detail();
                                    app.current_screen = CurrScreen::Detail;
                                } else {
                                    app.results.clear();
                                    let _ = app.search();
                                    app.selected_result = 0;
                                }
                            }
                            app::Search::Package => {
                                if !app.results_from_config {
                                    // already viewing package results, do nothing
                                } else {
                                    app.results.clear();
                                    let _ = app.search();
                                    app.selected_result = 0;
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        return Ok(false);
                    }
                    KeyCode::Tab => {
                        app.cycle_tab();
                    }
                    KeyCode::Up => {
                        app.selected_result = app.selected_result.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if app.selected_result + 1 < app.results.len() {
                            app.selected_result += 1;
                        }
                    }
                    _ => {}
                },
                CurrScreen::Detail => match key.code {
                    KeyCode::Esc => {
                        app.current_screen = CurrScreen::DisplayResults;
                    }
                    _ => {}
                },
                CurrScreen::Both => match key.code {
                    KeyCode::Enter => {
                        app.results.clear();
                        let _ = app.search();
                        app.selected_result = 0;
                        app.current_screen = CurrScreen::DisplayResults;
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrScreen::DisplayResults;
                    }
                    KeyCode::Tab => {
                        app.cycle_tab();
                    }
                    KeyCode::Backspace => {
                        app.search_option.pop();
                    }
                    KeyCode::Up => {
                        app.selected_result = app.selected_result.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if app.selected_result + 1 < app.results.len() {
                            app.selected_result += 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.search_option.push(c);
                    }
                    _ => {}
                },
            }
        }
    }
}
