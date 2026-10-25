use lazygitlab::headless::parse_script;

#[test]
fn test_parse_script_basic_keys() {
    let events = parse_script("q\nj\nk\n<Enter>\n<Esc>\n").unwrap();
    assert_eq!(events.len(), 5);

    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    let expected = [
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        },
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        },
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        },
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        },
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        },
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

#[tokio::test]
async fn test_scripted_event_source_fifo_order() {
    use crossterm::event::KeyCode;
    use lazygitlab::EventSource;
    use lazygitlab::headless::ScriptedEventSource;
    let src = ScriptedEventSource::from_text("a\nb\nc").unwrap();
    let mut got = Vec::new();
    for _ in 0..3 {
        if let Some(crossterm::event::Event::Key(k)) = src
            .next_event(std::time::Duration::from_millis(10))
            .await
            .unwrap()
            && let KeyCode::Char(ch) = k.code
        {
            got.push(ch);
        }
    }
    assert_eq!(
        got,
        vec!['a', 'b', 'c'],
        "ScriptedEventSource must deliver events in script order (FIFO), not LIFO"
    );
}

#[test]
fn test_script_items_directives() {
    use lazygitlab::headless::{Condition, ScriptItem, parse_script_items};
    use std::time::Duration;
    let items = parse_script_items(
        "g\nwait:250\ntimeout:5000\nuntil:Merge\nuntil_key:G:@mrs>=500\nmark:x\n",
    )
    .unwrap();
    assert_eq!(items.len(), 5);
    assert_eq!(items[4], ScriptItem::Mark("x".into()));
    assert_eq!(items[1], ScriptItem::Wait(Duration::from_millis(250)));
    match &items[2] {
        ScriptItem::Until {
            condition,
            press,
            timeout,
        } => {
            assert_eq!(*condition, Condition::Text("Merge".into()));
            assert!(press.is_none());
            assert_eq!(*timeout, Duration::from_millis(5000));
        }
        other => panic!("unexpected {other:?}"),
    }
    match &items[3] {
        ScriptItem::Until {
            condition, press, ..
        } => {
            assert_eq!(
                *condition,
                Condition::AtLeast {
                    list: "mrs".into(),
                    count: 500
                }
            );
            assert!(press.is_some());
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn test_script_directive_errors() {
    use lazygitlab::headless::parse_script_items;
    assert!(parse_script_items("wait:abc").is_err());
    assert!(parse_script_items("until:@bogus>=3").is_err());
    assert!(parse_script_items("until:").is_err());
    assert!(parse_script_items("until_key:G").is_err());
    assert!(lazygitlab::headless::parse_script("wait:10").is_err());
}

#[tokio::test]
async fn test_wait_delays_following_events() {
    use lazygitlab::EventSource;
    use lazygitlab::headless::ScriptedEventSource;
    use std::time::{Duration, Instant};
    let src = ScriptedEventSource::from_text("a\nwait:120\nb").unwrap();
    assert!(
        src.next_event(Duration::from_millis(50))
            .await
            .unwrap()
            .is_some()
    );
    let start = Instant::now();
    let mut got = None;
    while got.is_none() {
        got = src.next_event(Duration::from_millis(50)).await.unwrap();
    }
    assert!(start.elapsed() >= Duration::from_millis(100));
}

#[test]
fn test_until_key_repeat_must_be_positive() {
    use lazygitlab::headless::parse_script_items;
    let err = parse_script_items("until_key:j*0:@mrs>=5").unwrap_err();
    assert!(err.contains("at least 1"), "{err}");
    assert!(parse_script_items("until_key:j*-1:@mrs>=5").is_err());
    assert!(parse_script_items("until_key:j*1:@mrs>=5").is_ok());
}

#[test]
fn test_until_text_prefix_matches_literal_at_sign() {
    use lazygitlab::headless::{Condition, ScriptItem, parse_script_items};
    let items = parse_script_items("until:text:@ada").unwrap();
    match &items[0] {
        ScriptItem::Until { condition, .. } => {
            assert_eq!(*condition, Condition::Text("@ada".into()));
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(parse_script_items("until:text:").is_err());
}
