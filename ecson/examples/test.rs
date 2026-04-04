use ecson::prelude::*;

#[derive(Resource, Debug)]
struct Count(u128);

fn test(mut count: ResMut<Count>) {
    // if count.0.is_multiple_of(1000) {
    //     println!("{count:?}");
    // }

    println!("{count:?}");
    count.0 += 1;
}

fn main() {
    EcsonApp::new()
        .insert_resource(Count(0))
        .insert_resource(ServerTimeConfig {
            update_sleep: 0.01,
            tick_rate: 1.0,
            ..Default::default()
        })
        .add_systems(Update, test)
        .run();
}
