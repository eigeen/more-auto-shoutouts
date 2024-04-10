# 更新日志

## 0.2.0

**特性：**

- 支持单个动作内的多个伤害数据统计
- 所有内部实现全部采用async/await异步
- 单个trigger的执行转为创建异步任务，支持内部长时间阻塞
- 由于主动式伤害收集模式的实现，现在可以检查伤害为0的情况

**Break Changes:**

- 原配置文件 `trigger_on.damage` 条件取消，由 `check.damage` 替代
- 具体参考新配置文件[样例](mas-config.example.toml)

**旧配置：**

```toml
# 旧配置
[[trigger]]
action_mode = "sequential_all"

    [trigger.trigger_on.damage]
    value = { gt = 200 }

    [[trigger.check]]
    weapon_type.value = 0

    [[trigger.check]]
    fsm.value = { target = 3, id = 137 }

    [[trigger.action]]
    cmd = "SendChatMessage"
    param = "强击真三蓄！造成了{{damage}}伤害"
```

**新配置：**

```toml
# 新配置
[[trigger]]
action_mode = "sequential_all"
name = "大剑强击真三蓄"

    [trigger.trigger_on.fsm]
    new = { target = 3, id = 137 }

    [[trigger.check]]
    weapon_type.value = 0

    [[trigger.check]]
    [trigger.check.damage]
    damage = { gt = 0 }
    fsm = { target = 3, id = 137 }
    timeout = 1000

    [[trigger.action]]
    cmd = "SendChatMessage"
    param = "强击真三蓄！造成了{{damage}}伤害"

# 新配置
[[trigger]]
action_mode = "sequential_all"
name = "太刀登龙成功"

    [trigger.trigger_on.fsm]
    new = { target = 3, id = 92 }

    [[trigger.check]]
    weapon_type.value = 3

    # 伤害收集&检查条件
    # 当你在Action中需要使用{{damage}}时，
    # 即使不需要判断伤害，也必须要使用trigger.check.damage
    # 否则上下文获取不到伤害值，无法正常打印伤害
    [[trigger.check]]
    [trigger.check.damage]
    damage = { gt = 0 }
    fsm = { target = 3, id = 92 }
    timeout = 2000

    [[trigger.action]]
    cmd = "SendChatMessage"
    param = "*太刀登龙造成伤害{{damage}}"
```

## 0.1.6

修复游戏内命令使用后失效的问题

## 0.1.5

**特性：**

实现伤害 Trigger

引入钩子，需要在编译时提供条件`hooks`

**重构：**

1. 事件转发系统重构，支持定向转发和广播（旧系统只支持广播）
2. logger 实现优化，打印日志等级和模块名
3. Context 上下文共享模式更改，使用智能指针共享

## 0.1.4

修复 in 和 nin 比较模式总是生效的问题。

回到据点和集会时，自动重置 sequential_one 的计数器。

新增插件临时启用和禁用的开关命令 `!mas enable` 和 `!mas disable`。每次启动游戏默认启用。

## 0.1.3

支持自定义单个触发器的冷却时间。

修复sequence_one模式下，最后一个行为结束后数组越界错误。

优化debug提示。

## 0.1.2

减少debug日志长度。

## 0.1.1

首次支持游戏内命令功能。

支持热重载配置 通过游戏内输入框输入 `!mas reload` 进行重载。

新触发器：使用特定道具（包括衣装）。

日志可以打印当前Fsm状态，通过修改 `loader-config.json`，将日志等级设为 `DEBUG` 即可启用。

## 0.1.0

支持基本功能。

- 通用
  - 动作
  - 任务状态
- 武器专有
  - 太刀：开刃等级
  - 虫棍：红白黄三灯时间
  - 盾斧：
      - 红盾时间
      - 红剑时间
      - 电锯时间
      - 瓶子数量
      - 剑能量（瓶子外框）
