# QuantPilot v1.0.2 发布包

## 便携版
解压后双击 `QuantPilot.exe` 即可运行。
要求: Windows 10+ (含 WebView2)。

## 安装版 (.msi)
MSI 打包需要 WiX Toolset，网络环境就绪后执行:
```
cd src-tauri
cargo tauri build
```
输出: `target/release/bundle/msi/QuantPilot_1.0.2_x64_zh-CN.msi`

## 开发版
```
.\start.bat
```

