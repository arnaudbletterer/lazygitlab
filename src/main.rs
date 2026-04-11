use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{prelude::*, widgets::*};

struct App {
    should_quit: bool,
    menu_state: ListState,
    menu_items: Vec<String>,
}

impl App {
    fn new() -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        
        Self {
            should_quit: false,
            menu_state,
            menu_items: vec![
                "Merge Requests".to_string(),
                "Issues".to_string(),
                "Pipelines".to_string(),
                "Projects".to_string(),
            ],
        }
    }

    fn move_down(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.menu_state.select(Some(i));
    }

    fn move_up(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.menu_state.select(Some(i));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- SETUP ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    
    // Pass execution to the app loop
    let res = run_app(&mut terminal, &mut app);

    // --- RESTORE ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Application Error: {err}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>> 
where 
    <B as Backend>::Error: 'static 
{
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area()); // Fixed: Using .area() instead of .size()

    let header = Paragraph::new(" LazyGitlab | [j/k] Navigate [q] Quit ")
        .block(Block::default().borders(Borders::ALL).title(" Controls "))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = app.menu_items
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu "))
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.menu_state);
}