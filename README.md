<p align="center">
  <img src="./resources/icon.png" width="112" alt="Neko" />
</p>

<h1 align="center">Neko</h1>

<p align="center"><strong>中文</strong> · <a href="./README.en.md">English</a></p>

<p align="center"><strong>优雅的桌面休息提醒</strong> — 适用于 macOS、Windows 和 Linux。</p>

<p align="center">定时休息、智能重置与专注提醒；设置和数据均保存在本地。</p>

## 截图

### 休息与今日状态

休息节奏、稍后提醒选项，以及当天完成情况一目了然。

<p align="center">
  <img src="./docs/screenshot/1.png" width="880" alt="Neko 休息设置与今日状态" />
</p>

### 外观与休息预览

在设置中调整外观，并即时查看休息弹窗效果。

<p align="center">
  <img src="./docs/screenshot/2.png" width="880" alt="Neko 外观设置与休息预览" />
</p>

## 下载安装

从 [夸克网盘下载 Neko 安装包](https://pan.quark.cn/s/223657edd23b)，按你的系统选择对应文件。

| 平台    | 安装方式                                         |
| ------- | ------------------------------------------------ |
| macOS   | 打开对应芯片的 DMG，将 **Neko** 拖入「应用程序」 |
| Windows | 运行安装程序，随后从开始菜单或桌面快捷方式启动   |
| Linux   | AppImage 添加执行权限后运行，或安装 `.deb` 包    |

### macOS 无法打开时

当前安装包尚未完成 Apple 开发者签名和公证。若提示「已损坏，无法打开」，在终端执行：

```bash
xattr -cr /Applications/Neko.app
open /Applications/Neko.app
```

若提示「无法验证开发者」，可右键应用后选择「打开」，或在「系统设置 → 隐私与安全性」中选择「仍要打开」。启动后请在菜单栏右侧查找 Neko 图标；它不会显示在 Dock 中。

### Windows 与 Linux 提示

Windows 的未签名安装包可能出现 SmartScreen，选择「更多信息 → 仍要运行」即可。Linux 的 AppImage 需要先执行 `chmod +x Neko-*.AppImage`；`.deb` 包可使用 `sudo dpkg -i neko_*_amd64.deb` 安装。

## 功能

| 功能                  | 说明                                         |
| --------------------- | -------------------------------------------- |
| **可配置的休息节奏**  | 自定义休息频率、时长和提醒方式               |
| **休息提示**          | 消息卡片弹窗与系统通知配合提醒               |
| **智能重置**          | 根据工作时间、空闲和锁屏状态调整计时         |
| **菜单栏 / 系统托盘** | 从顶部菜单栏或系统托盘快速查看状态、打开设置 |
| **个性化体验**        | 音效、外观、登录自启和更新检查均可设置       |
| **多语言界面**        | 中文、English、日本語，默认跟随系统          |
| **本地数据**          | 偏好和设置保存在本机，不经过第三方服务       |

## 项目结构

```
src/
  pages/      # 设置 / 休息页面
  components/ # 可复用 React 组件
  shared/     # 共享类型、i18n、纯逻辑
  lib/        # Tauri 适配器与前端服务
src-tauri/
  src/        # 命令、配置、调度、监测与平台适配
  resources/  # 内置 WAV 音效
```

## 许可

[PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0)

Required Notice: Copyright (c) 2026 MultCat Authors

详见仓库根目录 [`LICENSE`](./LICENSE)。
