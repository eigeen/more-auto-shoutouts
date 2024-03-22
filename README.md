# Monster Hunter: World - More Auto Shoutouts Plugin

MHW 更多定型文mod

# 目录

- [常见问题](https://git.eigeen.com/eigeen/more-auto-shoutouts-docs)
- [支持功能](#支持功能)
- [配置文件](#配置文件)
- [编译](#编译)

# 支持功能

某些动作触发频率过高但又想设置，可以等后续更新冷却时间功能。

*盾斧的部分动作出现复用，可能导致误判断。后续更新会有解决方案，例如延迟触发等。*

[更新日志](CHANGELOG.md)

## 当前支持

### 通用配置

- 动作
- 任务状态
- 使用道具/衣装

> 通用配置与武器无关，所有武器均支持检测动作。
> 
> 某些武器有特殊状态，例如太刀、斩斧等，才会支持专有触发器。例如片手是否红jr等都可以通过动作判断。

### 武器专有

- 太刀：开刃等级
- 虫棍：红白黄三灯时间
- 盾斧：
    - 红盾时间
    - 红剑时间
    - 电锯时间
    - 瓶子数量
    - 剑能量（瓶子外框）

## 计划功能

- 双刀鬼人状态
- 斩斧充能
- 延迟触发
- 冷却时间（已实现全局，单trigger待定）
- BUFF获取

# 配置文件

配置文件在使用时放置于 `<游戏根目录>/nativePC/plugins/mas-config.toml`

## 结构

如果能够阅读源码，推荐阅读 [configs.rs](src/configs.rs)

或参考 [示例文件](mas-config.example.toml)，在此基础上修改

# 编译

1. 拉取工具库 [https://github.com/eigeen/mhw-toolkit](https://github.com/eigeen/mhw-toolkit)
2. 在同一目录下，拉取本仓库内容
3. 拉取MHW前置（本人fork版本内含一个FFI静态库，用于该插件发送日志） [https://github.com/eigeen/MHW-QuestLoader](https://github.com/eigeen/MHW-QuestLoader)，编译并获取 `loader.lib` 和 `LoaderFFI.lib` 两个静态库
4. 将静态库放在 `more-auto-shoutouts/lib` 目录内
5. 确保当前工作目录包含 `mhw-toolkit` 和 `more-auto-shoutouts`
6. 在 `more-auto-shoutouts` 目录内运行 `cargo build --release --features use_logger`

如果你不需要log功能，则可以忽略 3-4 步，并使用 `cargo build --release` 编译。