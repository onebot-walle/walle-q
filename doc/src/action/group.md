# 单极群组动作

## 获取群信息 get_group_info

动作请求:

| 字段       | 类型   | 说明  |
| ---------- | ------ | ----- |
| `group_id` | String | 群 ID |

动作响应：

| 字段名       | 数据类型 | 说明   |
| ------------ | -------- | ------ |
| `group_id`   | String   | 群 ID  |
| `group_name` | String   | 群名称 |

## 获取群列表 get_group_list

无动作请求数据

群信息列表，每一个元素的字段同 `get_group_info` 的响应数据。

## 获取群成员信息 get_group_member_info

动作请求:

| 字段       | 类型   | 说明    |
| ---------- | ------ | ------- |
| `group_id` | String | 群 ID   |
| `user_id`  | String | 用户 ID |

动作响应：

| 字段名             | 数据类型 | 说明          |
| ------------------ | -------- | ------------- |
| `user_id`          | String   | 用户 ID       |
| `user_name`        | String   | 用户名称/昵称 |
| `user_remark`      | String   | 备注名称      |
| `user_displayname` | String   | 群头衔        |

## 获取群成员列表 get_group_member_list

动作请求:

| 字段       | 类型   | 说明  |
| ---------- | ------ | ----- |
| `group_id` | String | 群 ID |

群信息列表，每一个元素的字段同 `get_group_member_info` 的响应数据。

## 设置群名称 set_group_name

动作请求:

| 字段         | 类型   | 说明   |
| ------------ | ------ | ------ |
| `group_id`   | String | 群 ID  |
| `group_name` | String | 群名称 |

无动作响应数据

## 退出群 leave_group

动作请求:

| 字段       | 类型   | 说明  |
| ---------- | ------ | ----- |
| `group_id` | String | 群 ID |

无动作响应数据

## 踢出群成员 kick_group_member

动作请求:

| 字段       | 类型   | 说明    |
| ---------- | ------ | ------- |
| `group_id` | String | 群 ID   |
| `user_id`  | String | 用户 ID |

无动作响应数据

## 禁言群成员 ban_group_member

动作请求:

| 字段       | 类型   | 说明           |
| ---------- | ------ | -------------- |
| `group_id` | String | 群 ID          |
| `user_id`  | String | 用户 ID        |
| `duration` | i64    | 时长，单位：秒 |

无动作响应数据

## 解禁群成员 unban_group_member

动作请求:

| 字段       | 类型   | 说明    |
| ---------- | ------ | ------- |
| `group_id` | String | 群 ID   |
| `user_id`  | String | 用户 ID |

无动作响应数据

## 设置群管理员 set_group_admin

动作请求:

| 字段       | 类型   | 说明    |
| ---------- | ------ | ------- |
| `group_id` | String | 群 ID   |
| `user_id`  | String | 用户 ID |

无动作响应数据

## 取消群管理员 unset_group_admin

动作请求:

| 字段       | 类型   | 说明    |
| ---------- | ------ | ------- |
| `group_id` | String | 群 ID   |
| `user_id`  | String | 用户 ID |

无动作响应数据

## * 处理加群请求 set_join_group

动作请求：

| 字段         | 类型   | 说明                   |
| ------------ | ------ | ---------------------- |
| `request_id` | i64    | 请求 ID                |
| `user_id`    | String | 用户 ID                |
| `group_id`   | String | 群 ID                  |
| `accept`     | bool   | 是否接受               |
| `block`      | bool   | 可选，是否禁止再次申请 |
| `message`    | String | 可选，拒绝理由         |

无动作响应数据

## * 获取加群申请 get_join_group_requests

无动作请求数据

动作响应为 `Vec<event.request.join_group>`

## * 处理群邀请 set_group_invited

动作请求：

| 字段         | 类型   | 说明     |
| ------------ | ------ | -------- |
| `request_id` | i64    | 请求 ID  |
| `group_id`   | String | 群 ID    |
| `accept`     | bool   | 是否接受 |

无动作响应数据

## * 获取群邀请 get_group_inviteds

无动作请求数据

动作响应为 `Vec<event.request.group_invited>`