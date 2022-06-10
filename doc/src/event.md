# 事件 Event

所有事件共有字段

| 字段     | 类型   | 说明                 |
| -------- | ------ | -------------------- |
| id       | String | 事件 ID              |
| impl     | String | 实现名，即 `Walle-Q` |
| platform | String | 平台名，即 `qq`      |
| self_id  | String | Bot ID               |
| time     | f64    | 事件戳，单位：秒     |

## 消息事件 message

### [标准]单用户消息事件 message.private

| 字段        | 类型    | 说明            |
| ----------- | ------- | --------------- |
| message_id  | String  | 消息 ID         |
| message     | Message | 消息对象        |
| alt_message | String  | 消息文本        |
| user_id     | String  | 发送者 ID       |
| user_name   | String  | 发送者 nickname |

### [扩展]群临时消息 message.private.group_temp

| 字段        | 类型    | 说明            |
| ----------- | ------- | --------------- |
| message_id  | String  | 消息 ID         |
| message     | Message | 消息对象        |
| alt_message | String  | 消息文本        |
| user_id     | String  | 发送者 ID       |
| group_id    | String  | 群 ID           |
| user_name   | String  | 发送者 nickname |

### [标准]群用户消息事件 message.group

| 字段         | 类型    | 说明       |
| ------------ | ------- | ---------- |
| message_id   | String  | 消息 ID    |
| message      | Message | 消息对象   |
| alt_message  | String  | 消息文本   |
| user_id      | String  | 发送者 ID  |
| group_id     | String  | 群 ID      |
| _user_card_  | String  | 发送者名片 |
| _group_name_ | String  | 群名称     |

## 通知事件 notice

### [标准]好友消息撤回 notice.private_message_delete

| 字段       | 类型   | 说明      |
| ---------- | ------ | --------- |
| message_id | String | 消息 ID   |
| user_id    | String | 发送者 ID |

### [标准]好友增加 notice.friend_increase

| 字段        | 类型   | 说明     |
| ----------- | ------ | -------- |
| user_id     | String | 好友 ID  |
| _user_name_ | String | 好友昵称 |

### [标准]好友减少 notice.friend_decrease

| 字段    | 类型   | 说明    |
| ------- | ------ | ------- |
| user_id | String | 好友 ID |

### [扩展]好友戳一戳 notice.friend_poke

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| user_id     | String | 发送者 ID |
| receiver_id | String | 接收者 ID |

### [标准]群成员增加 notice.group_member_increase

| 字段        | 类型   | 说明             |
| ----------- | ------ | ---------------- |
| sub_type    | String | 加群类型         |
| user_id     | String | 成员 ID          |
| group_id    | String | 群 ID            |
| operator_id | String | 操作者 ID (暂缺) |

### [标准]群成员减少 notice.group_member_decrease

| 字段        | 类型   | 说明                              |
| ----------- | ------ | --------------------------------- |
| sub_type    | String | 退群类型 leave \| kick \| disband |
| user_id     | String | 成员 ID                           |
| group_id    | String | 群 ID                             |
| operator_id | String | 操作者 ID                         |

### [标准]群成员禁言 notice.group_member_ban

| 字段        | 类型   | 说明               |
| ----------- | ------ | ------------------ |
| user_id     | String | 成员 ID            |
| group_id    | String | 群 ID              |
| operator_id | String | 操作者 ID          |
| _duration_  | i64    | 禁言时长，单位：秒 |

### [标准]群消息撤回 notice.group_message_delete

| 字段        | 类型   | 说明                      |
| ----------- | ------ | ------------------------- |
| sub_type    | String | 撤回类型 recall 或 delete |
| message_id  | String | 消息 ID                   |
| user_id     | String | 成员 ID                   |
| group_id    | String | 群 ID                     |
| operator_id | String | 操作者 ID                 |

### [标准]群管理员设置 notice.group_admin_set

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| user_id     | String | 成员 ID   |
| group_id    | String | 群 ID     |
| operator_id | String | 操作者 ID |

### [标准]群管理员取消 notice.group_admin_unset

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| user_id     | String | 成员 ID   |
| group_id    | String | 群 ID     |
| operator_id | String | 操作者 ID |

### [扩展]群名称更新 notice.group_name_update

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| group_id    | String | 群 ID     |
| group_name  | String | 群名称    |
| operator_id | String | 操作者 ID |

## 请求事件 request

### [扩展]好友添加请求 request.new_friend

| 字段       | 类型   | 说明          |
| ---------- | ------ | ------------- |
| request_id | i64    | 请求 ID       |
| user_id    | String | 用户 ID       |
| user_name  | String | 用户名称/昵称 |
| message    | String | 请求信息      |

### [扩展]新成员加群申请 request.join_group

| 字段         | 类型             | 说明          |
| ------------ | ---------------- | ------------- |
| request_id   | i64              | 请求 ID       |
| user_id      | String           | 用户 ID       |
| user_name    | String           | 用户名称/昵称 |
| group_id     | String           | 群 ID         |
| group_name   | String           | 群名称        |
| message      | String           | 请求信息      |
| suspicious   | bool             | 是否可疑      |
| invitor_id   | Option\<String\> | 邀请人 ID     |
| invitor_name | Option\<String\> | 邀请人名称    |

### [扩展]群邀请 request.group_invited

| 字段         | 类型             | 说明       |
| ------------ | ---------------- | ---------- |
| request_id   | i64              | 请求 ID    |
| group_id     | String           | 群 ID      |
| group_name   | String           | 群名称     |
| invitor_id   | Option\<String\> | 邀请人 ID  |
| invitor_name | Option\<String\> | 邀请人名称 |
