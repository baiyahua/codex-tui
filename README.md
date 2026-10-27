# Rust Wayland 长截图 MVP

这个项目不会调用 GNOME Screenshot。它使用：

- XDG ScreenCast Portal：显示 Wayland 强制要求的窗口授权界面
- PipeWire：获得被授权窗口的连续画面
- GStreamer `pipewiresrc`：把 PipeWire buffer 转换为 RGBA 字节流
- Rust：选择帧、检测重叠、拼接和编码 PNG

## 系统依赖

Ubuntu 上需要运行时工具和插件：

```bash
sudo apt install pipewire gstreamer1.0-tools gstreamer1.0-pipewire \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good
```

本项目通过 `gst-launch-1.0` 使用运行时插件，因此不需要
`libpipewire-0.3-dev` 或 GStreamer 开发头文件。# codex-tui
