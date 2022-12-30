# 消息事件 message

## 单用户消息事件 message.private

| 字段          | 类型    | 说明            |
| ------------- | ------- | --------------- |
| `message_id`  | String  | 消息 ID         |
| `message`     | Message | 消息对象        |
| `alt_message` | String  | 消息文本        |
| `user_id`     | String  | 发送者 ID       |
| * `user_name` | String  | 发送者 nickname |

## * 群临时消息 message.group_temp

| 字段          | 类型    | 说明            |
| ------------- | ------- | --------------- |
| `message_id`  | String  | 消息 ID         |
| `message`     | Message | 消息对象        |
| `alt_message` | String  | 消息文本        |
| `user_id`     | String  | 发送者 ID       |
| `group_id`    | String  | 群 ID           |
| `user_name`   | String  | 发送者 nickname |

## 群用户消息事件 message.group

| 字段           | 类型    | 说明       |
| -------------- | ------- | ---------- |
| `message_id`   | String  | 消息 ID    |
| `message`      | Message | 消息对象   |
| `alt_message`  | String  | 消息文本   |
| `user_id`      | String  | 发送者 ID  |
| `group_id`     | String  | 群 ID      |
| * `user_card`  | String  | 发送者名片 |
| * `group_name` | String  | 群名称     |