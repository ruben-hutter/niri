use super::*;

#[test]
fn focus_default_monitor_prefers_lowest_role() {
    let mut config = niri_config::Config::default();
    config.outputs.0.push(niri_config::Output {
        name: "headless-2".to_string(),
        role: Some(1),
        ..Default::default()
    });

    let mut f = Fixture::with_config(config);
    f.add_output(1, (1920, 1080));
    f.add_output(2, (1280, 720));

    f.niri_state().focus_default_monitor();

    let active = f.niri().layout.active_output().unwrap();
    assert_eq!(active.name(), "headless-2");
}
