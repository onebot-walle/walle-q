# 事件 Event

> \* 标记表示扩展字段或事件

所有事件共有字段

| 字段          | 类型   | 说明                               |
| ------------- | ------ | ---------------------------------- |
| `id`          | String | 事件 ID                            |
| `time`        | f64    | 事件戳，单位：秒                   |
| `type`        | String | `message` \| `notice` \| `request` |
| `detail_type` | String |                                    |
| `sub_type`    | String |                                    |
| `self`        | Self   | Self 标识                          |

