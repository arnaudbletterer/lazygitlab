use crate::models::{
    App, Context, DetailSection, MenuSelection, Panel, Project, ViewMode,
};

const NAV_WAIT: std::time::Duration = std::time::Duration::from_millis(750);

pub fn menu_index(context: Context, selection: MenuSelection) -> Option<usize> {
    match (context, selection) {
        (Context::Dashboard, MenuSelection::Home) => Some(0),
        (Context::Dashboard, MenuSelection::Todos) => Some(1),
        (Context::Dashboard, MenuSelection::MergeRequests) => Some(2),
        (Context::Dashboard, MenuSelection::Issues) => Some(3),
        (Context::Dashboard, MenuSelection::Pipelines) => Some(4),
        (Context::Explore, MenuSelection::Projects) => Some(0),
        (Context::Explore, MenuSelection::Groups) => Some(1),
        (Context::Project, MenuSelection::MergeRequests) => Some(0),
        (Context::Project, MenuSelection::Issues) => Some(1),
        (Context::Project, MenuSelection::Pipelines) => Some(2),
        (Context::Project, MenuSelection::Repository) => Some(3),
        (Context::Project, MenuSelection::Branches) => Some(4),
        (Context::Project, MenuSelection::Commits) => Some(5),
        (Context::Project, MenuSelection::Boards) => Some(6),
        (Context::Project, MenuSelection::Epics) => Some(7),
        _ => None,
    }
}

pub fn selection_for(context: Context, idx: usize) -> Option<MenuSelection> {
    const DASH: [MenuSelection; 5] = [
        MenuSelection::Home,
        MenuSelection::Todos,
        MenuSelection::MergeRequests,
        MenuSelection::Issues,
        MenuSelection::Pipelines,
    ];
    const PROJECT: [MenuSelection; 8] = [
        MenuSelection::MergeRequests,
        MenuSelection::Issues,
        MenuSelection::Pipelines,
        MenuSelection::Repository,
        MenuSelection::Branches,
        MenuSelection::Commits,
        MenuSelection::Boards,
        MenuSelection::Epics,
    ];
    match context {
        Context::Dashboard => DASH.get(idx).copied(),
        Context::Explore => [MenuSelection::Projects, MenuSelection::Groups]
            .get(idx)
            .copied(),
        Context::Project => PROJECT.get(idx).copied(),
    }
}

impl App {
    pub fn project_nav_rows(&self) -> usize {
        match &self.selected_project {
            Some(p) if p.namespace.as_ref().is_some_and(|n| n.kind == "group") => 8,
            Some(_) => 7,
            None => 0,
        }
    }

    /// A filter changed: empty only the list that filter applies to, then reload it.
    pub async fn apply_filter_change(&mut self) {
        match self.active_selection {
            MenuSelection::MergeRequests => self.clear_mrs(),
            MenuSelection::Issues => self.clear_issues(),
            MenuSelection::Todos => self.clear_todos(),
            _ => {}
        }
        self.filter_lists();
        if matches!(
            self.active_selection,
            MenuSelection::MergeRequests | MenuSelection::Issues | MenuSelection::Todos
        ) {
            self.nav_wait = Some(std::time::Instant::now());
            self.refresh().await;
        }
    }

    /// Context a project-aware jump (`g m`, help, Home keys) lands in: a project in scope
    /// always means project context.
    pub fn jump_context(&self) -> Context {
        if self.selected_project.is_some() {
            Context::Project
        } else {
            Context::Dashboard
        }
    }

    pub fn scope_for(&self, context: Context) -> Option<u64> {
        if context == Context::Project {
            self.selected_project.as_ref().map(|p| p.id)
        } else {
            None
        }
    }

    /// True while the list of the current screen is expected to arrive.
    pub fn list_loading(&self) -> bool {
        self.is_loading || self.nav_wait.is_some_and(|t| t.elapsed() < NAV_WAIT)
    }

    pub async fn navigate(&mut self, context: Context, selection: MenuSelection) {
        self.navigate_to(context, selection, Panel::Main).await;
    }

    pub async fn navigate_to(&mut self, context: Context, selection: MenuSelection, panel: Panel) {
        if context == Context::Project && self.selected_project.is_none() {
            return;
        }
        self.active_context = context;
        self.active_selection = selection;
        self.active_panel = panel;
        if let Some(idx) = menu_index(context, selection) {
            match context {
                Context::Dashboard => self.dashboard_menu_state.select(Some(idx)),
                Context::Explore => self.explore_menu_state.select(Some(idx)),
                Context::Project => self.project_menu_state.select(Some(idx)),
            }
        }
        self.enter_screen().await;
        self.update_preview().await;
    }

    /// Jump to a screen: project screens need a project in scope. Without one, Home
    /// cards and keys open the project picker instead of inventing a project context.
    pub async fn jump(&mut self, selection: MenuSelection) {
        match selection {
            MenuSelection::MergeRequests
            | MenuSelection::Issues
            | MenuSelection::Pipelines => {
                let ctx = self.jump_context();
                self.navigate(ctx, selection).await;
            }
            _ if self.selected_project.is_some() => {
                self.navigate(Context::Project, selection).await;
            }
            _ => {
                if self.active_selection == MenuSelection::Home {
                    self.switch_project().await;
                }
            }
        }
    }

    pub async fn open_project(&mut self, project: Project) {
        self.show_project_popup = false;
        self.set_project(project);
        self.navigate_to(Context::Project, MenuSelection::MergeRequests, Panel::Project)
            .await;
    }

    pub fn set_project(&mut self, project: Project) {
        let changed = self.selected_project.as_ref().map(|p| p.id) != Some(project.id);
        self.selected_project = Some(project);
        if changed {
            self.page_in_flight.clear();
            self.reset_project_scope();
            self.clear_mrs();
            self.clear_issues();
            self.clear_pipelines();
            self.mrs_scope = self.selected_project.as_ref().map(|p| p.id);
            self.issues_scope = self.mrs_scope;
            self.pipelines_scope = self.mrs_scope;
            self.repository_ref = None;
            self.repository_tree.clear();
            self.project_branches.clear();
            self.selected_file_content = None;
            self.file_scroll = 0;
            self.file_highlight_cache = None;
        }
    }

    pub(crate) fn clear_mrs(&mut self) {
        self.merge_requests.clear();
        self.mrs_cached_at = None;
        self.mrs_stale = false;
        self.mrs_sync = Default::default();
        self.mr_meta.clear();
        self.mrs_page = 1;
        self.mrs_has_more = true;
    }

    pub(crate) fn clear_issues(&mut self) {
        self.issues.clear();
        self.issues_page = 1;
        self.issues_has_more = true;
    }

    pub(crate) fn clear_pipelines(&mut self) {
        self.pipelines.clear();
        self.pipelines_page = 1;
        self.pipelines_has_more = true;
    }

    pub(crate) fn clear_todos(&mut self) {
        self.todos.clear();
        self.todos_etag = None;
        self.todos_etag_state = None;
        self.todos_page = 1;
        self.todos_has_more = true;
    }

    /// Drops everything that belongs to the previously viewed screen's detail pane.
    pub(crate) fn reset_view_state(&mut self) {
        self.selected_mr = None;
        self.selected_issue = None;
        self.selected_pipeline = None;
        self.selected_job = None;
        self.discussions.clear();
        self.discussions_key = None;
        self.diffs_key = None;
        self.mr_diffs.clear();
        self.is_fetching_diffs = false;
        self.jobs.clear();
        self.clear_related_items();
        self.detail_section = DetailSection::Metadata;
        self.view_mode = ViewMode::List;
        self.open_job_when_loaded = None;
        self.abort_mr_detail();
    }

    /// Empties the list of the destination screen only when its scope changed.
    pub(crate) fn reconcile_scope(&mut self) {
        let scope = self.scope_for(self.active_context);
        match self.active_selection {
            MenuSelection::MergeRequests if self.mrs_scope != scope => {
                self.clear_mrs();
                self.mrs_scope = scope;
            }
            MenuSelection::Issues if self.issues_scope != scope => {
                self.clear_issues();
                self.issues_scope = scope;
            }
            MenuSelection::Pipelines
                if self.active_context == Context::Project && self.pipelines_scope != scope =>
            {
                self.clear_pipelines();
                self.pipelines_scope = scope;
            }
            _ => {}
        }
    }

    fn destination_has_rows(&self) -> bool {
        match self.active_selection {
            MenuSelection::MergeRequests => !self.merge_requests.is_empty(),
            MenuSelection::Issues => !self.issues.is_empty(),
            MenuSelection::Pipelines => {
                if self.active_context == Context::Dashboard {
                    !self.cross_pipelines.is_empty()
                } else {
                    !self.pipelines.is_empty()
                }
            }
            MenuSelection::Todos => !self.todos.is_empty(),
            MenuSelection::Projects => !self.projects.is_empty(),
            MenuSelection::Groups => !self.groups.is_empty(),
            _ => false,
        }
    }

    /// The single transition every screen change goes through.
    pub(crate) async fn enter_screen(&mut self) {
        self.reset_view_state();
        self.reconcile_scope();
        self.filter_lists();
        if self.destination_has_rows() {
            self.refresh_background().await;
        } else {
            self.nav_wait = Some(std::time::Instant::now());
            self.refresh().await;
        }
    }
}
