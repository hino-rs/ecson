use ecson::prelude::*;

#[ecson::resource]
#[derive(Debug)]
struct Count(u32);

fn build_app() -> EcsonApp {
    let mut app = EcsonApp::new();
    app.insert_resource(Count(0));
    app.add_systems(Startup, increment);
    app.add_systems(Update, increment);

    app.add_systems(Update, end);
    app.add_systems(Shutdown, reset);
    app
}

fn increment(mut count: ResMut<Count>) {
    count.0 += 1;
}

fn end(count: Res<Count>, flag: Res<ShutdownFlag>) {
    if count.0 == 100 {
        EcsonApp::request_shutdown(flag);
    }
}

fn reset(mut count: ResMut<Count>) {
    count.0 = 0;
}

fn result(app: EcsonApp) -> u32 {
    let count = app.world().get_resource::<Count>().unwrap();
    count.0
}

#[test]
fn startup_test() {
    let mut app = build_app();
    app.startup();

    assert_eq!(result(app), 1);
}

#[test]
fn tick_once_test() {
    let mut app = build_app();
    app.tick_once();

    assert_eq!(result(app), 1);
}

#[test]
fn tick_n_test() {
    let n = 1000;

    let mut app = build_app();
    app.tick_n(n);

    assert_eq!(result(app), 1000);
}

#[test]
fn run_shutdown_test() {
    let mut app = build_app();
    app.run();

    assert_eq!(result(app), 0);
}
