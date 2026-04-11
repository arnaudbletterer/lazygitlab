use std::{io, time::Duration, borrow::Cow};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{prelude::*, widgets::*};
use gitlab::{AsyncGitlab, api::{self, AsyncQuery, Endpoint}};
use gitlab::api::endpoint_prelude::Method;
use serde::Deserialize;
use dotenvy::dotenv;

#[derive(Debug, Clone)]
struct AddDiscussionNote {
    project: u64,
    merge_request: u64,
    discussion: String,
    body: String,
}

impl Endpoint for AddDiscussionNote {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/discussions/{}/notes",
            self.project, self.merge_request, self.discussion,
        )
        .into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

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
    id: u64,
    body: String,
    author: Author,
    #[serde(rename = "type")]
    note_type: Option<String>,
    position: Option<Position>,
}

#[derive(Debug, Deserialize, Clone)]
struct Position {
    #[allow(dead_code)]
    base_sha: String,
    #[allow(dead_code)]
    start_sha: String,
    #[allow(dead_code)]
    head_sha: String,
    new_path: Option<String>,
    new_line: Option<u64>,
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
    discussion_state: ListState,
    input_mode: bool,
    reply_input: String,
}

impl App {
    fn new(gitlab: AsyncGitlab) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        
        let mut projects_state = ListState::default();
        projects_state.select(Some(0));

        let mut mrs_state = ListState::default();
        mrs_state.select(Some(0));

        let mut discussion_state = ListState::default();
        discussion_state.select(Some(0));

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
            discussion_state,
            input_mode: false,
            reply_input: String::new(),
        }
    }

    fn move_down(&mut self) {
        if self.input_mode { return; }
        if self.focus_on_menu {
            let i = match self.menu_state.selected() {
                Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 },
                None => 0,
            };
            self.menu_state.select(Some(i));
            self.active_selection = self.menu_items[i].1;
        } else {
            match self.view_mode {
                ViewMode::List => {
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
                ViewMode::Detail => {
                    if !self.discussions.is_empty() {
                        let i = match self.discussion_state.selected() {
                            Some(i) => if i >= self.discussions.len() - 1 { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.discussion_state.select(Some(i));
                    }
                }
            }
        }
    }

    fn move_up(&mut self) {
        if self.input_mode { return; }
        if self.focus_on_menu {
            let i = match self.menu_state.selected() {
                Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 },
                None => 0,
            };
            self.menu_state.select(Some(i));
            self.active_selection = self.menu_items[i].1;
        } else {
            match self.view_mode {
                ViewMode::List => {
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
                ViewMode::Detail => {
                    if !self.discussions.is_empty() {
                        let i = match self.discussion_state.selected() {
                            Some(i) => if i == 0 { self.discussions.len() - 1 } else { i - 1 },
                            None => 0,
                        };
                        self.discussion_state.select(Some(i));
                    }
                }
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
        // Sort or filter if needed
        self.discussions = discussions;
        Ok(())
    }

    async fn post_reply(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.reply_input.is_empty() { return Ok(()); }
        
        if let Some(mr) = &self.selected_mr {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let endpoint = AddDiscussionNote {
                        project: mr.project_id,
                        merge_request: mr.iid,
                        discussion: disc.id.clone(),
                        body: self.reply_input.clone(),
                    };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    self.reply_input.clear();
                    self.input_mode = false;
                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                }
            }
        }
        Ok(())
    }

    async fn enter(&mut self) {
        if self.input_mode {
            let _ = self.post_reply().await;
            return;
        }

        if self.focus_on_menu {
            self.focus_on_menu = false;
        } else {
            match self.view_mode {
                ViewMode::List => {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if let Some(i) = self.mrs_state.selected() {
                                if let Some(mr) = self.merge_requests.get(i) {
                                    self.selected_mr = Some(mr.clone());
                                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                                    self.discussion_state.select(Some(0)); // Default to first discussion
                                    self.view_mode = ViewMode::Detail;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ViewMode::Detail => {
                    self.input_mode = true;
                }
            }
        }
    }

    fn back(&mut self) {
        if self.input_mode {
            self.input_mode = false;
            self.reply_input.clear();
        } else if self.view_mode == ViewMode::Detail {
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
    let _ = app.fetch_merge_requests().await;

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
                    if app.input_mode {
                        match key.code {
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Esc => app.back(),
                            KeyCode::Char(c) => app.reply_input.push(c),
                            KeyCode::Backspace => { app.reply_input.pop(); }
                            _ => {}
                        }
                    } else {
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

    let (sidebar_highlight_style, sidebar_symbol) = if app.focus_on_menu {
        (Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD), ">> ")
    } else {
        (Style::default(), "   ")
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu ").border_style(sidebar_style))
        .highlight_style(sidebar_highlight_style)
        .highlight_symbol(sidebar_symbol);

    f.render_stateful_widget(list, main_chunks[0], &mut app.menu_state);

    // Main Content
    let content_style = if !app.focus_on_menu {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let (content_highlight_style, content_symbol) = if !app.focus_on_menu {
        (Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD), "> ")
    } else {
        (Style::default(), "  ")
    };

    match app.active_selection {
        MenuSelection::Projects => {
            let projects: Vec<ListItem> = app.projects
                .iter()
                .map(|p| ListItem::new(format!("{} ({})", p.name, p.path_with_namespace)))
                .collect();
            
            let projects_list = List::new(projects)
                .block(Block::default().borders(Borders::ALL).title(" Projects ").border_style(content_style))
                .highlight_style(content_highlight_style)
                .highlight_symbol(content_symbol);
            
            f.render_stateful_widget(projects_list, main_chunks[1], &mut app.projects_state);
        }
        MenuSelection::MergeRequests => {
            let mrs: Vec<ListItem> = app.merge_requests
                .iter()
                .map(|mr| ListItem::new(format!("[{}] {} - {} by @{}", mr.state, mr.references.full, mr.title, mr.author.username)))
                .collect();
            
            let mrs_list = List::new(mrs)
                .block(Block::default().borders(Borders::ALL).title(" My Merge Requests (Created) ").border_style(content_style))
                .highlight_style(content_highlight_style)
                .highlight_symbol(content_symbol);
            
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
            .constraints([
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(0),
                Constraint::Length(if app.input_mode { 3 } else { 0 }),
            ])
            .split(f.area());

        let header_text = if app.input_mode {
            " Typing reply... [Enter] Post [Esc] Cancel "
        } else {
            " [j/k] Nav Disc [Enter] Reply [Esc] Back [v] VS Code [r] Refresh [q] Quit "
        };

        let header = Paragraph::new(format!(" LazyGitlab | {}", header_text))
            .block(Block::default().borders(Borders::ALL).title(" Controls "))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, chunks[0]);

        let description = mr.description.as_deref().unwrap_or("No description provided.");
        let mr_info = Paragraph::new(format!(
            "Title: {}\nAuthor: @{}\nState: {}\nURL: {}\n\nDescription: {}",
            mr.title, mr.author.username, mr.state, mr.web_url, description
        ))
        .block(Block::default().borders(Borders::ALL).title(format!(" MR: {} ", mr.references.full)))
        .wrap(Wrap { trim: true });
        f.render_widget(mr_info, chunks[1]);

        let comments_list: Vec<ListItem> = app.discussions
            .iter()
            .map(|d| {
                let mut lines = Vec::new();
                for (i, note) in d.notes.iter().enumerate() {
                    if i == 0 {
                        // Header for the first note in discussion
                        lines.push(Line::from(vec![
                            Span::styled(format!("@{}", note.author.username), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                            Span::raw(" says:"),
                        ]));

                        // If it's a code comment, show the context
                        if let Some(pos) = &note.position {
                            if let (Some(path), Some(line)) = (&pos.new_path, pos.new_line) {
                                lines.push(Line::from(vec![
                                    Span::styled(format!("  Code at {}:{}", path, line), Style::default().fg(Color::Yellow)),
                                ]));
                            }
                        }
                    } else {
                        // Indented reply
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(format!("@{}", note.author.username), Style::default().fg(Color::Blue)),
                            Span::raw(" replied:"),
                        ]));
                    }
                    lines.push(Line::from(format!("  {}", note.body)));
                    lines.push(Line::from(""));
                }
                ListItem::new(lines)
            })
            .collect();

        let comments_style = if !app.input_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let (disc_highlight_style, disc_symbol) = if !app.input_mode {
            (Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD), "> ")
        } else {
            (Style::default(), "  ")
        };

        let comments = List::new(comments_list)
            .block(Block::default().borders(Borders::ALL).title(" Discussions ").border_style(comments_style))
            .highlight_style(disc_highlight_style)
            .highlight_symbol(disc_symbol);

        f.render_stateful_widget(comments, chunks[2], &mut app.discussion_state);
        if app.input_mode {
            let input = Paragraph::new(app.reply_input.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(" Reply "));
            f.render_widget(input, chunks[3]);
        }
    }
}