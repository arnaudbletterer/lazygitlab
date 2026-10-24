use lazygitlab::ui::Theme;
use std::path::PathBuf;

#[test]
fn theme_is_never_loaded_from_the_working_directory() {
    let paths = Theme::candidate_paths(Some("/home/u"), Some("/appdata"));
    assert!(paths.iter().all(|p| p.is_absolute()), "{paths:?}");
    assert!(paths.contains(&PathBuf::from("/home/u/.config/lazygitlab/theme.toml")));
    assert!(Theme::candidate_paths(None, None).is_empty());
}
