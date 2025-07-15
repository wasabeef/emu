# テスタビリティ向上のためのリファクタリング計画

## 概要

Emu プロジェクトのテスタビリティを向上させるため、**挙動を変えずに**内部実装を改善します。依存性注入、関数分割、エラーハンドリングの改善により、テストカバレッジを効率的に向上させます。

## 基本原則

1. **挙動の保持**: 外部から見た動作は一切変更しない
2. **段階的変更**: 小さな変更を積み重ねる
3. **テストファースト**: リファクタリング前にテストを書く

## Phase 1: CommandRunner の抽象化（優先度: 高）

### 現状の問題

```rust
pub struct AndroidManager {
    command_runner: CommandRunner,  // 直接埋め込み
    // ...
}
```

### 改善案

```rust
// トレイトの定義
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn run(&self, command: &str, args: &[&str]) -> Result<String>;
    async fn spawn(&self, command: &str, args: &[&str]) -> Result<u32>;
    async fn run_with_retry(&self, command: &str, args: &[&str], retries: u32) -> Result<String>;
}

// 実装
#[async_trait]
impl CommandExecutor for CommandRunner {
    async fn run(&self, command: &str, args: &[&str]) -> Result<String> {
        self.run(command, args).await
    }
    // ...
}

// AndroidManager の変更
pub struct AndroidManager {
    command_executor: Box<dyn CommandExecutor>,  // 抽象化
    // ...
}

impl AndroidManager {
    pub fn new() -> Result<Self> {
        Self::with_executor(Box::new(CommandRunner::new()))
    }

    pub fn with_executor(executor: Box<dyn CommandExecutor>) -> Result<Self> {
        // テスト時はモックを注入可能
    }
}
```

### メリット

- テスト時にモックを注入可能
- 外部依存を分離
- 挙動は完全に同一

## Phase 2: パース関数の分割（優先度: 中）

### 現状の問題

```rust
pub fn parse_avd_list_output(output: &str) -> Vec<AndroidDevice> {
    // 200 行以上の巨大関数
}
```

### 改善案

```rust
pub fn parse_avd_list_output(output: &str) -> Vec<AndroidDevice> {
    AvdListParser::new(output).parse()
}

struct AvdListParser<'a> {
    lines: std::str::Lines<'a>,
    current_line: Option<&'a str>,
}

impl<'a> AvdListParser<'a> {
    fn new(output: &'a str) -> Self {
        Self {
            lines: output.lines(),
            current_line: None,
        }
    }

    fn parse(mut self) -> Vec<AndroidDevice> {
        let mut devices = Vec::new();
        while let Some(device) = self.parse_next_device() {
            devices.push(device);
        }
        devices
    }

    fn parse_next_device(&mut self) -> Option<AndroidDevice> {
        // 単一デバイスのパース（テスト可能）
    }

    fn extract_device_name(&self, line: &str) -> Option<String> {
        // 名前抽出ロジック（個別テスト可能）
    }

    fn extract_api_level(&self, line: &str) -> Option<u32> {
        // API レベル抽出（個別テスト可能）
    }
}
```

### メリット

- 各メソッドを個別にテスト可能
- ロジックの見通しが良くなる
- エッジケースのテストが容易

## Phase 3: エラーハンドリングの改善（優先度: 中）

### 現状の問題

```rust
let api_level = parts[1].parse::<u32>().unwrap();  // panic の可能性
```

### 改善案

```rust
let api_level = parts.get(1)
    .ok_or_else(|| DeviceError::ParseError("Missing API level".into()))?
    .parse::<u32>()
    .map_err(|e| DeviceError::ParseError(format!("Invalid API level: {}", e)))?;
```

### メリット

- panic を回避
- エラーメッセージが明確
- テストでエラーケースを検証可能

## Phase 4: IosManager の改善（優先度: 中）

### 現状の問題

- プラットフォーム依存コードが散在
- JSON パースエラーが不明瞭

### 改善案

```rust
#[cfg(target_os = "macos")]
impl IosManager {
    pub fn new_with_executor(executor: Box<dyn CommandExecutor>) -> Result<Self> {
        // 依存性注入対応
    }
}

// JSON パース用の中間構造体
#[derive(Deserialize, Debug)]
struct SimctlOutput {
    devices: HashMap<String, Vec<SimulatorDevice>>,
}

#[derive(Deserialize, Debug)]
struct SimulatorDevice {
    name: String,
    udid: String,
    state: String,
    #[serde(rename = "isAvailable")]
    is_available: bool,
}
```

## Phase 5: テストヘルパーの整備（優先度: 高）

### モックビルダー

```rust
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub struct MockCommandExecutor {
        responses: HashMap<String, String>,
    }

    impl MockCommandExecutor {
        pub fn new() -> Self {
            Self { responses: HashMap::new() }
        }

        pub fn with_response(mut self, command: &str, response: &str) -> Self {
            self.responses.insert(command.to_string(), response.to_string());
            self
        }
    }

    #[async_trait]
    impl CommandExecutor for MockCommandExecutor {
        async fn run(&self, command: &str, args: &[&str]) -> Result<String> {
            let key = format!("{} {}", command, args.join(" "));
            self.responses.get(&key)
                .cloned()
                .ok_or_else(|| anyhow!("No mock response for: {}", key))
        }
    }
}
```

## 実装順序

1. **Week 1**: CommandExecutor トレイトの導入
   - AndroidManager への適用
   - 既存テストが全て通ることを確認
2. **Week 2**: パース関数の分割
   - AvdListParser の実装
   - 単体テストの追加
3. **Week 3**: エラーハンドリング改善
   - unwrap() の除去
   - エラーケースのテスト追加
4. **Week 4**: IosManager の改善
   - 依存性注入対応
   - JSON パースの堅牢化

## 期待される成果

- **テストカバレッジ**: +10-15% の向上
- **テスト作成時間**: 50% 削減
- **バグ発見率**: 早期段階での発見が増加
- **保守性**: コードの可読性と変更容易性が向上

## 注意事項

- 各ステップで必ず全テストが通ることを確認
- パフォーマンスのベンチマークを取り、劣化がないことを確認
- レビューを段階的に実施し、挙動の変更がないことを検証
