# 单用户动作

## 获取机器人自身信息 get_self_info

无动作请求数据

动作响应：

| 字段               | 类型   | 说明            |
| ------------------ | ------ | --------------- |
| `user_id`          | String | 机器人用户 ID   |
| `user_name`        | String | 机器人名称/昵称 |
| `user_displayname` | String | 显示名称        |

## 获取用户信息 get_user_info

动作请求:

| 字段      | 类型   | 说明    |
| --------- | ------ | ------- |
| `user_id` | String | 用户 ID |

动作响应：

| 字段               | 类型   | 说明      |
| ------------------ | ------ | --------- |
| `user_id`          | String | 用户 ID   |
| `user_name`        | String | 名称/昵称 |
| `user_remark`      | String | 备注名称  |
| `user_displayname` | String | 显示名称  |

## 获取好友列表 get_friend_list

无动作请求数据

好友信息列表，每一个元素的字段同 `get_user_info` 的响应数据。

## * 处理好友请求 set_new_friend

动作请求：

| 字段         | 类型   | 说明         |
| ------------ | ------ | ------------ |
| `user_id`    | String | 用户 ID      |
| `request_id` | i64    | 请求 ID      |
| `accept`     | bool   | 是否接受请求 |

无动作响应数据


## * 删除好友 delete_friend

| 字段      | 类型   | 说明    |
| --------- | ------ | ------- |
| `user_id` | String | 用户 ID |

> **注意**：如果目标用户不是好友，不会返回错误信息


无动作响应数据

## 获取好友请求列表 get_new_friend_requests

无动作请求数据

动作响应为 `Vec<event.request.new_friend>`