//! Foundation of the plugin system.

use bevy_ecs::world::World;

use crate::app::*;

/// Enum representing the lifecycle state of plugins.
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum PluginsState {
    Adding,
    Ready,
    Finished,
    Cleaned,
}

/// The base trait that individual plugins implement.
pub trait Plugin {
    fn build(&mut self, app: &mut EcsonApp);

    /// Cleanup logic called on shutdown.
    fn cleanup(&self, _app: &mut World) {}
}

/// Trait that allows passing a single `Plugin` or a tuple of `Plugin`s to `app.add_plugins()`.
pub trait Plugins {
    fn add_to_app(self, app: &mut EcsonApp);
}

impl<P: Plugin + 'static> Plugins for P {
    fn add_to_app(mut self, app: &mut EcsonApp) {
        self.build(app);
        app.plugins.push(Box::new(self));
    }
}

macro_rules! impl_plugins_for_tuples {
    ($($name:ident),*) => {
        impl<$($name: Plugin + 'static),*> Plugins for ($($name,)*) {
            #[allow(non_snake_case)]
            fn add_to_app(self, app: &mut EcsonApp) {
                let ($(mut $name,)*) = self;
                $(
                    $name.build(app);
                    app.plugins.push(Box::new($name));
                )*
            }
        }
    };
}

impl_plugins_for_tuples!(P1);
impl_plugins_for_tuples!(P1, P2);
impl_plugins_for_tuples!(P1, P2, P3);
impl_plugins_for_tuples!(P1, P2, P3, P4);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6, P7);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6, P7, P8);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6, P7, P8, P9);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
