# VideoCaptor

视频转 GIF 工具，支持多档位预设、动态参数调节、实时进度反馈。

我是做游戏开发的，经常需要给策划看一些效果表现，为了方便自己写了这个工具，顺便开源了。

## 功能

- 拖拽或选择视频文件
- 自定义时间范围裁剪
- 快速一键导出 GIF
- 最快只需 3 步操作

## 常见问题

### "已损坏，无法打开"

将应用拖入 `/Applications` 后若提示：

> "VideoCaptor"已损坏，无法打开。你应该将它移到废纸篓。

这是 macOS Gatekeeper 安全机制导致的，应用本身没有问题。

打开**终端**，执行以下命令后回车，输入密码即可：

```bash
sudo xattr -rd com.apple.quarantine /Applications/VideoCaptor.app
```

之后即可正常打开应用。

## 技术架构

详见 [docs/tech.md](./docs/tech.md)。

## 开源协议

Apache License 2.0
