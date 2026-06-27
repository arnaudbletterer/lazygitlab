use lazygitlab::headless::{parse_script, parse_options};

#[test]
fn test_parse_script_basic_keys() {
    let events = parse_script("q\nj\nk\n<Enter>\n<Esc>\n").unwrap();
    assert_eq!(events.len(), 5);

    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    let expected = [
        KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE },
        KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE },
        KeyEvent { code: KeyCode::Char('k'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE },
        KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE },
        KeyEvent { code: KeyCode::Esc, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE },
    ];
    for (i, exp) in expected.iter().enumerate() {
        match &events[i] {
            Event::Key(k) => assert_eq!(k.code, exp.code, "event {i} code"),
            _ => panic!("event {i} is not a Key"),
        }
    }
}

#[test]
fn test_parse_script_modifiers() {
    use crossterm::event::KeyModifiers;
    let events = parse_script("<S-q>\n<C-r>\n<A-x>\n").unwrap();
    assert_eq!(events.len(), 3);
    use crossterm::event::{Event, KeyCode};
    if let Event::Key(k) = &events[0] {
        assert_eq!(k.code, KeyCode::Char('q'));
        assert!(k.modifiers.contains(KeyModifiers::SHIFT));
    } else {
        panic!("not a key event");
    }
    if let Event::Key(k) = &events[1] {
        assert_eq!(k.code, KeyCode::Char('r'));
        assert!(k.modifiers.contains(KeyModifiers::CONTROL));
    }
    if let Event::Key(k) = &events[2] {
        assert_eq!(k.code, KeyCode::Char('x'));
        assert!(k.modifiers.contains(KeyModifiers::ALT));
    }
}

#[test]
fn test_parse_script_function_keys() {
    use crossterm::event::{Event, KeyCode};
    let events = parse_script("<F1>\n<F12>\nup\ndown\n").unwrap();
    assert_eq!(events.len(), 4);
    if let Event::Key(k) = &events[0] {
        assert_eq!(k.code, KeyCode::F(1));
    }
    if let Event::Key(k) = &events[1] {
        assert_eq!(k.code, KeyCode::F(12));
    }
    if let Event::Key(k) = &events[2] {
        assert_eq!(k.code, KeyCode::Up);
    }
    if let Event::Key(k) = &events[3] {
        assert_eq!(k.code, KeyCode::Down);
    }
}

#[test]
fn test_parse_script_mouse() {
    use crossterm::event::{MouseButton, MouseEventKind};
    let events = parse_script("mouse:down:10:5\nmouse:scroll_down:0:0\n").unwrap();
    assert_eq!(events.len(), 2);
    if let crossterm::event::Event::Mouse(m) = &events[0] {
        assert_eq!(m.column, 10);
        assert_eq!(m.row, 5);
        assert!(matches!(m.kind, MouseEventKind::Down(MouseButton::Left)));
    }
}

#[test]
fn test_parse_script_comments_and_blank_lines() {
    let events = parse_script("# a comment\n\n   \nq\n# another\n").unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_parse_options_default() {
    let opts = parse_options(&[]).unwrap();
    assert_eq!(opts.width, 120);
    assert_eq!(opts.height, 40);
    assert!(opts.script_path.is_none());
    assert!(opts.frames_dir.is_none());
}

#[test]
fn test_parse_options_with_flags() {
    let args: Vec<String> = ["--width", "200", "--height", "50", "--script", "foo.txt", "--frames-dir", "out"]
        .iter().map(|s| s.to_string()).collect();
    let opts = parse_options(&args).unwrap();
    assert_eq!(opts.width, 200);
    assert_eq!(opts.height, 50);
    assert_eq!(opts.script_path.as_deref().unwrap().to_str().unwrap(), "foo.txt");
    assert_eq!(opts.frames_dir.as_deref().unwrap().to_str().unwrap(), "out");
}

#[test]
fn test_parse_options_unknown() {
    let args = vec!["--unknown".to_string()];
    assert!(parse_options(&args).is_err());
}
