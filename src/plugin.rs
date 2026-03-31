//! プラグインシステムの基盤

use crate::app::*;

/// プラグインのライフサイクル状態を表す列挙型。
/// （※将来的なステート管理のためのプレースホルダーです）
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum PluginsState {
    Adding,
    Ready,
    Finished,
    Cleaned,
}

// ============================================================================
// プラグインシステム基盤 (Traits & Macros)
// ============================================================================

/// 個別のプラグインが実装する基本トレイト。
pub trait Plugin {
    /// プラグインがアプリケーションに対してシステムやリソースを登録する処理を記述します。
    fn build(self, app: &mut EcsonApp);
}

/// `app.add_plugins()` に単一の `Plugin` や、複数の `Plugin` をまとめたタプルを渡せるようにするトレイト。
pub trait Plugins {
    /// 自身に含まれるプラグインをアプリケーションに追加します。
    fn add_to_app(self, app: &mut EcsonApp);
}

// 単一の Plugin を Plugins として扱えるようにするための実装
impl<P: Plugin> Plugins for P {
    fn add_to_app(self, app: &mut EcsonApp) {
        self.build(app);
    }
}

// タプルに対する Plugins の一括実装用マクロ
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

// 最大6つのプラグインをタプルとして渡せるように実装を展開
impl_plugins_for_tuples!(P1);
impl_plugins_for_tuples!(P1, P2);
impl_plugins_for_tuples!(P1, P2, P3);
impl_plugins_for_tuples!(P1, P2, P3, P4);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5);
impl_plugins_for_tuples!(P1, P2, P3, P4, P5, P6);

