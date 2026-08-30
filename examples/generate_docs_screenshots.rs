use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn color_to_hex(color: Color, is_bg: bool, is_light: bool) -> String {
    if is_light {
        match color {
            Color::Reset => {
                if is_bg {
                    "#ffffff".to_string()
                } else {
                    "#24292f".to_string()
                }
            }
            Color::Black => "#24292f".to_string(),
            Color::Red | Color::LightRed => "#cf222e".to_string(),
            Color::Green | Color::LightGreen => "#1a7f37".to_string(),
            Color::Yellow | Color::LightYellow => "#9a6700".to_string(),
            Color::Blue | Color::LightBlue => "#0969da".to_string(),
            Color::Magenta | Color::LightMagenta => "#8250df".to_string(),
            Color::Cyan | Color::LightCyan => "#0550ae".to_string(),
            Color::Gray => "#57606a".to_string(),
            Color::DarkGray => "#8c959f".to_string(),
            Color::White => {
                if is_bg {
                    "#ffffff".to_string()
                } else {
                    "#24292f".to_string()
                }
            }
            Color::Rgb(100, 100, 200) => {
                if is_bg {
                    "#ddf4ff".to_string()
                } else {
                    "#0969da".to_string()
                }
            }
            Color::Rgb(r, g, b) => {
                if is_bg {
                    "#e8f0fe".to_string()
                } else {
                    format!(
                        "#{:02x}{:02x}{:02x}",
                        r.saturating_sub(60),
                        g.saturating_sub(60),
                        b.saturating_sub(60)
                    )
                }
            }
            Color::Indexed(_) => {
                if is_bg {
                    "#ffffff".to_string()
                } else {
                    "#24292f".to_string()
                }
            }
        }
    } else {
        match color {
            Color::Reset => {
                if is_bg {
                    "#181825".to_string()
                } else {
                    "#cdd6f4".to_string()
                }
            }
            Color::Black => "#11111b".to_string(),
            Color::Red => "#f38ba8".to_string(),
            Color::Green => "#a6e3a1".to_string(),
            Color::Yellow => "#f9e2af".to_string(),
            Color::Blue => "#89b4fa".to_string(),
            Color::Magenta => "#cba6f7".to_string(),
            Color::Cyan => "#89dceb".to_string(),
            Color::Gray => "#bac2de".to_string(),
            Color::DarkGray => "#585b70".to_string(),
            Color::LightRed => "#eba0ac".to_string(),
            Color::LightGreen => "#94e2d5".to_string(),
            Color::LightYellow => "#f9e2af".to_string(),
            Color::LightBlue => "#74c7ec".to_string(),
            Color::LightMagenta => "#f5c2e7".to_string(),
            Color::LightCyan => "#b4befe".to_string(),
            Color::White => "#ffffff".to_string(),
            Color::Rgb(100, 100, 200) => {
                if is_bg {
                    "#313244".to_string()
                } else {
                    "#89b4fa".to_string()
                }
            }
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Color::Indexed(_) => {
                if is_bg {
                    "#181825".to_string()
                } else {
                    "#cdd6f4".to_string()
                }
            }
        }
    }
}

const JETBRAINS_MONO_REGULAR: &[u8] =
    include_bytes!("../docs/assets/fonts/JetBrainsMono-Regular.ttf");
const JETBRAINS_MONO_BOLD: &[u8] =
    include_bytes!("../docs/assets/fonts/JetBrainsMono-Bold.ttf");

fn generate_svg(buffer: &Buffer, filename: &str, is_light: bool) {
    let cols = buffer.area().width as usize;
    let rows = buffer.area().height as usize;

    let cell_w = 9;
    let cell_h = 19;
    let header_h = 36;
    let pad = 24;

    let term_w = cols * cell_w;
    let term_h = rows * cell_h;
    let total_w = term_w + pad * 2;
    let total_h = term_h + header_h + pad * 2;

    use base64::Engine;
    let b64_regular = base64::engine::general_purpose::STANDARD
        .encode(JETBRAINS_MONO_REGULAR);
    let b64_bold =
        base64::engine::general_purpose::STANDARD.encode(JETBRAINS_MONO_BOLD);

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        total_w, total_h, total_w, total_h
    ));

    let shadow_opacity = if is_light { "0.18" } else { "0.6" };
    let shadow_color = "#000000";
    let container_bg = if is_light { "#ffffff" } else { "#181825" };
    let container_stroke = if is_light {
        "rgba(0, 0, 0, 0.12)"
    } else {
        "rgba(255, 255, 255, 0.12)"
    };
    let header_bg = if is_light { "#ebebef" } else { "#1e1e2e" };
    let header_line = if is_light { "#d0d7de" } else { "#13141a" };

    svg.push_str("<defs>\n");
    svg.push_str("  <filter id=\"macos-shadow\" x=\"-8%\" y=\"-8%\" width=\"116%\" height=\"120%\">\n");
    svg.push_str(&format!("    <feDropShadow dx=\"0\" dy=\"20\" stdDeviation=\"24\" flood-color=\"{}\" flood-opacity=\"{}\"/>\n", shadow_color, shadow_opacity));
    svg.push_str(&format!("    <feDropShadow dx=\"0\" dy=\"4\" stdDeviation=\"8\" flood-color=\"{}\" flood-opacity=\"0.12\"/>\n", shadow_color));
    svg.push_str("  </filter>\n");
    svg.push_str("  <style>\n");
    svg.push_str(&format!(
        "    @font-face {{ font-family: 'JetBrains Mono'; src: url('data:font/truetype;charset=utf-8;base64,{}') format('truetype'); font-weight: normal; font-style: normal; }}\n",
        b64_regular
    ));
    svg.push_str(&format!(
        "    @font-face {{ font-family: 'JetBrains Mono'; src: url('data:font/truetype;charset=utf-8;base64,{}') format('truetype'); font-weight: bold; font-style: normal; }}\n",
        b64_bold
    ));
    svg.push_str("    text { font-family: 'JetBrains Mono', 'Apple Color Emoji', 'Segoe UI Emoji', 'Noto Color Emoji', 'Segoe UI Symbol', 'SF Mono', SFMono-Regular, ui-monospace, Menlo, Monaco, monospace; text-rendering: geometricPrecision; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; }\n");
    svg.push_str("  </style>\n");
    svg.push_str("</defs>\n");

    // Transparent outer canvas
    svg.push_str(&format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"none\"/>\n",
        total_w, total_h
    ));

    // Terminal Container with Shadow & Rounded Corners
    svg.push_str(&format!(
        "<g filter=\"url(#macos-shadow)\">\n  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"12\" ry=\"12\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>\n</g>\n",
        pad, pad, term_w, term_h + header_h, container_bg, container_stroke
    ));

    // Sleek Title Bar Header with Rounded Top Corners (macOS Sequoia)
    svg.push_str(&format!(
        "<path d=\"M {} {} a 12 12 0 0 1 12 -12 h {} a 12 12 0 0 1 12 12 v {} h -{} z\" fill=\"{}\"/>\n",
        pad, pad + 12, term_w - 24, header_h - 12, term_w, header_bg
    ));
    svg.push_str(&format!(
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>\n",
        pad,
        pad + header_h,
        pad + term_w,
        pad + header_h,
        header_line
    ));

    // macOS Traffic Light Buttons (Apple macOS Sequoia: Red, Orange, Green)
    let btn_y = pad + 18;
    svg.push_str(&format!(
        "<circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"#ff5f57\" stroke=\"#e0443e\" stroke-width=\"0.5\"/>\n",
        pad + 20, btn_y
    ));
    svg.push_str(&format!(
        "<circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"#febc2e\" stroke=\"#d89e24\" stroke-width=\"0.5\"/>\n",
        pad + 40, btn_y
    ));
    svg.push_str(&format!(
        "<circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"#28c840\" stroke=\"#1aab29\" stroke-width=\"0.5\"/>\n",
        pad + 60, btn_y
    ));

    // Seamless Merged Terminal Screen Background Spans (Zero grid seams / block artifacts)
    let screen_x = pad;
    let screen_y = pad + header_h;
    let empty_bg = if is_light { "#ffffff" } else { "#181825" };

    for y in 0..rows {
        let mut x = 0;
        while x < cols {
            let cell = &buffer[(x as u16, y as u16)];
            let bg = cell.bg;
            let color_hex = color_to_hex(bg, true, is_light);
            if color_hex != empty_bg {
                let start_x = x;
                while x < cols
                    && color_to_hex(
                        buffer[(x as u16, y as u16)].bg,
                        true,
                        is_light,
                    ) == color_hex
                {
                    x += 1;
                }
                let span_len = x - start_x;
                svg.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
                    screen_x + start_x * cell_w,
                    screen_y + y * cell_h,
                    span_len * cell_w,
                    cell_h,
                    color_hex
                ));
            } else {
                x += 1;
            }
        }
    }

    // High-Resolution Smooth Text Rendering (Merged Text Runs)
    let default_fg = if is_light { "#24292f" } else { "#cdd6f4" };
    svg.push_str(&format!(
        "<g font-family=\"'JetBrains Mono', 'SF Mono', monospace\" font-size=\"13px\" fill=\"{}\">\n",
        default_fg
    ));
    for y in 0..rows {
        let mut x = 0;
        while x < cols {
            let cell = &buffer[(x as u16, y as u16)];
            let symbol = cell.symbol();
            if symbol == " " && cell.modifier.is_empty() {
                x += 1;
                continue;
            }
            if symbol.trim().is_empty() && symbol != " " {
                x += 1;
                continue;
            }

            let start_x = x;
            let fg = cell.fg;
            let is_bold =
                cell.modifier.contains(ratatui::style::Modifier::BOLD);
            let fg_hex = color_to_hex(fg, false, is_light);
            let mut text_buf = String::new();

            while x < cols {
                let cur_cell = &buffer[(x as u16, y as u16)];
                let cur_sym = cur_cell.symbol();
                let cur_fg_hex = color_to_hex(cur_cell.fg, false, is_light);
                let cur_bold =
                    cur_cell.modifier.contains(ratatui::style::Modifier::BOLD);

                if cur_fg_hex != fg_hex || cur_bold != is_bold {
                    break;
                }

                let mut esc = cur_sym
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;");
                if esc == " " {
                    esc = "&#160;".to_string();
                }
                text_buf.push_str(&esc);
                x += 1;
            }

            let fill_attr = if fg_hex == default_fg {
                String::new()
            } else {
                format!(" fill=\"{}\"", fg_hex)
            };

            let weight_attr = if is_bold {
                " font-weight=\"bold\""
            } else {
                ""
            };

            svg.push_str(&format!(
                "<text x=\"{}\" y=\"{}\"{}{}>{}</text>\n",
                screen_x + start_x * cell_w,
                screen_y + y * cell_h + 14,
                fill_attr,
                weight_attr,
                text_buf
            ));
        }
    }
    svg.push_str("</g>\n");
    svg.push_str("</svg>\n");

    let mut file = File::create(filename).unwrap();
    file.write_all(svg.as_bytes()).unwrap();
}

fn generate_pair_svg(buffer: &Buffer, base_name: &str) {
    generate_svg(
        buffer,
        &format!("docs/assets/screenshots/{}.svg", base_name),
        false,
    );
    generate_svg(
        buffer,
        &format!("docs/assets/screenshots/{}-light.svg", base_name),
        true,
    );
}

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"id":1,"username":"abletterer","name":"Arnaud Bletterer"}),
        ))
        .mount(&server)
        .await;
    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.active_panel = Panel::MainDetail;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "lazygitlab".into(),
        path_with_namespace: Some("abletterer/lazygitlab".into()),
        web_url: None,
        default_branch: Some("main".into()),
        namespace: None,
    });
    (server, app)
}

#[tokio::main]
async fn main() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(140, 36);
    let mut terminal = Terminal::new(backend).unwrap();

    std::fs::create_dir_all("docs/assets/screenshots").unwrap();

    // 1. Dashboard View (Todos, MRs, Issues)
    app.view_mode = ViewMode::List;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::Todos;
    app.todos = vec![
        Todo {
            id: 101,
            project: None,
            action_name: "approval_required".into(),
            target_type: "MergeRequest".into(),
            target_url: "".into(),
            body: "Merge request !124 requires your review & approval".into(),
            state: "pending".into(),
            created_at: "2026-08-30T07:00:00Z".into(),
            group: None,
        },
        Todo {
            id: 102,
            project: None,
            action_name: "mentioned".into(),
            target_type: "Issue".into(),
            target_url: "".into(),
            body: "mentioned you in issue #47: 'Check async diff rendering performance'".into(),
            state: "pending".into(),
            created_at: "2026-08-30T06:45:00Z".into(),
            group: None,
        },
    ];
    app.filtered_todos = vec![
        GroupedItem::Item(app.todos[0].clone()),
        GroupedItem::Item(app.todos[1].clone()),
    ];
    app.merge_requests = vec![
        MergeRequest {
            iid: 124,
            project_id: 1,
            title: "feat(diff): add syntax-highlighted side-by-side diff review".into(),
            description: Some("Implements syntect syntax highlighting in split diff panel.\n\nCloses #42.".into()),
            author: Author::new(1, "abletterer".into()),
            web_url: "https://gitlab.com/abletterer/lazygitlab/-/merge_requests/124".into(),
            state: "opened".into(),
            references: None,
            upvotes: Some(5),
            downvotes: Some(0),
            labels: vec![
                Label { id: 1, name: "feature".into(), color: "#89b4fa".into(), text_color: None },
                Label { id: 2, name: "review-ready".into(), color: "#a6e3a1".into(), text_color: None },
            ],
            milestone: Some(Milestone { id: 1, title: "v0.16.0".into() }),
            assignees: vec![Author::new(1, "abletterer".into())],
            reviewers: vec![Author::new(2, "reviewer1".into())],
            squash: Some(true),
            created_at: Some("2026-08-30T05:30:00Z".into()),
            draft: Some(false),
            diff_refs: None,
            reactions: vec![],
            merge_status: Some("can_be_merged".into()),
            detailed_merge_status: None,
            has_conflicts: Some(false),
            head_pipeline: Some(PipelineMin { id: 9812, status: "success".into() }),
            source_branch: "feat/diff-highlight".into(),
            target_branch: "master".into(),
            participants: vec![],
        },
        MergeRequest {
            iid: 123,
            project_id: 1,
            title: "fix(pipelines): handle cancelled pipeline status gracefully".into(),
            description: None,
            author: Author::new(2, "contributor".into()),
            web_url: "https://gitlab.com/abletterer/lazygitlab/-/merge_requests/123".into(),
            state: "opened".into(),
            references: None,
            upvotes: Some(2),
            downvotes: Some(0),
            labels: vec![Label { id: 3, name: "bug".into(), color: "#f38ba8".into(), text_color: None }],
            milestone: None,
            assignees: vec![],
            reviewers: vec![Author::new(1, "abletterer".into())],
            squash: None,
            created_at: Some("2026-08-29T14:00:00Z".into()),
            draft: Some(false),
            diff_refs: None,
            reactions: vec![],
            merge_status: Some("can_be_merged".into()),
            detailed_merge_status: None,
            has_conflicts: Some(false),
            head_pipeline: Some(PipelineMin { id: 9801, status: "running".into() }),
            source_branch: "fix/cancel-status".into(),
            target_branch: "master".into(),
            participants: vec![],
        },
    ];
    app.filtered_mrs = vec![
        GroupedItem::Item(app.merge_requests[0].clone()),
        GroupedItem::Item(app.merge_requests[1].clone()),
    ];
    app.selected_mr = Some(app.merge_requests[0].clone());
    app.mrs_state.select(Some(0));
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "dashboard");

    // 2. MR Diff review (with inline discussion, approvals)
    app.active_context = Context::Project;
    app.view_mode = ViewMode::Diff;
    app.approvals = Some(ApprovalInfo {
        approvals_required: Some(1),
        approvals_left: Some(0),
        approved_by: vec![ApprovalUser {
            user: Some(Author::new(2, "reviewer1".into())),
        }],
    });
    app.mr_diffs = vec![MRDiff {
        old_path: "src/app/mrs/review.rs".into(),
        new_path: "src/app/mrs/review.rs".into(),
        diff: "diff --git a/src/app/mrs/review.rs b/src/app/mrs/review.rs".into(),
        collapsed: false,
        display_lines: vec![
            DiffDisplayLine::Code("@@ -42,6 +42,12 @@ pub async fn post_comment(&mut self) -> Result<()> {".into(), None, None),
            DiffDisplayLine::Code("     let client = self.gitlab.clone();".into(), Some(42), Some(42)),
            DiffDisplayLine::Code("     let pid = self.selected_project.as_ref().unwrap().id;".into(), Some(43), Some(43)),
            DiffDisplayLine::Code("-    let note = self.textarea.lines().join(\"\\n\");".into(), Some(44), None),
            DiffDisplayLine::Code("+    let body = self.textarea.lines().join(\"\\n\");".into(), None, Some(44)),
            DiffDisplayLine::Code("+    let discussion = CreateMRDiscussion {".into(), None, Some(45)),
            DiffDisplayLine::Code("+        project: pid,".into(), None, Some(46)),
            DiffDisplayLine::Code("+        body: body.clone(),".into(), None, Some(47)),
            DiffDisplayLine::Code("+    };".into(), None, Some(48)),
            DiffDisplayLine::Code("     tokio::spawn(async move {".into(), Some(45), Some(49)),
            DiffDisplayLine::Code("         let ep = discussion;".into(), Some(46), Some(50)),
            DiffDisplayLine::Code("         let _ = ep.query_async(&client).await;".into(), Some(47), Some(51)),
            DiffDisplayLine::Code("     });".into(), Some(48), Some(52)),
            DiffDisplayLine::Code(" }".into(), Some(49), Some(53)),
        ],
    }];
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "mr_diff");

    // 3. Issue Detail & Management
    app.view_mode = ViewMode::List;
    app.active_selection = MenuSelection::Issues;
    app.issues = vec![Issue {
        iid: 47,
        project_id: 1,
        title: "Optimize tree loading for massive repositories with 10k+ files".into(),
        description: Some("Fetching full recursive repository tree on mono-repos causes initial delay.\n\n### Proposal\n- Implement lazy expansion for directory nodes\n- Cache visited directories\n- Add search filter debouncing".into()),
        author: Author::new(1, "abletterer".into()),
        assignees: vec![Author::new(1, "abletterer".into())],
        state: "opened".into(),
        web_url: "https://gitlab.com/abletterer/lazygitlab/-/issues/47".into(),
        references: None,
        issue_type: Some("issue".into()),
        labels: vec![Label { id: 1, name: "performance".into(), color: "#f38ba8".into(), text_color: None }, Label { id: 2, name: "high-priority".into(), color: "#fab387".into(), text_color: None }],
        milestone: Some(Milestone { id: 1, title: "v0.16.0".into() }),
        due_date: Some("2026-09-15".into()),
        start_date: Some("2026-08-25".into()),
        weight: Some(3),
        confidential: false,
        created_at: Some("2026-08-25T10:00:00Z".into()),
        upvotes: Some(8),
        downvotes: Some(0),
        task_completion_status: None,
        reactions: vec![],
        participants: vec![],
    }];
    app.filtered_issues = vec![GroupedItem::Item(app.issues[0].clone())];
    app.selected_issue = Some(app.issues[0].clone());
    app.issues_state.select(Some(0));
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "issue_detail");

    // 4. Kanban Board
    app.view_mode = ViewMode::Board;
    app.active_selection = MenuSelection::Boards;
    app.selected_board = Some(Board {
        id: 1,
        name: "Sprint Kanban".into(),
        lists: Some(vec![
            BoardList {
                id: 1,
                label: Some(Label { id: 1, name: "Backlog".into(), color: "#6c7086".into(), text_color: None }),
                position: Some(1),
                list_type: "label".into(),
            },
            BoardList {
                id: 2,
                label: Some(Label { id: 2, name: "In Progress".into(), color: "#89b4fa".into(), text_color: None }),
                position: Some(2),
                list_type: "label".into(),
            },
            BoardList {
                id: 3,
                label: Some(Label { id: 3, name: "Review".into(), color: "#f9e2af".into(), text_color: None }),
                position: Some(3),
                list_type: "label".into(),
            },
            BoardList {
                id: 4,
                label: Some(Label { id: 4, name: "Done".into(), color: "#a6e3a1".into(), text_color: None }),
                position: Some(4),
                list_type: "label".into(),
            },
        ]),
    });
    let mut board_issues = HashMap::new();
    board_issues.insert(1, vec![app.issues[0].clone()]);
    board_issues.insert(2, vec![Issue {
        iid: 48,
        project_id: 1,
        title: "Support custom themes via theme.toml".into(),
        description: None,
        author: Author::new(1, "abletterer".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "".into(),
        references: None,
        issue_type: None,
        labels: vec![Label { id: 3, name: "theming".into(), color: "#cba6f7".into(), text_color: None }],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: Some(1),
        confidential: false,
        created_at: None,
        upvotes: Some(4),
        downvotes: Some(0),
        task_completion_status: None,
        reactions: vec![],
        participants: vec![],
    }]);
    app.board_issues = board_issues;
    app.board_lists_state = Default::default();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "kanban");

    // 5. Job Log Viewer
    app.view_mode = ViewMode::JobLog;
    app.selected_job = Some(Job {
        id: 10842,
        name: "test:cargo-clippy".into(),
        stage: "test".into(),
        status: "success".into(),
        created_at: "2026-08-30T07:12:00Z".into(),
        started_at: Some("2026-08-30T07:12:05Z".into()),
        finished_at: Some("2026-08-30T07:12:35Z".into()),
        duration: Some(30.0),
        user: Some(Author::new(1, "abletterer".into())),
        runner: None,
        allow_failure: false,
        web_url: "https://gitlab.com/abletterer/lazygitlab/-/jobs/10842".into(),
        tag_list: vec!["docker".into(), "linux".into()],
    });
    app.job_log = "Running with gitlab-runner 17.2.0 on runner-z8q2...\n\u{1b}[32;1mFetching changes with git depth set to 20...\u{1b}[0;m\nReinitialized existing Git repository in /builds/abletterer/lazygitlab/.git/\nChecking out a1b2c3d as feat/diff-highlight...\n\u{1b}[34;1m$ cargo clippy --all-targets -- -D warnings\u{1b}[0;m\n   Compiling lazygitlab v0.16.0 (/builds/abletterer/lazygitlab)\n    Checking lazygitlab v0.16.0 (/builds/abletterer/lazygitlab)\n    Finished `dev` profile [unoptimized + debuginfo] in 4.82s\n\u{1b}[32;1mJob succeeded\u{1b}[0;m".into();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "job_log");

    // 6. CI/CD Pipelines
    app.view_mode = ViewMode::List;
    app.active_selection = MenuSelection::Pipelines;
    app.pipelines = vec![
        Pipeline { id: 9812, project_id: 1, status: "success".into(), source: "push".into(), ref_name: "main".into(), web_url: "".into(), created_at: "2026-08-30T07:10:00Z".into(), started_at: None, duration: Some(125.0) },
        Pipeline { id: 9811, project_id: 1, status: "running".into(), source: "push".into(), ref_name: "feat/diff-highlight".into(), web_url: "".into(), created_at: "2026-08-30T07:05:00Z".into(), started_at: None, duration: Some(45.0) },
        Pipeline { id: 9810, project_id: 1, status: "failed".into(), source: "merge_request_event".into(), ref_name: "fix/parser-edge-case".into(), web_url: "".into(), created_at: "2026-08-30T06:30:00Z".into(), started_at: None, duration: Some(30.0) },
    ];
    app.filtered_pipelines = app.pipelines.clone();
    app.selected_pipeline = Some(app.pipelines[0].clone());
    app.pipelines_state.select(Some(0));
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "pipelines");

    // 7. Repository Tree & File Viewer
    app.active_selection = MenuSelection::Repository;
    app.repository_tree = vec![
        RepositoryTreeItem { id: "1".into(), name: "src".into(), r#type: "tree".into(), path: "src".into(), mode: "040000".into() },
        RepositoryTreeItem { id: "2".into(), name: "api.rs".into(), r#type: "blob".into(), path: "src/api.rs".into(), mode: "100644".into() },
        RepositoryTreeItem { id: "3".into(), name: "main.rs".into(), r#type: "blob".into(), path: "src/main.rs".into(), mode: "100644".into() },
        RepositoryTreeItem { id: "4".into(), name: "models.rs".into(), r#type: "blob".into(), path: "src/models.rs".into(), mode: "100644".into() },
        RepositoryTreeItem { id: "5".into(), name: "Cargo.toml".into(), r#type: "blob".into(), path: "Cargo.toml".into(), mode: "100644".into() },
        RepositoryTreeItem { id: "6".into(), name: "README.md".into(), r#type: "blob".into(), path: "README.md".into(), mode: "100644".into() },
    ];
    use base64::Engine;
    let sample_code = "use crossterm::event::{self, Event};\nuse gitlab::GitlabBuilder;\nuse lazygitlab::*;\n\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let config = Config::from_env();\n    let client = GitlabBuilder::new(config.url, config.token).build_async().await?;\n    run_app(client).await\n}";
    app.selected_file_content = Some(RepositoryFile {
        file_name: "main.rs".into(),
        file_path: "src/main.rs".into(),
        size: sample_code.len() as u64,
        encoding: "base64".into(),
        content_sha256: "abc".into(),
        ref_name: "main".into(),
        blob_id: "def".into(),
        commit_id: "123".into(),
        last_commit_id: "123".into(),
        content: base64::engine::general_purpose::STANDARD.encode(sample_code),
    });
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "repo_tree");

    // 8. Help & Command Palette
    app.view_mode = ViewMode::List;
    app.show_help_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "help_palette");
    app.show_help_popup = false;

    // 9. Project Switcher Popup
    app.show_project_popup = true;
    app.projects = vec![
        Project { id: 1, name: "lazygitlab".into(), path_with_namespace: Some("abletterer/lazygitlab".into()), web_url: None, default_branch: Some("main".into()), namespace: None },
        Project { id: 2, name: "infrastructure".into(), path_with_namespace: Some("devops/infrastructure".into()), web_url: None, default_branch: Some("master".into()), namespace: None },
        Project { id: 3, name: "web-frontend".into(), path_with_namespace: Some("frontend/web-app".into()), web_url: None, default_branch: Some("main".into()), namespace: None },
    ];
    app.filtered_projects = app.projects.clone();
    app.projects_state.select(Some(0));
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    generate_pair_svg(terminal.backend().buffer(), "project_switch");
    app.show_project_popup = false;

    // 10. Generate Automated Cheatsheet Markdown
    generate_cheatsheet_file(&mut app);
    println!("Vector screenshots and cheatsheet markdown generated successfully.");
}

fn format_shortcut_kbd(shortcut: &str) -> String {
    let parts: Vec<&str> = shortcut.split_whitespace().collect();
    if parts.len() > 1 {
        parts
            .into_iter()
            .map(|p| {
                if p == "/" || p == "+" || p == "-" {
                    format!(" {} ", p)
                } else {
                    format!("<kbd>{}</kbd>", p)
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    } else {
        format!("<kbd>{}</kbd>", shortcut)
    }
}

fn generate_cheatsheet_file(app: &mut App) {
    let mut categories: HashMap<String, Vec<(String, String)>> = HashMap::new();

    let states = [
        (ViewMode::List, MenuSelection::MergeRequests),
        (ViewMode::List, MenuSelection::Issues),
        (ViewMode::List, MenuSelection::Pipelines),
        (ViewMode::List, MenuSelection::Boards),
        (ViewMode::List, MenuSelection::Repository),
        (ViewMode::List, MenuSelection::Todos),
        (ViewMode::Diff, MenuSelection::MergeRequests),
        (ViewMode::JobLog, MenuSelection::Pipelines),
        (ViewMode::Board, MenuSelection::Boards),
        (ViewMode::Tree, MenuSelection::Repository),
    ];

    for (view_mode, selection) in states {
        app.view_mode = view_mode;
        app.active_selection = selection;
        for item in app.get_available_actions() {
            let cat = item.category.clone();
            let entry = (item.shortcut.clone(), item.description.clone());
            let list = categories.entry(cat).or_default();
            if !list.contains(&entry) {
                list.push(entry);
            }
        }
    }

    let mut doc = String::new();
    doc.push_str("# lazygitlab Keyboard Cheatsheet\n\n");
    doc.push_str("> **Note:** This cheatsheet is **automatically generated from the application's keymap definitions** to ensure it remains 100% synchronized with the latest release.\n\n");
    doc.push_str("[📥 Download Printable 1-Page Landscape PDF Cheatsheet](../assets/lazygitlab-cheatsheet.pdf){ .md-button .md-button--primary }\n\n");
    doc.push_str("### 📄 1-Page Quick Reference Preview\n\n");
    doc.push_str("Click on the preview to open the high-resolution lightbox or download the vector PDF above:\n\n");
    doc.push_str("![lazygitlab Cheatsheet Preview](../assets/lazygitlab-cheatsheet-preview.png)\n\n");
    doc.push_str("---\n\n");

    doc.push_str("## ⚡ Essential Shortcuts (Top 10)\n\n");
    doc.push_str("| Shortcut | Action | Description |\n");
    doc.push_str("| :--- | :--- | :--- |\n");
    doc.push_str("| <kbd>?</kbd> | **Command Palette** | Open interactive help and keymap finder |\n");
    doc.push_str("| <kbd>Tab</kbd> / <kbd>Shift+Tab</kbd> | **Cycle Panels** | Switch focus between side menu, list, and detail view |\n");
    doc.push_str("| <kbd>j</kbd> / <kbd>k</kbd> | **Navigate Items** | Move selection down / up |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>m</kbd> | **Merge Requests** | Jump directly to Merge Requests view |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>i</kbd> | **Issues** | Jump directly to Issues view |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>b</kbd> | **Kanban Boards** | Jump directly to Sprint Kanban Boards |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>p</kbd> | **Pipelines** | Jump directly to CI/CD Pipelines |\n");
    doc.push_str("| <kbd>p</kbd> | **Project Switcher** | Filter and quick-switch between GitLab projects |\n");
    doc.push_str("| <kbd>/</kbd> | **Filter Search** | Search and filter active list items in real-time |\n");
    doc.push_str("| <kbd>q</kbd> | **Quit** | Exit lazygitlab |\n\n");
    doc.push_str("---\n\n");

    let ordered_cats = [
        ("Global", "🌐 Global & Navigation"),
        ("Merge Request", "🔀 Merge Requests & Review"),
        ("Diff", "📝 Diff Review & Comments"),
        ("Issue", "📋 Issues & Triage"),
        ("Pipeline", "🚀 CI/CD Pipelines"),
        ("Job Log", "📜 Live Job Log Viewer"),
        ("Repository", "🌲 Repository & Files"),
        ("Filter", "🔍 Filters & Search"),
        ("Discussion", "💬 Discussions & Notes"),
    ];

    for (cat_key, cat_title) in ordered_cats {
        if let Some(items) = categories.get(cat_key) {
            doc.push_str(&format!("## {}\n\n", cat_title));
            doc.push_str("| Shortcut | Description |\n");
            doc.push_str("| :--- | :--- |\n");
            for (shortcut, desc) in items {
                doc.push_str(&format!("| {} | {} |\n", format_shortcut_kbd(shortcut), desc));
            }
            doc.push('\n');
        }
    }

    doc.push_str("---\n\n");
    doc.push_str("## 🧭 Quick Jump (`g` prefix)\n\n");
    doc.push_str("| Key Combination | Target View |\n");
    doc.push_str("| :--- | :--- |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>t</kbd> | Personal To-Dos |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>m</kbd> | Merge Requests |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>i</kbd> | Issues |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>b</kbd> | Kanban Boards |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>p</kbd> | CI/CD Pipelines |\n");
    doc.push_str("| <kbd>g</kbd> <kbd>r</kbd> | Repository Tree |\n");

    let mut file = File::create("docs/reference/cheatsheet.md").unwrap();
    file.write_all(doc.as_bytes()).unwrap();
}
