// Exampleではないです
// 適当実験したいときに使います。

use ecson::prelude::*;

#[derive(Resource, Debug)]
struct Count(u128);

fn x(mut count: ResMut<Count>) {
    count.0 += 1;
    println!("{count:?}");
}

fn main() {
    EcsonApp::new()
        .insert_resource(Count(0))
        .add_systems(Update, x)
        .tick_n(100);
}
