# 事件 Event

> \* 标记表示扩展字段或事件

所有事件共有字段

| 字段        | 类型   | 说明                         |
| ----------- | ------ | ---------------------------- |
| id          | String | 事件 ID                      |
| time        | f64    | 事件戳，单位：秒             |
| type        | String | message \| notice \| request |
| detail_type | String |                              |
| sub_type    | String |                              |

## 消息事件 message

| 字段 | 类型 | 说明      |
| ---- | ---- | --------- |
| self | Self | Self 标识 |

### 单用户消息事件 message.private

| 字段        | 类型    | 说明            |
| ----------- | ------- | --------------- |
| message_id  | String  | 消息 ID         |
| message     | Message | 消息对象        |
| alt_message | String  | 消息文本        |
| user_id     | String  | 发送者 ID       |
| * user_name | String  | 发送者 nickname |

### * 群临时消息 message.group_temp

| 字段        | 类型    | 说明            |
| ----------- | ------- | --------------- |
| message_id  | String  | 消息 ID         |
| message     | Message | 消息对象        |
| alt_message | String  | 消息文本        |
| user_id     | String  | 发送者 ID       |
| group_id    | String  | 群 ID           |
| * user_name | String  | 发送者 nickname |

### 群用户消息事件 message.group

| 字段         | 类型    | 说明       |
| ------------ | ------- | ---------- |
| message_id   | String  | 消息 ID    |
| message      | Message | 消息对象   |
| alt_message  | String  | 消息文本   |
| user_id      | String  | 发送者 ID  |
| group_id     | String  | 群 ID      |
| * user_card  | String  | 发送者名片 |
| * group_name | String  | 群名称     |

## 通知事件 notice

### 好友消息撤回 notice.private_message_delete

| 字段       | 类型   | 说明      |
| ---------- | ------ | --------- |
| message_id | String | 消息 ID   |
| user_id    | String | 发送者 ID |

### 好友增加 notice.friend_increase

| 字段        | 类型   | 说明     |
| ----------- | ------ | -------- |
| user_id     | String | 好友 ID  |
| * user_name | String | 好友昵称 |

### 好友减少 notice.friend_decrease

| 字段    | 类型   | 说明    |
| ------- | ------ | ------- |
| user_id | String | 好友 ID |

### * 好友戳一戳 notice.friend_poke

> 此事件为扩展事件

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| user_id     | String | 发送者 ID |
| receiver_id | String | 接收者 ID |

### 群成员增加 notice.group_member_increase

| 字段        | 类型   | 说明             |
| ----------- | ------ | ---------------- |
| sub_type    | String | 加群类型         |
| user_id     | String | 成员 ID          |
| group_id    | String | 群 ID            |
| operator_id | String | 操作者 ID (暂缺) |

### 群成员减少 notice.group_member_decrease

| 字段        | 类型   | 说明                              |
| ----------- | ------ | --------------------------------- |
| sub_type    | String | 退群类型 leave \| kick \| disband |
| user_id     | String | 成员 ID                           |
| group_id    | String | 群 ID                             |
| operator_id | String | 操作者 ID                         |

### 群成员禁言 notice.group_member_ban

| 字段        | 类型   | 说明               |
| ----------- | ------ | ------------------ |
| user_id     | String | 成员 ID            |
| group_id    | String | 群 ID              |
| operator_id | String | 操作者 ID          |
| * duration  | i64    | 禁言时长，单位：秒 |

### 群消息撤回 notice.group_message_delete

| 字段        | 类型   | 说明                      |
| ----------- | ------ | ------------------------- |
| sub_type    | String | 撤回类型 recall 或 delete |
| message_id  | String | 消息 ID                   |
| user_id     | String | 成员 ID                   |
| group_id    | String | 群 ID                     |
| operator_id | String | 操作者 ID                 |

### 群管理员设置 notice.group_admin_set

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| user_id     | String | 成员 ID   |
| group_id    | String | 群 ID     |
| operator_id | String | 操作者 ID |

### 群管理员取消 notice.group_admin_unset

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| user_id     | String | 成员 ID   |
| group_id    | String | 群 ID     |
| operator_id | String | 操作者 ID |

### * 群名称更新 notice.group_name_update

> 此事件为扩展事件

| 字段        | 类型   | 说明      |
| ----------- | ------ | --------- |
| group_id    | String | 群 ID     |
| group_name  | String | 群名称    |
| operator_id | String | 操作者 ID |

## 请求事件 request

| 字段 | 类型 | 说明      |
| ---- | ---- | --------- |
| self | Self | Self 标识 |

### * 好友添加请求 request.new_friend

> 此事件为扩展事件

| 字段       | 类型   | 说明          |
| ---------- | ------ | ------------- |
| request_id | i64    | 请求 ID       |
| user_id    | String | 用户 ID       |
| user_name  | String | 用户名称/昵称 |
| message    | String | 请求信息      |

### * 新成员加群申请 request.join_group

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

### * 群邀请 request.group_invited

| 字段         | 类型             | 说明       |
| ------------ | ---------------- | ---------- |
| request_id   | i64              | 请求 ID    |
| group_id     | String           | 群 ID      |
| group_name   | String           | 群名称     |
| invitor_id   | Option\<String\> | 邀请人 ID  |
| invitor_name | Option\<String\> | 邀请人名称 |
