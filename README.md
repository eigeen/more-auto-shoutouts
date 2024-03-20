# Monster Hunter: World - More Auto Shoutouts plugin

MHW 更多定型文mod

# 编译

1. 拉取 [https://github.com/eigeen/mhw-toolkit](https://github.com/eigeen/mhw-toolkit)
2. 在同一目录下，拉取本仓库内容
3. 确保当前工作目录包含`mhw-toolkit`和`more-auto-shoutouts`
4. 运行`cargo build --release`

# 配置文件

配置文件在使用时放置于`<游戏根目录>/nativePC/plugins/mas-config.toml`

## 结构

如果能够阅读源码，推荐阅读 [configs.rs](blob/main/src/configs.rs)

或参考 [示例文件](blob/main/mas-config.example.toml)，在此基础上修改
