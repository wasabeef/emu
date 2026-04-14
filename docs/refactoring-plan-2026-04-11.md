# Emu リファクタリング計画

Date: 2026-04-11

## 目的

この文書は、Emu コードベース向けに整理し直したリファクタリング計画である。
現在の実装、現在のテスト、および初稿に対する 2 回目の自己レビューをもとにまとめている。

狙いは、挙動を変えずに安全に変更しやすいコードベースへ寄せること。

- `App` を薄い orchestrator にする
- `AndroidManager` と `IosManager` を責務ごとに分割する
- `platform -> app` の依存逆転を解消する
- state logic を凝集させて test しやすくする
- 高リスクな巨大ファイルを、挙動を変えずに縮小する

## 挙動維持の契約

このリファクタリングは、厳密な挙動維持の契約のもとで進める。

重要な前提:

- 完全な形式仕様なしに、挙動が 100% 変わらないことを数学的に証明することはできない
- そのため、この計画ではこの repository で取り得る最も強い実務上の保証を使う
  - 挙動維持を前提にした PR ルール
  - characterization test
  - 安定した verification command
  - rollback checkpoint

この project における「挙動を変えない」とは、次を意味する。

1. 同じ user input が同じ visible mode transition を生むこと
2. 同じ device operation が同じ platform command と state transition を引き起こすこと
3. 同じ startup path が同じ loading、details、log coordination を生むこと
4. 同じ cache read / write ルールが維持されること
5. 既存 test が保証している render contract が維持されること
6. structural PR が copy、ordering、command semantics、timing policy、error handling を意図的に変えないこと

この条件のどれかが変わるなら、その PR は structural-only ではなく、behavior change として扱う。

## 現状

主な hotspot は次のとおり。

- [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs) `4317` 行
- [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs) module extraction 開始前で `3924` 行
- [src/app/state/mod.rs](/Users/a12622/git/emu/src/app/state/mod.rs) state module extraction 開始前で `1668` 行
- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs) `1510` 行
- [src/managers/ios/mod.rs](/Users/a12622/git/emu/src/managers/ios/mod.rs) module extraction 開始前で `1466` 行

これらの file は、複数責務を同時に持っていた。

## 進捗スナップショット

この計画は、必要な structural phase を最後まで実行済みである。

完了した structural checkpoint:

- `DeviceDetails` を `app::state` から抽出
- `ApiLevelCache` を `app::state` から抽出
- `src/app/state.rs` を `src/app/state/mod.rs` と sibling module 群へ変換
- app helper module を抽出
  - `api_levels.rs`
  - `background.rs`
  - `details.rs`
  - `logs.rs`
  - `refresh.rs`
  - `input.rs`
  - `create_device.rs`
  - `device_actions.rs`
  - `tests.rs`
- app state helper module を抽出
  - `ui.rs`
  - `logs.rs`
  - `cache.rs`
  - `api_levels.rs`
  - `details.rs`
  - `forms.rs`
  - `navigation.rs`
  - `notifications.rs`
  - `tests.rs`
- `src/managers/android.rs` を `src/managers/android/mod.rs` へ変換
- `src/managers/ios.rs` を `src/managers/ios/mod.rs` へ変換
- Android helper module を抽出
  - `parser.rs`
  - `sdk.rs`
  - `version.rs`
  - `details.rs`
  - `create.rs`
  - `install.rs`
  - `discovery.rs`
  - `lifecycle.rs`
  - `tests.rs`
- iOS helper module を抽出
  - `discovery.rs`
  - `details.rs`
  - `lifecycle.rs`
  - `tests.rs`
- UI helper module を抽出
  - `dialogs/mod.rs`
  - `dialogs/create_device.rs`
  - `dialogs/confirmation.rs`
  - `dialogs/api_levels.rs`
  - `dialogs/notifications.rs`
  - `panels/mod.rs`
  - `panels/device_lists.rs`
  - `panels/details.rs`
  - `panels/logs.rs`
  - `panels/commands.rs`

最新 checkpoint 時点の file size:

- [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs) `234` 行
- [src/app/tests.rs](/Users/a12622/git/emu/src/app/tests.rs) `1082` 行
- [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs) `517` 行
- [src/managers/android/tests.rs](/Users/a12622/git/emu/src/managers/android/tests.rs) `943` 行
- [src/app/state/mod.rs](/Users/a12622/git/emu/src/app/state/mod.rs) `337` 行
- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs) は render shell に整理済みで、[src/ui/dialogs/mod.rs](/Users/a12622/git/emu/src/ui/dialogs/mod.rs) と [src/ui/panels/mod.rs](/Users/a12622/git/emu/src/ui/panels/mod.rs) へ分割済み
- [src/models/device_info/mod.rs](/Users/a12622/git/emu/src/models/device_info/mod.rs) は `device_info` の entrypoint になり、[priority.rs](/Users/a12622/git/emu/src/models/device_info/priority.rs)、[parsing.rs](/Users/a12622/git/emu/src/models/device_info/parsing.rs)、[tests.rs](/Users/a12622/git/emu/src/models/device_info/tests.rs) へ分割済み

現在のレビュー方針:

- 抽出済みの箇所は、今後も完全に挙動維持する
- これ以上の structural split は、review 価値が churn を上回るときだけ行う
- すべての structural checkpoint は targeted test と `cargo clippy --all-targets --all-features -- -D warnings` を継続通過する
- policy change、parsing correction、fallback adjustment は必ず別の behavior commit に分ける

## 完了レビュー

Date: 2026-04-13

この refactor は、structural な目標としては実質完了の状態に到達した。

完了した成果:

- `App` は薄い orchestration shell になった
- `app/state/` は責務ごとに分割された
- Android / iOS manager は読みやすい facade module と focused helper sibling に整理された
- `ui` rendering は dialogs / panels に分割された
- `device_info` は monolithic file ではなくなった
- 各 structural checkpoint 後も full verification を通し続けられた

追加のレビュー結論:

- [src/ui/widgets.rs](/Users/a12622/git/emu/src/ui/widgets.rs)、[src/managers/common.rs](/Users/a12622/git/emu/src/managers/common.rs)、[src/models/device.rs](/Users/a12622/git/emu/src/models/device.rs)、[src/models/error.rs](/Users/a12622/git/emu/src/models/error.rs) をこれ以上分割する理由は現時点では弱い
- これらの file はサイズこそあるが、責務の一貫性はまだ保たれている
- さらに structural churn を入れると、保守性の改善より review quality の低下が先に来やすい
- 将来ここへ手を入れるなら、file size ではなく behavior change や feature pressure をきっかけにする

## 次の UX タスク

以下は structural refactor PR の scope 外であり、別の follow-up PR で扱う。

- `Create Android Device` で Android system image が未 install のときの empty state を改善する
- Android API level list が空のときは device creation を無効化、または明確に block する
- 現在の誤解を招く placeholder (`Pixel 9 API`) を、前提不足が分かる copy に置き換える
- Android system image installation を先に開く導線を追加する

## パフォーマンス改善計画

Date: 2026-04-13

優先対象:

- Android device list の表示と refresh の遅さ
- Android system images list の表示と refresh の遅さ

### パフォーマンス改善の契約

この改善では次を守る。

1. speed のために correctness を落とす変更は、明示レビューなしでは入れない
2. cache invalidation のルールは、理解可能で test 可能な形を保つ
3. list 表示を速くする目的で stale な Android SDK data を再導入しない
4. performance 由来の behavior change は、無関係な refactor と分離する
5. benchmark threshold の変更は、測定された cost model の変化で説明できること

### 現在の hot path

#### 1. Android device list

- [src/managers/android/lifecycle.rs](/Users/a12622/git/emu/src/managers/android/lifecycle.rs)
  - `list_devices_parallel()`

主なコスト源:

- `avdmanager list avd` parsing
- `adb devices` status mapping
- per-device API と version の推定
- API level、device priority、fallback name による stable sorting

メモ:

- stable sort 自体は必要
- 課題は sort を消すことではなく、sorted path を軽くすること

#### 2. Android system images list

- [src/managers/android/install.rs](/Users/a12622/git/emu/src/managers/android/install.rs)
  - `list_api_levels()`
- [src/app/api_levels.rs](/Users/a12622/git/emu/src/app/api_levels.rs)
  - `open_api_level_management()`
  - install / uninstall 後の refresh flow

主なコスト源:

- `sdkmanager --list --verbose` の full command 実行
- dialog を開くたびの package list 全量再 parse
- install / uninstall 後の再 parse
- variant と installed-state map の再構築

メモ:

- stale disk cache の誤用は correctness fix で解消済み
- 次にやるのは stale persistence の復活ではなく、安全な session-level reuse

#### 3. Startup と background preloading

- [src/app/background.rs](/Users/a12622/git/emu/src/app/background.rs)
  - `start_background_device_loading()`

主なコスト源:

- eager な Android target loading
- eager な device loading
- startup 時に metadata work と list work が近接していること

### 測定戦略

既存で使う測定:

- [tests/performance/startup_benchmark_test.rs](/Users/a12622/git/emu/tests/performance/startup_benchmark_test.rs)
  - startup benchmark
  - device list benchmark
  - UI render benchmark
- [tests/performance/memory_usage_test.rs](/Users/a12622/git/emu/tests/performance/memory_usage_test.rs)
  - repeated device list performance

次に追加する測定:

1. Android system images dialog open benchmark
   - cold open の `list_api_levels()` コスト
   - 同 session での reopen コスト
2. Android device refresh の段階別 timing
   - discovery time
   - status mapping time
   - metadata enrichment time
   - sort time
3. install / uninstall 後の refresh timing
   - operation 完了から更新済み images list が visible になるまでの時間

ルール:

- 大きな cache 変更や refresh 戦略変更の前に、まず測定を追加する

### Phase 1: 挙動を変えない cheap win

広い refresh model は変えず、現実装で顕在化している遅さを減らす。

#### 1A. Android device list path

予定:

- stable sort は維持しつつ、sort key の再計算を減らす
- hot path 上の repeated string normalization / parsing を避ける
- 「表示用の重い metadata」と「ordering 用の軽い key」を必要に応じて分ける
- tests または debug logging で stage-level timing を入れる

完了条件:

- ordering regression がない
- parsing regression がない
- correctness を維持した sort のまま、device list benchmark の余裕が戻る

#### 1B. Android system images list path

予定:

- 同一 UI session 内での重複 `sdkmanager --list --verbose` 呼び出しを減らす
- session-scoped な parsed images snapshot を導入する
- install / uninstall 後に明示的に invalidate する
- create-device 側の API availability は installed / live state に基づく形を維持する

完了条件:

- stale installed-state regression がない
- 同一 session 内の dialog reopen が cold open より意味のある速度改善になる
- install / uninstall 後も最新 state へ refresh される

### Phase 2: 現行アーキテクチャ内での metadata cache

hot path が毎回計算しすぎている部分へ cache を入れる。ただし full refresh model の作り直しはまだやらない。

#### 2A. Android device metadata cache

候補:

- `AVD name -> api_level`
- `AVD name -> display version`
- `AVD name -> sort priority`
- `AVD name -> device type / category hint`

置き場所候補:

- [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs)
- [src/models/device_info/mod.rs](/Users/a12622/git/emu/src/models/device_info/mod.rs)
- [src/app/state/cache.rs](/Users/a12622/git/emu/src/app/state/cache.rs)

方針:

- まずは session-scoped に留める
- create / delete / wipe、または source config change で invalidate する
- mutable な SDK state に対して、opaque な長寿命 disk cache は復活させない

#### 2B. Android images list session cache

候補:

- parse 済みの `sdkmanager --list --verbose` 結果
- installed variant を組み立て済みの `ApiLevel` entry

invalidate 契機:

- install success
- uninstall success
- API management screen での explicit refresh
- app restart

### Phase 3: 軽い status refresh と重い metadata refresh の分離

refresh 戦略に踏み込む最初の phase。Phase 1 / 2 の測定後に専用 PR で扱う。

#### 3A. Android device refresh model

予定:

- stable な metadata snapshot を保持する
- runtime status refresh を分離する
- expensive な metadata 再計算は changed / newly discovered device だけに限定する

#### 3B. Startup sequencing

予定:

- まず「UI に useful な list が出せる」ことを優先する
- 安全な範囲で、二次的な Android metadata work を initial visible list の後ろへ回す

ガードレール:

- existing startup / detail / log coordination contract を崩さないこと

### パフォーマンス改善の PR 順序

推奨順序:

1. Android device list / images list 向け benchmark と instrumentation の改善
2. `list_devices_parallel()` 内の device-list hot path cleanup
3. `list_api_levels()` 向け session cache
4. Android device metadata 向け session cache
5. diff-oriented refresh、または status / metadata refresh の分離

### 早すぎる段階でやらないこと

測定で必要性が見えるまでは避ける。

- 新しい external cache library の導入
- simpler な session cache を試す前に、複雑な cross-thread cache sharing へ進むこと
- command latency が本丸か分からないうちに、広い async fan-out を入れること
- より強い invalidation 設計なしに、mutable な Android SDK list state を再び disk へ持続化すること

### パフォーマンス PR のレビュー観点

各 performance PR は、少なくとも次を明示できること。

1. どの path が速くなったか
2. 何の測定でそれを示すか
3. どの cache または shortcut を入れたか
4. その invalidate 条件は何か
5. user-visible behavior の何を維持したか
6. benchmark threshold を変えたなら、その理由は何か

## 依存関係の棚卸し

この section では、元々の依存問題と、現在残っている許容範囲の依存を分けて整理する。

### この refactor で解消した過去の依存逆転

1. [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs)
   - `crate::app::state::DeviceDetails` を import しなくなった
   - `crate::app::state::ApiLevelCache` を import しなくなった

2. [src/managers/ios/mod.rs](/Users/a12622/git/emu/src/managers/ios/mod.rs)
   - `crate::app::state::DeviceDetails` を import しなくなった

3. [src/models/device_info/mod.rs](/Users/a12622/git/emu/src/models/device_info/mod.rs)
   - 旧来の monolithic `device_info.rs` layout をやめた
   - device info test は dedicated module へ移した

### 現時点で許容できる依存

1. [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs)
   - `AppState` に依存した rendering は許容できる

2. [src/ui/widgets.rs](/Users/a12622/git/emu/src/ui/widgets.rs)
   - `Panel` に依存した widget behavior は許容できる

これらは lower-layer inversion ではなく、現時点でこれ以上の churn を正当化するほどではない。

## 挙動 lock の対象

この refactor 中に明示的に lock する挙動は次のとおり。

- `App::new()` から始まる startup sequence
- background cache loading
- background device loading
- panel switching behavior
- device details refresh behavior
- log stream coordination behavior
- create device workflow behavior
- delete device workflow behavior
- wipe device workflow behavior
- Android target cache behavior
- Android / iOS の device detail construction behavior
- 既存 test が保証している current render contract

これらのどれかを変える PR は、必ず split して behavior PR として扱う。

## 自己レビューで見つかったこと

最初の draft は方向性自体は正しかったが、実装順序を tighten すべき箇所がいくつかあった。

### Finding 1: top-level rename は早すぎる

初稿では `src/domain/`、`src/platform/`、`src/state/` を早い段階で導入していた。
しかしそれでは、責務分離が済む前に import churn が大きくなりすぎる。

見直した判断:

- first wave では current top-level module name を維持する
- まず既存 root の内側で分割する
- broad rename は構造が安定してから再検討する

つまり:

- `models/` は当面維持する
- `managers/` は当面維持する
- state は top-level `state/` へ出さず、`app/state/` の下に置く

### Finding 2: `DeviceDetails` だけ動かしても不十分

refactor 開始時点では、manager は `crate::app::state::DeviceDetails` を返していた。
もし `DeviceDetails` を移しても、その中に `app::state::Panel` が残っていれば、依存逆転は解消されない。

見直した判断:

- `DeviceDetails` は `app::state` から切り離す
- `platform` field は [src/models/platform.rs](/Users/a12622/git/emu/src/models/platform.rs) の `Platform` を使う
- 抽出後の details model に `app` 由来の型は残さない

これは最初に切るべき、意味のある architecture 上の境界だった。

同じ周辺にはもう 1 つ問題があった。

- `AndroidManager` が `app::state` 由来の `ApiLevelCache` を read / write していた

つまり本当の課題は「1 つ型を動かすこと」ではなく、

- UI state ではない `managers -> app::state` の data-type dependency をまとめて外すこと

だった。

### Finding 3: `app/state.rs` は `app` の下に置くべき

初稿では state を top-level `src/state/` に出していたが、レビュー後にそれは最善ではないと判断した。

理由:

- `AppState` は reusable な domain model ではなく application state である
- `ui` rendering は自然にこれへ依存する
- `app/` 配下に残したほうが churn が減り、ownership も明確になる

見直した判断:

- `src/app/state.rs` は `src/app/state/` submodule 群へ分割する
- first wave では top-level package へは移さない

### Finding 4: Rust の file-to-directory 変換は専用 PR に分ける

次の変換は structural だが review ノイズが大きい。

- `src/app/state.rs` -> `src/app/state/mod.rs`
- `src/managers/android.rs` -> `src/managers/android/mod.rs`
- `src/managers/ios.rs` -> `src/managers/ios/mod.rs`

ここに code extraction を混ぜると、review quality が落ちる。

見直した判断:

- 各 file-to-directory 変換は dedicated な structural PR に分ける
- その PR では挙動を完全に維持する
- その後で sibling module の抽出へ進む

### Finding 5: `ui -> app::state` は主問題ではない

初稿では `ui` が `AppState` を import していることも早く減らしたい問題として扱っていた。
しかし、それはこの project の主問題ではない。

`ui` rendering が application state に依存すること自体は自然である。
本当に危険なのは、lower layer が higher layer に依存すること。

見直した判断:

- 優先するのは `managers -> app` 依存の除去
- `ui -> app::state` cleanup は secondary、かつ optional とする

### Finding 6: top-level package rename は churn に見合わない可能性が高い

`models -> domain` や `managers -> platform` という rename は、最終形としては魅力がある。
ただし自己レビューの結果、これは optional かつ後半に回すべきだと判断した。

見直した判断:

- first wave では top-level package rename を必須にしない
