use ecson::prelude::*;

#[derive(Resource, Debug)]
struct Count(u128);

fn test(mut count: ResMut<Count>) {
    println!("{count:?}");
    count.0 += 1;
}

fn main() {
    EcsonApp::new()
        .insert_resource(Count(0))
        .insert_resource(ServerTimeConfig {
            tick_rate: 1.0,
            ..Default::default()
        })
        .add_systems(Update, test)
        .run();
}
