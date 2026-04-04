//! プラグインシステムの基盤

use crate::app::*;

/// プラグインのライフサイクル状態を表す列挙型。
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum PluginsState {
    Adding,
    Ready,
    Finished,
    Cleaned,
}

/// 個別のプラグインが実装する基本トレイト。
pub trait Plugin {
    fn build(self, app: &mut EcsonApp);
}

/// `app.add_plugins()` に単一の `Plugin` や複数の `Plugin` タプルを渡せるようにするトレイト。
pub trait Plugins {
    fn add_to_app(self, app: &mut EcsonApp);
}

impl<P: Plugin> Plugins for P {
    fn add_to_app(self, app: &mut EcsonApp) {
        self.build(app);
    }
}

macro_rules! impl_plugins_for_tuples {
    ($($name:ident),*) => {
        impl<$($name: Plugin),*> Plugins for ($($name,)*) {
            #[allow(non_snake_case)]
            fn add_to_app(self, app: &mut EcsonApp) {
                let ($($name,)*) = self;
                $($name.build(app);)*
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
