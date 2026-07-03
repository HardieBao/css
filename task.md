# 任务：修仙主题 Live2D 桌宠

> 目标：把额度小组件扩展为「会跑会动的九尾狐桌宠」+「修仙风格额度面板」。
> 狐狸的情绪、行为、配色由 Codex 真实额度驱动。

---

## 一、背景与决策记录

### 1.1 效果图 → 真实数据映射

| 效果图元素 | 数据来源 | 结论 |
| --- | --- | --- |
| 灵力 XX%（环形仪表） | `quota.primary`（5 小时窗口）剩余百分比 | 已有数据 |
| 道蕴 XX%（进度条） | `quota.secondary`（周窗口）剩余百分比 | 已有数据 |
| 复苏 HH:MM:SS（倒计时） | `quota.primary.resetsAt`，前端每秒滴答 | 需新增 ticker |
| 轮回 X天X时 | `quota.secondary.resetsAt` | 已有数据 |
| 状态词 / 提示文案 | 按灵力百分比分档映射 | 纯前端逻辑 |
| 狐狸表情与配色变化 | mood 状态机驱动 | 本任务核心 |
| 场景 7「项目消耗」侧栏 | Codex CLI 无分项目消耗数据 | 砍掉 |
| 背景山水 / 桌面场景 | 属于壁纸，不在应用窗口内 | 不做 |

### 1.2 已确认的技术选型

- **渲染方案：Live2D**（用户选定）。`pixi.js` + `pixi-live2d-display` + Cubism Core 运行时。
- **窗口架构**：新增第三种窗口模式 `pet`（现有 `panel` / `ball` 之外），
  小尺寸透明置顶窗，窗口自身移动 = 狐狸在桌面上跑。
- **两层解耦**：行为引擎（状态机 + 移动）与渲染层（Live2D 模型）分离，
  mood → 动作/表情映射放 JSON 配置，换模型不改代码。
- **模型素材策略**：开发期用 Live2D 官方免费示例模型（moc3 格式，如 Hiyori/Mao）垫场，
  用户后续获取九尾狐模型（Booth / nizima / 委托）后通过配置换皮。

### 1.3 前置认知（预期管理）

- Live2D 是形变动画，移动时为「飘移 + 身体倾斜」，不是四足交替步行。
- Cubism Core 为 Live2D 专有许可（个人/小规模免费），不可修改分发；
  发布前需在 README 标注，官方示例模型仅限开发调试，正式发布前确认许可或换用户自备模型。
- 窗口矩形内的透明区域仍会拦截鼠标点击 → 宠物窗口尺寸裁到最小。

### 1.4 mood 六档状态机（全局共用）

| mood | 触发条件 | 桌宠行为 | 面板配色 |
| --- | --- | --- | --- |
| `full` | 灵力 ≥ 80 | 欢快游走、动作频繁 | 蓝青发光 |
| `steady` | 40 ≤ 灵力 < 80 | 正常散步 | 蓝金 |
| `low` | 1 ≤ 灵力 < 40 | 慢移、叹气坐下 | 橙红告警 |
| `empty` | 灵力 = 0 | 原地睡觉冒 Zzz，停止游走 | 灰白去饱和 |
| `revive` | 检测到从低位跳回满额（重置发生） | 一次性庆祝动作，随后回 `full` | 高亮闪光 |
| `error` | 额度读取失败 | 歪头困惑 + 问号气泡 | 灰 + 红点 |

---

## 二、Phase 1 — Live2D 渲染跑通（最大风险先行）

**目标：pet 窗口里出现一个会呼吸的 Live2D 模型。**

- [ ] 安装依赖：`pixi.js@6.5.10` + `pixi-live2d-display@0.4.0`（已验证兼容组合，锁版本）
- [ ] 下载 Cubism 4 Core（`live2dcubismcore.min.js`）放入 `public/vendor/`，
      在 `index.html` 以普通 script 标签引入（必须先于 pixi-live2d-display 加载）
- [ ] 下载官方示例模型（moc3 格式）放入 `public/live2d/models/<模型名>/`
- [ ] 新建 `src/app/pet/live2d-renderer.js`（工厂函数 `createLive2dRenderer`）：
  - [ ] 透明背景 PIXI Application（`backgroundAlpha: 0`），canvas 挂载到 pet 容器
  - [ ] 加载 `.model3.json`，自动播放 idle 动作组
  - [ ] 暴露接口：`setExpression(name)` / `playMotion(group)` / `setParam(id, value)` /
        `setFlip(bool)` / `pause()` / `resume()` / `destroy()`
- [ ] `index.html` 增加 pet 模式 DOM 容器（`#petRoot`，默认 hidden）
- [ ] 渲染帧率上限 30fps（PIXI ticker `maxFPS`）

**验收**：`npm run tauri:dev` 手动把容器显示出来，模型正常渲染、透明背景无黑底、idle 呼吸循环播放。

---

## 三、Phase 2 — pet 窗口模式 + 行为引擎

**目标：狐狸窗口沿屏幕底边自主游走，可拖拽、可唤回面板。**

### 3.1 窗口模式扩展

- [ ] `src/app/constants.js`：`WIDGET_MODES` 增加 `PET: "pet"`，新增 `PET_SIZE`（约 240×300）
- [ ] `src/app/window-controller.js`：`applyWidgetModeWindow` 增加 pet 分支
      （设尺寸、初始位置贴工作区底边）
- [ ] 检查 `src-tauri/src/settings.rs` 对 `widget_mode` 的校验逻辑，
      如有枚举白名单则加入 `pet`；新增 `pet_position` 持久化字段（Rust 侧 + 前端 DEFAULT_SETTINGS）
- [ ] 模式切换入口：面板工具栏按钮 / 托盘菜单项（`src-tauri/src/tray.rs`）
- [ ] 双击狐狸 → 回 panel 模式（复用 ball 的双击手势逻辑）

### 3.2 行为引擎

- [ ] 新建 `src/app/pet/pet-engine.js`（工厂函数 `createPetEngine`）：
  - [ ] 状态机：`idle / wander / sit / sleep / celebrate / dragged / fall`
  - [ ] 游走循环：`requestAnimationFrame` 驱动，节流 ~30fps 调 `service.window.setPosition`，
        沿工作区底边（任务栏上沿）水平移动
  - [ ] 随机行为决策：走一段 → 停下发呆（idle）→ 坐一会（sit）→ 继续走，权重可配
  - [ ] 屏幕边缘检测 → 转身，调 `renderer.setFlip()` 镜像
  - [ ] 多显示器：复用 `window-controller.js` 现有的 monitor 工作区计算
- [ ] 拖拽：按住拎起（`dragged` 状态，动作暂停）→ 松手在半空 → `fall` 状态坠落回底边（缓动）
- [ ] **性能验证（本 Phase 第一件事）**：Windows 上 30fps `setPosition` 顺滑度实测，
      不顺滑则降步频/加大步长兜底

**验收**：狐狸沿底边来回游走、到边转身、可拖走松手回落、双击回面板；CPU 占用可接受（任务管理器 < 5%）。

---

## 四、Phase 3 — 额度联动（灵魂所在）

**目标：额度变化实时改变狐狸的情绪与行为。**

- [ ] 新建 `src/app/pet/mood.js`：`deriveMood(quota, error)` 纯函数，
      按 1.4 表格分档；`revive` 检测 = 上次灵力 < 40 且本次 ≥ 95
- [ ] 新建 `public/live2d/pet-config.json`：mood → `{ expression, motionGroup, wanderSpeed, wanderEnabled }`
      映射表（换模型只改此文件）
- [ ] `quota-controller.js` 刷新成功/失败后通知 pet 引擎更新 mood
- [ ] mood 行为接线：
  - [ ] `full`：游走速度快、动作组活跃
  - [ ] `low`：速度减半、周期性播叹气/坐下
  - [ ] `empty`：强制 `sleep`，停止游走，头顶 Zzz 气泡（DOM 覆盖层）
  - [ ] `revive`：播一次庆祝动作 + 光环 CSS 动效，完毕回 `full`
  - [ ] `error`：问号气泡
- [ ] 点击狐狸 → 弹信息气泡（DOM）：灵力 % / 道蕴 % / 复苏倒计时，几秒后自动收起；再点击打开面板
- [ ] 眼神跟随鼠标：后端新增 Tauri 命令包装 `cursor_position()`（`src-tauri/src/commands.rs`），
      前端低频轮询（~10Hz）喂给模型 `ParamAngleX/Y` 视线参数；失败则降级为窗口内跟随

**验收**：改刷新间隔到 1 分钟实测六种 mood 切换；断网/改错 CLI 路径验证 `error`；等待一次真实重置或 mock 验证 `revive`。

---

## 五、Phase 4 — 狐狸模型接入（依赖用户素材）

- [ ] 用户获取九尾狐 Live2D 模型（渠道：Booth / nizima / itch.io / 委托建模），
      要求：Cubism 3+（moc3）、含 idle 动作组、最好含表情文件（.exp3.json）
- [ ] 模型放入 `public/live2d/models/fox/`
- [ ] 更新 `pet-config.json` 的模型路径与动作/表情名映射
- [ ] 逐 mood 核对动作效果，缺失的表情用参数覆盖（`setParam`）兜底
- [ ] 确认模型与 Cubism Core 的分发许可，README 补充署名/许可说明

**验收**：狐狸模型完整替换示例模型，六种 mood 表现正确。

---

## 六、Phase 5 — 修仙面板主题（效果图的面板部分）

**目标：panel 模式下的修仙风格新主题 `xianxia`。**

- [ ] `src/app/constants.js`：`THEMES` 增加 `xianxia`（中英 label），i18n 增加词条：
      灵力/道蕴/复苏/轮回/状态词六档/提示文案
- [ ] `index.html`：新增修仙面板 DOM（与现有面板互斥显隐，`body[data-theme="xianxia"]` 控制）：
  - [ ] SVG 环形灵力仪表（stroke-dasharray 驱动 + 发光滤镜）
  - [ ] 道蕴横向进度条
  - [ ] 复苏倒计时（HH:MM:SS，每秒滴答，仅该主题激活时运行）
  - [ ] 轮回剩余（X天X时）
  - [ ] 状态行 + 提示胶囊（✨ 常态 / ⚠ 告警）
  - [ ] 0% 时显示「推荐行动」静态建议区（图 4）
- [ ] 新建 `src/themes/xianxia.css`：毛玻璃 + 金描边 + `data-mood` 四套配色变量
      （蓝青/金/橙红/灰白），动效遵守 `prefers-reduced-motion`
- [ ] 面板尺寸按主题区分（xianxia 约 500×360），`window-controller.js` 支持 per-theme panel size
- [ ] 渲染逻辑接入 `src/app/render.js`（复用 mood 派生，禁止重复实现分档）

**验收**：设置里切到修仙主题，六档配色与状态词正确，倒计时逐秒走，切回旧主题无残留。

---

## 七、Phase 6 — 打磨与收尾

- [ ] 设置项：桌宠游走开关（关闭 = 原地待机）、活动范围（当前显示器/所有显示器）
- [ ] 性能：`sleep`/`idle` 状态降 PIXI 帧率至 10fps；窗口隐藏时 `pause()` 渲染
- [ ] 托盘菜单补齐三模式切换
- [ ] 体积检查：pixi + core 约 +600KB，确认安装包增量可接受
- [ ] README 更新：桌宠功能说明、模型许可声明、自备模型换皮指南
- [ ] 回归：`cargo test --manifest-path src-tauri/Cargo.toml`（Rust 侧 settings 枚举变更）
- [ ] 手动回归：panel / ball / pet 三模式互切、重启后模式与位置恢复、多显示器拖拽

---

## 八、风险清单

| 风险 | 等级 | 缓解 |
| --- | --- | --- |
| Windows 高频 `setPosition` 不顺滑 | 高 | Phase 2 首项实测；降步频、加步长、必要时改「瞬移 + 原地动画」 |
| pixi-live2d-display 与 PIXI 版本兼容 | 中 | 锁定 pixi@6.5.10 + 0.4.0 组合，不追新 |
| 九尾狐模型迟迟不到位 | 中 | 示例模型垫场不阻塞；配置化换皮使素材完全解耦 |
| Cubism Core / 示例模型许可 | 中 | 发布前核对许可条款，README 标注；示例模型不进正式发布包 |
| WebGL 常驻渲染功耗 | 中 | 30fps 上限 + 睡眠降帧 + 隐藏暂停 |
| 透明区域拦截鼠标点击 | 低 | pet 窗口尺寸裁到最小 |
| `settings.rs` 校验拒绝 `pet` 模式 | 低 | Phase 2 首先检查后端枚举，同步扩展 + 单测 |

## 九、验证命令

```powershell
# 前端 + 桌面壳联调
npm run tauri:dev

# Rust 单测（settings / quota 变更后必跑）
cargo test --manifest-path src-tauri/Cargo.toml
```

## 十、进度跟踪

| Phase | 状态 | 备注 |
| --- | --- | --- |
| Phase 1 渲染跑通 | 未开始 | |
| Phase 2 窗口模式 + 行为引擎 | 未开始 | |
| Phase 3 额度联动 | 未开始 | |
| Phase 4 狐狸模型接入 | 未开始 | 等用户素材，不阻塞其他 Phase |
| Phase 5 修仙面板主题 | 未开始 | 可与 Phase 2/3 并行 |
| Phase 6 打磨收尾 | 未开始 | |
