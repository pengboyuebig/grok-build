## 目标
在 **Settings → Custom AI Model Config** 中将“配置名称”和“实际调用模型 ID”分离：

```toml
[model.acme-gpt]
name = "acme-gpt"
model = "gpt-4o-2024-08-06"
base_url = "https://api.example.com/v1"
api_key = "..."
api_backend = "openai"
```

这里：
- **Name**：`acme-gpt`，用于 `[model.<name>]` 配置表名、编辑/删除目标以及模型选择器的显示名称。
- **Custom Model**：`gpt-4o-2024-08-06`，作为请求发往模型供应商时实际使用的模型 ID。

## 实施步骤
1. **补充设置字段与状态映射**
   - 在 `crates/codegen/xai-grok-pager/src/settings/defs.rs` 的 `Custom AI Model Config` 分组中新增 `Name` 字符串字段。
   - 保持现有 `Custom Model` 字段，但更新其说明为“供应商的实际模型 ID”。
   - 在 `xai-grok-shared` 的 `UiConfig` 中增加一个持久化字段，用于保存当前表单中的实际模型 ID；保留既有 `custom_model_selected` 作为配置名称，避免破坏已有用户配置和当前删除逻辑。
   - 将新增字段接入 settings registry、字符串编辑 action、setter 与 `[ui]` 持久化效果。

2. **调整保存和更新语义**
   - 将 `SaveCustomApiConfig` 的模糊 `model` 参数拆成 `name` 与 `model_id`。
   - 保存前要求 Base URL、Name、Custom Model 都非空。
   - `Name` 决定写入/覆盖哪个 `[model.<name>]` 表；相同 Name 即更新已有配置，不同 Name 即新增配置。
   - `Custom Model` 写入 TOML 的 `model = "<model_id>"`，不再被用作 TOML 表名。
   - 保持保存后的模型目录刷新逻辑。

3. **调整删除语义和界面提示**
   - 删除操作只使用 Name（即 `[model.<name>]` 的名称）定位目标。
   - 延续现有可见的两次 `x` 确认：第一次显示确认提示，第二次才删除，任意其他按键取消。
   - 删除当前 Name 时同步清空对应的 UI 名称字段，并保持现有持久化清理行为。

4. **更新 shell 配置写入器**
   - 扩展 `save_custom_api_model` / `upsert_custom_model_section`，接收独立的 `name` 和 `model_id`。
   - 用 Name 写入嵌套 TOML 表键与 `name` 字段；用 model_id 写入 `model` 字段。
   - 保留原子写入、写锁、旧式 dotted key 兼容迁移，以及重载模型目录的流程。

5. **测试与构建验证**
   - 扩展 TOML 写入单元测试，验证表名、`name`、`model` 三者各自正确：Name 不会误写为供应商模型 ID。
   - 运行相关 `cargo test`（shell 配置写入器及 pager 设置路径可覆盖的测试）。
   - 执行 `cargo check -p xai-grok-pager -p xai-grok-shell`。
   - 构建 `cargo build -p xai-grok-pager-bin --release`，并验证 `target\release\xai-grok-pager.exe --version`。

## 兼容性
已有用户写入的 `[model.<旧名称>]` 不会被迁移或删除。新字段只影响之后通过设置页新建或更新的模型；更新旧配置时，填写其现有的 Name 即可覆盖该配置。