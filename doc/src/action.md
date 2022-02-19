# 动作 Action

## 获取近期事件 get_latest_events

动作请求：

| 字段    | 类型 | 说明                                             |
| ------- | ---- | ------------------------------------------------ |
| limit   | i64  | 获取的事件数量上限，0 使用默认值 10              |
| timeout | i64  | 没有事件时要等待的秒数，0 表示使用短轮询，不等待 |

动作响应为 Vec&lt;Event&gt; 

## 获取支持的动作列表 get_supported_actions

无动作请求数据

动作响应为 Vec&lt;String&gt; 

## 获取运行状态 get_status

无动作请求数据

动作响应：

| 字段   | 类型 | 说明                                            |
| ------ | ---- | ----------------------------------------------- |
| good   | bool | 是否各项状态都符合预期，OneBot 实现各模块均正常 |
| online | bool | 是否在线                                        |

## 获取版本信息 get_version

无动作请求数据

动作响应：

| 字段           | 类型   | 说明           |
| -------------- | ------ | -------------- |
| impl           | String | `Walle-Q`      |
| platform       | String | `qq`           |
| version        | String | Walle-Q 版本号 |
| onebot_version | String | `12`           |

## 发送消息 send_message

动作请求：

| 字段        | 类型                 | 说明                   |
| ----------- | -------------------- | ---------------------- |
| detail_type | String               | `private` 或者 `group` |
| group_id    | Option&lt;String&gt; | 群 ID                  |
| user_id     | Option&lt;String&gt; | 用户 ID                |
| message     | Message              | 消息内容               |

动作响应：

| 字段       | 类型   | 说明             |
| ---------- | ------ | ---------------- |
| message_id | String | 消息 ID          |
| time       | f64    | 时间戳，单位：秒 |

## 删除消息 delete_message

动作请求：

| 字段       | 类型   | 说明    |
| ---------- | ------ | ------- |
| message_id | String | 消息 ID |

无动作响应数据

## 获取机器人自身信息 get_self_info

无动作请求数据

动作响应：

| 字段     | 类型   | 说明            |
| -------- | ------ | --------------- |
| user_id  | String | 机器人用户 ID   |
| nickname | String | 机器人名称/昵称 |

## 获取用户信息 get_user_info

动作请求:

| 字段    | 类型   | 说明    |
| ------- | ------ | ------- |
| user_id | String | 用户 ID |

动作响应：

| 字段     | 类型   | 说明      |
| -------- | ------ | --------- |
| user_id  | String | 用户 ID   |
| nickname | String | 名称/昵称 |

## 获取好友列表 get_friend_list

无动作请求数据

好友信息列表，每一个元素的字段同 `get_user_info` 的响应数据。

## 获取群信息 get_group_info

动作请求:

| 字段     | 类型   | 说明  |
| -------- | ------ | ----- |
| group_id | string | 群 ID |

动作响应：

| 字段名     | 数据类型 | 说明   |
| ---------- | -------- | ------ |
| group_id   | string   | 群 ID  |
| group_name | string   | 群名称 |

## 获取群列表 get_group_list

无动作请求数据

群信息列表，每一个元素的字段同 `get_group_info` 的响应数据。

## 获取群成员列表 get_group_member_info

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

动作响应：

| 字段名   | 数据类型 | 说明          |
| -------- | -------- | ------------- |
| user_id  | string   | 用户 ID       |
| nickname | string   | 用户名称/昵称 |

## 获取群成员列表 get_group_member_list

动作请求:

| 字段     | 类型   | 说明  |
| -------- | ------ | ----- |
| group_id | string | 群 ID |

群信息列表，每一个元素的字段同 `get_group_member_info` 的响应数据。

## 设置群名称 set_group_name

动作请求:

| 字段       | 类型   | 说明   |
| ---------- | ------ | ------ |
| group_id   | string | 群 ID  |
| group_name | string | 群名称 |

无动作响应数据

## 退出群 leave_group

动作请求:

| 字段     | 类型   | 说明  |
| -------- | ------ | ----- |
| group_id | string | 群 ID |

无动作响应数据

## 踢出群成员 kick_group_member

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据

## 禁言群成员 ban_group_member

动作请求:

| 字段     | 类型   | 说明           |
| -------- | ------ | -------------- |
| group_id | string | 群 ID          |
| user_id  | string | 用户 ID        |
| duration | i64    | 时长，单位：秒 |

无动作响应数据

## 解禁群成员 unban_group_member

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据

## 设置群管理员 set_group_admin

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据

## 取消群管理员 unset_group_admin

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据