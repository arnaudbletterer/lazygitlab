use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{prelude::*, widgets::*};
use gitlab::{AsyncGitlab, api::{self, AsyncQuery}};
use serde::Deserialize;
use dotenvy::dotenv;

#[derive(Debug, Deserialize, Clone)]
struct Project {
    #[allow(dead_code)]
    id: u64,
    name: String,
    path_with_namespace: String,
}

#[derive(Debug, Deserialize, Clone)]
struct MergeRequest {
    #[allow(dead_code)]
    id: u64,
    iid: u64,
    project_id: u64,
    title: String,
    description: Option<String>,
    author: Author,
    web_url: String,
    state: String,
    references: References,
}

#[derive(Debug, Deserialize, Clone)]
struct Discussion {
    id: String,
    notes: Vec<Note>,
}

#[derive(Debug, Deserialize, Clone)]
struct Note {
    body: String,
    author: Author,
}

#[derive(Debug, Deserialize, Clone)]
struct References {
    full: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Author {
    username: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MenuSelection {
    Projects,
    MergeRequests,
    Issues,
    Pipelines,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ViewMode {
    List,
    Detail,
}

struct App {
    should_quit: bool,
    menu_state: ListState,
    menu_items: Vec<(String, MenuSelection)>,
    active_selection: MenuSelection,
    gitlab: AsyncGitlab,
    projects: Vec<Project>,
    projects_state: ListState,
    merge_requests: Vec<MergeRequest>,
    mrs_state: ListState,
    focus_on_menu: bool,
    view_mode: ViewMode,
    selected_mr: Option<MergeRequest>,
    discussions: Vec<Discussion>,
}

impl App {
    fn new(gitlab: AsyncGitlab) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        
        let mut projects_state = ListState::default();
        projects_state.select(Some(0));

        let mut mrs_state = ListState::default();
        mrs_state.select(Some(0));

        Self {
            should_quit: false,
            menu_state,
            menu_items: vec![
                ("Projects".to_string(), MenuSelection::Projects),
                ("Merge Requests".to_string(), MenuSelection::MergeRequests),
                ("Issues".to_string(), MenuSelection::Issues),
                ("Pipelines".to_string(), MenuSelection::Pipelines),
            ],
            active_selection: MenuSelection::Projects,
            gitlab,
            projects: Vec::new(),
            projects_state,
            merge_requests: Vec::new(),
            mrs_state,
            focus_on_menu: true,
            view_mode: ViewMode::List,
            selected_mr: None,
            discussions: Vec::new(),
        }
    }

    fn move_down(&mut self) {
        if self.focus_on_menu {
            let i = match self.menu_state.selected() {
                Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 },
                None => 0,
            };
            self.menu_state.select(Some(i));
            self.active_selection = self.menu_items[i].1;
        } else {
            match self.active_selection {
                MenuSelection::Projects => {
                    if !self.projects.is_empty() {
                        let i = match self.projects_state.selected() {
                            Some(i) => if i >= self.projects.len() - 1 { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.projects_state.select(Some(i));
                    }
                }
                MenuSelection::MergeRequests => {
                    if !self.merge_requests.is_empty() {
                        let i = match self.mrs_state.selected() {
                            Some(i) => if i >= self.merge_requests.len() - 1 { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.mrs_state.select(Some(i));
                    }
                }
                _ => {}
            }
        }
    }

    fn move_up(&mut self) {
        if self.focus_on_menu {
            let i = match self.menu_state.selected() {
                Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 },
                None => 0,
            };
            self.menu_state.select(Some(i));
            self.active_selection = self.menu_items[i].1;
        } else {
            match self.active_selection {
                MenuSelection::Projects => {
                    if !self.projects.is_empty() {
                        let i = match self.projects_state.selected() {
                            Some(i) => if i == 0 { self.projects.len() - 1 } else { i - 1 },
                            None => 0,
                        };
                        self.projects_state.select(Some(i));
                    }
                }
                MenuSelection::MergeRequests => {
                    if !self.merge_requests.is_empty() {
                        let i = match self.mrs_state.selected() {
                            Some(i) => if i == 0 { self.merge_requests.len() - 1 } else { i - 1 },
                            None => 0,
                        };
                        self.mrs_state.select(Some(i));
                    }
                }
                _ => {}
            }
        }
    }

    fn toggle_focus(&mut self) {
        self.focus_on_menu = !self.focus_on_menu;
    }

    async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = api::projects::Projects::builder()
            .membership(true)
            .build()?;
        let projects: Vec<Project> = endpoint.query_async(&self.gitlab).await?;
        self.projects = projects;
        Ok(())
    }

    async fn fetch_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = api::merge_requests::MergeRequests::builder()
            .scope(api::merge_requests::MergeRequestScope::CreatedByMe)
            .state(api::merge_requests::MergeRequestState::Opened)
            .build()?;
        let mrs: Vec<MergeRequest> = endpoint.query_async(&self.gitlab).await?;
        self.merge_requests = mrs;
        Ok(())
    }

    async fn fetch_discussions(&mut self, project_id: u64, mr_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = api::projects::merge_requests::discussions::MergeRequestDiscussions::builder()
            .project(project_id)
            .merge_request(mr_iid)
            .build()?;
        let discussions: Vec<Discussion> = endpoint.query_async(&self.gitlab).await?;
        self.discussions = discussions;
        Ok(())
    }

    async fn enter(&mut self) {
        if self.focus_on_menu {
            self.focus_on_menu = false;
        } else {
            match self.active_selection {
                MenuSelection::MergeRequests => {
                    if let Some(i) = self.mrs_state.selected() {
                        if let Some(mr) = self.merge_requests.get(i) {
                            self.selected_mr = Some(mr.clone());
                            let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                            self.view_mode = ViewMode::Detail;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn back(&mut self) {
        if self.view_mode == ViewMode::Detail {
            self.view_mode = ViewMode::List;
            self.selected_mr = None;
            self.discussions.clear();
        } else if !self.focus_on_menu {
            self.focus_on_menu = true;
        }
    }

    fn open_in_vscode(&self) {
        if let Some(mr) = &self.selected_mr {
            // Using the browser to open the GitLab MR which can then trigger VS Code
            // or use a direct vscode URI if preferred.
            // For now, let's open the web_url.
            let _ = std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!("Start-Process '{}'", mr.web_url))
                .spawn();
        }
    }

    async fn refresh(&mut self) {
        match self.active_selection {
            MenuSelection::Projects => { let _ = self.fetch_projects().await; }
            MenuSelection::MergeRequests => { 
                if self.view_mode == ViewMode::Detail {
                    if let Some(mr) = &self.selected_mr {
                        let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                    }
                } else {
                    let _ = self.fetch_merge_requests().await; 
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "gitlab.com".to_string());
    let token = std::env::var("GITLAB_TOKEN").expect("GITLAB_TOKEN must be set");

    let client = gitlab::GitlabBuilder::new(url, token)
        .build_async()
        .await?;

    // --- SETUP ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client);
    
    // Initial fetch
    let _ = app.fetch_projects().await;

    // Pass execution to the app loop
    let res = run_app(&mut terminal, &mut app).await;

    // --- RESTORE ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Application Error: {err}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>> 
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
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => app.toggle_focus(),
                        KeyCode::Char('r') => app.refresh().await,
                        KeyCode::Enter => app.enter().await,
                        KeyCode::Backspace | KeyCode::Esc => app.back(),
                        KeyCode::Char('v') => app.open_in_vscode(),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    if app.view_mode == ViewMode::Detail {
        render_detail_view(f, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    let header = Paragraph::new(" LazyGitlab | [j/k] Nav [Tab] Switch [Enter] Detail [r] Refresh [q] Quit ")
        .block(Block::default().borders(Borders::ALL).title(" Controls "))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(chunks[1]);

    // Sidebar
    let items: Vec<ListItem> = app.menu_items
        .iter()
        .map(|(name, _)| ListItem::new(name.as_str()))
        .collect();

    let sidebar_style = if app.focus_on_menu {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu ").border_style(sidebar_style))
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.menu_state);

    // Main Content
    let content_style = if !app.focus_on_menu {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    match app.active_selection {
        MenuSelection::Projects => {
            let projects: Vec<ListItem> = app.projects
                .iter()
                .map(|p| ListItem::new(format!("{} ({})", p.name, p.path_with_namespace)))
                .collect();
            
            let projects_list = List::new(projects)
                .block(Block::default().borders(Borders::ALL).title(" Projects ").border_style(content_style))
                .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            
            f.render_stateful_widget(projects_list, main_chunks[1], &mut app.projects_state);
        }
        MenuSelection::MergeRequests => {
            let mrs: Vec<ListItem> = app.merge_requests
                .iter()
                .map(|mr| ListItem::new(format!("[{}] {} - {} by @{}", mr.state, mr.references.full, mr.title, mr.author.username)))
                .collect();
            
            let mrs_list = List::new(mrs)
                .block(Block::default().borders(Borders::ALL).title(" My Merge Requests (Created) ").border_style(content_style))
                .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            
            f.render_stateful_widget(mrs_list, main_chunks[1], &mut app.mrs_state);
        }
        _ => {
            let placeholder = Paragraph::new(format!("Content for {:?}", app.active_selection))
                .block(Block::default().borders(Borders::ALL).title(" Content ").border_style(content_style));
            f.render_widget(placeholder, main_chunks[1]);
        }
    }
}

fn render_detail_view(f: &mut Frame, app: &mut App) {
    if let Some(mr) = &app.selected_mr {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(f.area());

        let header = Paragraph::new(" LazyGitlab | [Esc/Backspace] Back [v] VS Code [r] Refresh [q] Quit ")
            .block(Block::default().borders(Borders::ALL).title(" Controls "))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, chunks[0]);

        let details_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(chunks[1]);

        let description = mr.description.as_deref().unwrap_or("No description provided.");
        let mr_info = Paragraph::new(format!(
            "Title: {}\nAuthor: @{}\nState: {}\nURL: {}\n\nDescription:\n{}",
            mr.title, mr.author.username, mr.state, mr.web_url, description
        ))
        .block(Block::default().borders(Borders::ALL).title(format!(" MR: {} ", mr.references.full)))
        .wrap(Wrap { trim: true });
        f.render_widget(mr_info, details_chunks[0]);

        let comments_list: Vec<ListItem> = app.discussions
            .iter()
            .flat_map(|d| &d.notes)
            .map(|n| {
                let text = vec![
                    Line::from(vec![
                        Span::styled(format!("@{}", n.author.username), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                        Span::raw(" says:"),
                    ]),
                    Line::from(n.body.as_str()),
                    Line::from(""),
                ];
                ListItem::new(text)
            })
            .collect();

        let comments = List::new(comments_list)
            .block(Block::default().borders(Borders::ALL).title(" Discussions "));
        
        f.render_widget(comments, details_chunks[1]);
    }
}