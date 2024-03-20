# Monster Hunter: World - More Auto Shoutouts Plugin

MHW 更多定型文mod

# 编译

1. 拉取工具库 [https://github.com/eigeen/mhw-toolkit](https://github.com/eigeen/mhw-toolkit)
2. 在同一目录下，拉取本仓库内容
3. 拉取MHW前置修改版 [https://github.com/eigeen/MHW-QuestLoader](https://github.com/eigeen/MHW-QuestLoader)，编译并获取 `loader.lib` 和 `LoaderFFI.lib` 两个静态库
4. 将静态库放在 `more-auto-shoutouts/lib` 目录内
5. 确保当前工作目录包含 `mhw-toolkit` 和 `more-auto-shoutouts`
6. 运行 `cargo build --release`

# 配置文件

配置文件在使用时放置于`<游戏根目录>/nativePC/plugins/mas-config.toml`

## 结构

如果能够阅读源码，推荐阅读 [configs.rs](src/configs.rs)

或参考 [示例文件](mas-config.example.toml)，在此基础上修改
