<!-- markdownlint-disable MD033 MD026 MD034 -->

<div align="center">
  <img src="public/logo.png" alt="QuantumTV Logo" width="120" height="auto">
  <h1>QuantumTV</h1>
  <p><strong>跨平台 · 本地优先 · 零账号 的开源影视聚合播放器</strong></p>
  <p>用 Rust + Tauri 打造,装机即用,数据只在你自己电脑里。</p>

  <p>
    <a href="https://github.com/Geon97/QuantumTV/releases/latest">下载最新版</a> ·
    <a href="#-快速开始">快速开始</a> ·
    <a href="#-功能特性">功能特性</a> ·
    <a href="#-tvbox-配置">TVBox</a> ·
    <a href="#-安全与隐私">隐私说明</a>
  </p>
</div>

<p align="center">
  <img src="https://img.shields.io/badge/Windows-supported-0078D6?logo=windows" />
  <img src="https://img.shields.io/badge/macOS-supported-000?logo=apple" />
  <img src="https://img.shields.io/badge/Linux-supported-FCC624?logo=linux&logoColor=black" />
  <img src="https://img.shields.io/badge/Android-supported-3DDC84?logo=android&logoColor=white" />
</p>

> ⚖️ 本项目仅用于 Rust / Tauri / 现代 Web 技术的学习与研究,**不提供任何影视内容**,所有播放源由用户自行配置。使用前请阅读 [安全与隐私](#-安全与隐私) 与 [免责声明](#-免责声明)。

---

## 为什么是 QuantumTV?

市面上的"影视聚合"工具大多是浏览器扩展或 Electron 套壳,体积动辄上百 MB、内存吃满、还要联网账号。QuantumTV 想做的事很简单:

- 🪶 **足够轻** —— Tauri + 系统 WebView,安装包数 MB,启动毫秒级,内存占用是 Electron 同类产品的 1/5。
- 🦀 **足够安全** —— 核心逻辑跑在 Rust,无 Node.js Runtime 暴露,关键路径走 Tauri 权限模型。
- 🔒 **足够私密** —— 无账号体系,无云同步,无遥测。播放记录、收藏、配置全部存在本地 SQLite,卸载即清零。
- 🧩 **足够开放** —— 不绑死任何资源站,支持 CMS 标准接口和 TVBox 订阅,你的源你做主。

## ✨ 功能特性

### 播放与浏览

- 🔍 **多源聚合搜索** —— 一次查询同时命中所有已配置的资源站,自动去重合并
- ▶️ **完整播放体验** —— 基于 Plyr + HLS.js,支持倍速、画质切换、记忆进度、上次播放位置一键继续
- ⏭️ **片头片尾跳过** —— 单剧/全局两级配置,按集精确到秒
- 💾 **观看历史 & 收藏夹** —— 全本地,跨剧集自动汇聚
- 🎯 **个性化推荐** —— 基于本地播放历史的离线推荐引擎,数据不出本机
- 🌐 **豆瓣发现** —— 热门电影/剧集/综艺/新番榜单,带评分与年份过滤

### 工程与体验

- 🪟 **多平台原生壳** —— Windows / macOS / Linux / Android 一份代码
- ⌨️ **老板键** —— <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>X</kbd> 瞬间隐藏/显示窗口
- 🚦 **源智能调度** —— 自动统计每个源的响应时延和成功率,慢/失败源自动降权
- 🔄 **配置订阅** —— 支持远程订阅 URL,后台 24h 自动拉取更新
- 📺 **TVBox 兼容** —— 内置 `/api/tvbox` 端点,直接当 TVBox 后端用
- 🛡️ **CMS 全量代理** —— 桌面端原生网络栈,彻底告别 CORS 和 Mixed Content

## 🚀 快速开始

### 普通用户:下载预编译版本

直接到 [Releases 页面](https://github.com/Geon97/QuantumTV/releases/latest) 下载对应平台的安装包:

| 平台                  | 推荐文件                                   |
| --------------------- | ------------------------------------------ |
| Windows               | `QuantumTV_x.y.z_x64-setup.exe`            |
| macOS (Apple Silicon) | `QuantumTV_x.y.z_aarch64.dmg`              |
| macOS (Intel)         | `QuantumTV_x.y.z_x64.dmg`                  |
| Linux                 | `quantum-tv_x.y.z_amd64.AppImage` / `.deb` |
| Android               | `app-universal-release.apk`                |

安装后首次启动会是空壳,进入 **设置 → 配置文件** 填入你自己的资源站 JSON 即可使用。

### 开发者:从源码构建

```bash
# 依赖: Node.js 20+, Rust 1.90+, 平台对应的 Tauri 系统依赖
git clone https://github.com/Geon97/QuantumTV.git
cd QuantumTV

npm install
npm run tauri dev        # 开发模式
npm run tauri build      # 打生产包
```

构建 Android 需要额外配置 Android SDK / NDK,详见 [Tauri 移动端文档](https://v2.tauri.app/start/prerequisites/#android)。

## ⚙️ 配置文件示例

```json
{
  "cache_time": 7200,
  "api_site": {
    "dyttzy": {
      "api": "http://xxx.com/api.php/provide/vod",
      "name": "示例资源",
      "detail": "http://xxx.com"
    }
  },
  "custom_category": [{ "name": "华语", "type": "movie", "query": "华语" }]
}
```

| 字段              | 说明                      |
| ----------------- | ------------------------- |
| `cache_time`      | 接口缓存时间(秒)          |
| `api_site`        | 资源站列表,key 为唯一标识 |
| `custom_category` | 首页自定义分类            |

## 📺 TVBox 配置

桌面/服务器端启动后,内置的 Axum 服务在 `:3000` 暴露 TVBox 标准接口:

```text
http://<host>:3000/api/tvbox
http://<host>:3000/api/tvbox?subscriptionUrl=https://example.com/sub.json
```

- 订阅源优先级: `subscriptionUrl` 参数 > `PARSES_FILE` > `PARSES_URL`
- `forceSpiderRefresh=1` 强制刷新 Spider JAR

## 🐳 Docker 部署(可选,仅 TVBox API)

如果你只想跑后端 API 给 TVBox 用,可以用 Docker:

```bash
docker compose up -d
```

`docker-compose.yml` 示例:

```yaml
services:
  api:
    image: ghcr.io/geon97/quantumtv:latest
    container_name: quantumtv-api
    ports: ['3086:3000']
    environment:
      - RUST_LOG=info
      - PARSES_URL=http://127.0.0.1
      - PARSES_FILE=/data/data.json
      - SERVER_IP=127.0.0.1
      - BIND_ADDR=0.0.0.0
    volumes:
      - ./crates/api-server/data.json:/data/data.json:ro
    restart: unless-stopped
```

## 🛠 技术栈

| 分类     | 主要依赖                                                                                                               |
| -------- | ---------------------------------------------------------------------------------------------------------------------- |
| 桌面壳   | [Tauri 2](https://tauri.app/)                                                                                          |
| 后端语言 | [Rust 1.90](https://www.rust-lang.org/) + [Axum](https://github.com/tokio-rs/axum) + [SQLite](https://www.sqlite.org/) |
| 前端框架 | [Next.js 16](https://nextjs.org/) + [React 19](https://react.dev/) + [TypeScript 5](https://www.typescriptlang.org/)   |
| 样式     | [Tailwind CSS 4](https://tailwindcss.com/)                                                                             |
| 播放器   | [Plyr](https://github.com/sampotts/plyr) + [HLS.js](https://github.com/video-dev/hls.js)                               |

## 🏛️ 架构一览

```
┌─────────────────────────────────────────────────────┐
│  Next.js 16 (React 19, TS)   ← UI / 状态 / 路由     │
├─────────────────────────────────────────────────────┤
│  Tauri 2 IPC                ← 类型安全的命令桥       │
├─────────────────────────────────────────────────────┤
│  Rust Core (quantumtv-core) ← 搜索聚合 / 配置解析    │
│  Rust Commands              ← 代理 / 缓存 / 推荐     │
│  SQLite                     ← 本地持久化            │
├─────────────────────────────────────────────────────┤
│  Axum API Server (可选)     ← TVBox 端点 / 远程订阅  │
└─────────────────────────────────────────────────────┘
```

桌面端和 Android 共享同一份前端,Rust 核心 crate (`quantumtv-core`) 同时被 Tauri 命令和独立的 Axum API server 复用,无冗余。

## 🔒 安全与隐私

- **个人使用**: 请仅在个人或家庭网络中部署,**不要**在公共计算机或共享服务器上运行。
- **数据归属**: 项目不收集任何用户数据。播放记录、收藏、配置只存在你本机的 SQLite 文件里。
- **网络出口**: 默认只访问你自己配置的资源站、豆瓣公开 API、以及 GitHub 检查更新。
- **法律责任**: 用户因自行配置第三方资源所产生的法律风险由用户本人承担。

## ⚠️ 免责声明

<details>
<summary><strong>展开查看完整声明</strong></summary>

- 项目部署后为**空壳**,**无内置播放源**,所有内容资源需用户自行收集与配置。
- 请**不要**在 B 站、小红书、微信公众号、抖音、今日头条或其他中国大陆社交平台发布视频或文章宣传本项目,**不授权**任何"科技周刊/月刊"类项目或站点收录本项目。
- 本项目主要用于 **Rust / Tauri / 现代 Web 技术**的学习、研究与技术实践,**不以内容分发或商业使用为目的**。
- 用户因自行配置第三方资源、使用或传播本项目所产生的任何法律风险与后果,**均由用户本人承担**,项目作者及贡献者不承担任何责任。
- 本项目**不面向中国大陆地区**提供任何形式的服务、运营或内容分发。任何在该地区的使用行为均属于用户个人行为,与本项目及其开发者无关。

</details>

## 🤝 贡献

欢迎 Issue 与 PR。提交 PR 前请确保:

```bash
npm run lint && npm run typecheck && npm test
cd src-tauri && cargo test
```

## 📄 License

[MIT](LICENSE) © 2025 QuantumTV & Contributors

## 🙏 致谢

- [DecoTV](https://github.com/Decohererk/DecoTV/releases/tag/v1.1.0) —— 本项目的二次开发基础
- [Plyr](https://github.com/sampotts/plyr) · [HLS.js](https://github.com/video-dev/hls.js) —— 核心播放器
- [Zwei](https://github.com/bestzwei) · [CMLiussss](https://github.com/cmliu) —— 豆瓣数据服务
- 所有为学习与研究目的提供公开数据接口的站点与社区

## 📈 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Geon97/QuantumTV&type=date&legend=top-left)](https://www.star-history.com/#Geon97/QuantumTV&type=date&legend=top-left)

---

<div align="center">
  <p><strong>🌟 如果对你有帮助,点个 Star 让我知道 🌟</strong></p>
  <p><sub>Made with ❤️ by <a href="https://github.com/Geon97">Geon97</a> and <a href="https://github.com/Geon97/QuantumTV/graphs/contributors">Contributors</a></sub></p>
</div>
