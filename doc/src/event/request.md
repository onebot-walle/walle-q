# 请求事件 request

## * 好友添加请求 request.new_friend

| 字段         | 类型   | 说明          |
| ------------ | ------ | ------------- |
| `request_id` | i64    | 请求 ID       |
| `user_id`    | String | 用户 ID       |
| `user_name`  | String | 用户名称/昵称 |
| `message`    | String | 请求信息      |

## * 新成员加群申请 request.join_group

| 字段           | 类型             | 说明          |
| -------------- | ---------------- | ------------- |
| `request_id`   | i64              | 请求 ID       |
| `user_id`      | String           | 用户 ID       |
| `user_name`    | String           | 用户名称/昵称 |
| `group_id`     | String           | 群 ID         |
| `group_name`   | String           | 群名称        |
| `message`      | String           | 请求信息      |
| `suspicious`   | bool             | 是否可疑      |
| `invitor_id`   | Option\<String\> | 邀请人 ID     |
| `invitor_name` | Option\<String\> | 邀请人名称    |

## * 群邀请 request.group_invited

| 字段           | 类型             | 说明       |
| -------------- | ---------------- | ---------- |
| `request_id`   | i64              | 请求 ID    |
| `group_id`     | String           | 群 ID      |
| `group_name`   | String           | 群名称     |
| `invitor_id`   | Option\<String\> | 邀请人 ID  |
| `invitor_name` | Option\<String\> | 邀请人名称 |
