use lazygitlab::sanitize::{strip_all_escapes, strip_control_sequences};

#[test]
fn plain_text_is_untouched() {
    assert_eq!(
        strip_control_sequences("hello\tworld\r\n"),
        "hello\tworld\r\n"
    );
}

#[test]
fn sgr_colors_are_preserved() {
    let s = "\u{1b}[1;31merror\u{1b}[0m done";
    assert_eq!(strip_control_sequences(s), s);
}

#[test]
fn osc_title_and_clipboard_are_removed() {
    assert_eq!(strip_control_sequences("a\u{1b}]0;pwned\u{7}b"), "ab");
    assert_eq!(strip_control_sequences("a\u{1b}]52;c;AAAA\u{1b}\\b"), "ab");
}

#[test]
fn cursor_movement_and_screen_clears_are_removed() {
    assert_eq!(
        strip_control_sequences("a\u{1b}[2Jb\u{1b}[10;5Hc\u{1b}[?1049hd"),
        "abcd"
    );
}

#[test]
fn bell_backspace_and_c1_controls_are_removed() {
    assert_eq!(
        strip_control_sequences("a\u{7}\u{8}b\u{9b}31mc\u{9d}x"),
        "ab31mcx"
    );
}

#[test]
fn dangling_escape_is_dropped() {
    assert_eq!(strip_control_sequences("abc\u{1b}"), "abc");
    assert_eq!(strip_control_sequences("abc\u{1b}[31"), "abc");
}

#[test]
fn strip_all_drops_sgr_too() {
    assert_eq!(strip_all_escapes("\u{1b}[31mred\u{1b}[0m"), "red");
}
