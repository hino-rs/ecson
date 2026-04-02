# ECS解説

ECSは「Entity Component System」の略称で、アーキテクチャパターンの1つです。
よくデータ志向設計(DOD)と混同されがちですが、根本的に別レイヤーの概念です。
このサイトでは、ECS駆動であるEcsonを使った開発が円滑になるように、ECSについて解説します。

どうやって解説しようかなーと悩んだのですが、OOP, DOD, ECSの順番で説明してみます。

## OOP

言わずと知れたオブジェクト指向ですが、これはDODの対称ともいえるので、まずはOOPを改めて理解しましょう。
オブジェクトは、**「データ」と「処理」をセットにしたモノ**です。
オブジェクトを中心とすることで、人間にとって直感的に扱うことができますが、CPUにとってはそうでありません。

例えば、

```Rust
#[derive(Default)]
struct User<'a> {
    id: u64,
    name: &'a str,
    bio: &'a str,
    age: u8,
    friends: Box<Vec<User<'a>>>,
}

impl<'a> User<'a> {
    fn edit_bio(&mut self, new_bio: &'a str) {
        self.bio = new_bio;
    }
}
```

このようなオブジェクトがあったとしましょう。
1000000のインスタンスを生成し、ベクタに追加したとして、

```Rust
let mut users: Vec<User> = vec![];

//
// usersにUserを1000000追加する処理
//

for id in 1..users.len() {
    if users[id].id % 2 == 0 {
        users[id].edit_bio("I like the Rust language");
    }
}
```

このような処理をしましょう(意味不明ですが)。
このとき、本質的には`User`の`bio`を書き換えたいだけなのに、毎回その他5つのフィールドを読み込んでしまっています。

|フィールド|型|サイズ(バイト)|必要|
|---|---|---|---|
|`id`|`u64`|8|✕|
|`name`|`&str`|16|✕|
|`bio`|`&str`|16|〇|
|`age`|`u8`|1|✕|
|`friends`|`Box`|8|✕|

計49バイトなんですが、この場合8の倍数(56)に揃えるために、コンパイラがパディングを7バイト分入れてしまいます。
その方が気持ちが良いからだと思ってください。

さあ、もう言いたいことは分かりますよね。
必要なのは8バイトだけなので、14.2%(8/56)しか意味がないんです。言い換えれば、85.8%ほど無駄になっています。

サイズを強調しましたが、ここで重要なのは、サイズではなく`bio`と`bio`の間にデータがあることなんです。
CPUが毎回`bio`の場所を見つけて処理しないといけない、いわゆるキャッシュに乗りづらくなってしまう(ヒット率が低い)のです。

## DOD

そういった問題を解決するのがDODなんです。
OOPが「人間ファースト」であれば、DODは「CPUファースト」ともいえるでしょう。

もしかしたら、今のあなたはこの構造体を見れば一瞬で理解できるかもしれません。

```Rust
#[derive(Default)]
struct UsersData<'a> {
    ids: Vec<u64>,
    names: Vec<&'a str>,
    bios: Vec<&'a str>,
    ages: Vec<u8>,
    friend_ids: Vec<Vec<usize>>,
}
```

そうです、データの持ち方を「構造体の配列」から「配列の構造体」へとひっくり返したのです。
前者をAoS(Array of Structures)、後者をSoA(Structure of Arrays)と呼びます。

さて、以下はAoSとSoAどっちでしょうか？
```
[u64, u64, u64, u64, u64, u64 ...]
[&str, &str, &str, &str, &str, ...]
[&str, &str, &str, &str, &str, ...]
[u8, u8, u8, u8, u8, u8, u8, u8 ...]
[Vec<usize>, Vec<usize>, Vec<usize>, ...]
```

続いて、以下はどうでしょう
```
[{u64, &str, &str, u8, Vec<usize>}, {u64, &str, &str, u8, Vec<usize>}, {u64, &str, &str, u8, Vec<usize>}]
```

言うまでもないですが、上がAoS(OOP)、下がSoA(DOD)です。

bioを編集する操作をしてみましょう。

```Rust
for i in 0..users.ids.len() {
    if users.ids[i] % 2 == 0 {
        users.bios[i] = "I like the Rust language";
    }
}
```

はい、1対象あたり&str(8バイト)のみを扱う形にすることができました。
パディングも発生せずに無駄0ですし、
CPUは常に前回の1つ隣にあるデータにアクセスすれば良いので、キャッシュに超絶乗りやすい(ヒット率が高い)です。

## ECS

さて、DODの本質を理解したところで、ECSについてみてみましょうか。

導入でも言った通り、ECSは3つの要素で構成され、かつ完全にそれぞれ分離しています。

- Entity (エンティティ)
  - 単なる「空の箱」または「ID(識別番号)」です。
- Component (コンポーネント)
  - 「純粋なデータのみ」の集まりです。
- System (システム)
  - 「純粋なロジックのみ」を担当します。

先ほどのプログラムをECS的な実装にしてみましょう。

```Rust
#[derive(Default)]
struct World<'a> {
    // コンポーネント群
    ids: Vec<u64>,
    names: Vec<&'a str>,
    bios: Vec<&'a str>,
    ages: Vec<u8>,
    friends: Vec<Vec<usize>>, 
}

// システム
fn edit_bio_system(ids: &[u64], bios: &mut [&str]) {
    for (id, bio) in ids.iter().zip(bios.iter_mut()) {
        if id % 2 == 0 {
            *bio = "I like the Rust language";
        }
    }
}

fn main() {
    let mut world = World::default();

    // エンティティを1000000作る
    for id in 1..=1000000 {
        world.ids.push(id);
        world.names.push("");
        world.bios.push("");
        world.ages.push(0);
        world.friends.push(vec![]);
    }

    edit_bio_system(&world.ids, &mut world.bios);
}
```

ECSの世界を作り、コンポーネントを用意し、エンティティに付与しています。
一言でいえば、ECSはDODを扱いやすくしたアーキテクチャなんです。SoAによるメリットを完全に受けられます。
さらに、Rustにおいて1つの構造体の一部を不変参照、別の一部を可変参照として同時に借りるのは困難ですが、ECSのように配列が独立していれば、`edit_bio_system(&world.ids, &mut world.bios);`のように別々に借用を渡すことができます。
