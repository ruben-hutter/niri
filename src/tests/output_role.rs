use smithay::output::Output;

use super::client::ClientId;
use super::*;

fn find_output(f: &Fixture, name: &str) -> Output {
    f.state
        .server
        .state
        .niri
        .global_space
        .outputs()
        .find(|o| o.name() == name)
        .unwrap()
        .clone()
}

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

#[test]
fn role_anchored_workspaces_survive_home_to_work_commute() {
    // The field trace, end to end through Niri: workspaces on the home role-2 monitor camp
    // on the laptop, then the work dock connects role 2 first and role 1 a moment later.
    // The workspaces must end up on the work role-2 monitor, not on the new primary.
    let mut config = niri_config::Config::default();
    // Work setup.
    config.outputs.0.push(niri_config::Output {
        name: "headless-1".to_string(),
        role: Some(1),
        ..Default::default()
    });
    config.outputs.0.push(niri_config::Output {
        name: "headless-2".to_string(),
        role: Some(2),
        ..Default::default()
    });
    // Home setup.
    config.outputs.0.push(niri_config::Output {
        name: "headless-3".to_string(),
        role: Some(1),
        ..Default::default()
    });
    config.outputs.0.push(niri_config::Output {
        name: "headless-4".to_string(),
        role: Some(2),
        ..Default::default()
    });
    // Laptop panel.
    config.outputs.0.push(niri_config::Output {
        name: "headless-5".to_string(),
        role: Some(3),
        ..Default::default()
    });

    let mut f = Fixture::with_config(config);

    // Boot at home: home main (role 1), home side (role 2), laptop (role 3).
    f.add_output(3, (1920, 1080));
    f.add_output(4, (1920, 1080));
    f.add_output(5, (1280, 720));

    // Work in two workspaces on the home side monitor.
    let home_side = find_output(&f, "headless-4");
    f.niri().layout.focus_output(&home_side);
    let id = f.add_client();
    let add_window = |f: &mut Fixture, id: ClientId| {
        let window = f.client(id).create_window();
        let surface = window.surface.clone();
        window.commit();
        f.roundtrip(id);

        let window = f.client(id).window(&surface);
        window.attach_new_buffer();
        window.set_size(100, 100);
        window.ack_last_and_commit();
        f.double_roundtrip(id);
    };
    add_window(&mut f, id);
    f.niri().layout.switch_workspace_down();
    add_window(&mut f, id);

    let windows_on = |f: &Fixture| -> Vec<String> {
        let niri = &f.state.server.state.niri;
        niri.layout
            .workspaces()
            .filter(|(_, _, ws)| ws.has_windows())
            .map(|(mon, _, _)| mon.unwrap().output_name().to_string())
            .collect()
    };

    assert_eq!(windows_on(&f), ["headless-4", "headless-4"]);

    // Undock at home: both workspaces camp on the laptop.
    let home_main = find_output(&f, "headless-3");
    f.niri().remove_output(&home_main);
    f.niri().remove_output(&home_side);
    assert_eq!(windows_on(&f), ["headless-5", "headless-5"]);

    // Dock at work: role 2 connects first, then role 1 takes over as primary.
    f.add_output(2, (1920, 1080));
    f.add_output(1, (1920, 1080));
    f.roundtrip(id);

    assert_eq!(
        windows_on(&f),
        ["headless-2", "headless-2"],
        "role-2 workspaces must land on the role-2 monitor, not on the new primary"
    );
}
