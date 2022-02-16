# 事件 Event

所有事件共有字段

| 字段     | 类型   | 说明                 |
| -------- | ------ | -------------------- |
| id       | String | 事件ID               |
| impl     | String | 实现名，即 `Walle-Q` |
| platform | String | 平台名，即 `qq`      |
| selfid   | String | Bot ID               |
| time     | f64    | 事件戳，单位：秒     |

## 消息事件 message

### 单用户消息事件 message.private

| 字段        | 类型    | 说明      |
| ----------- | ------- | --------- |
| message_id  | String  | 消息 ID   |
| message     | Message | 消息对象  |
| alt_message | String  | 消息文本  |
| user_id     | String  | 发送者 ID |

### 群用户消息事件 message.group

| 字段        | 类型    | 说明      |
| ----------- | ------- | --------- |
| message_id  | String  | 消息 ID   |
| message     | Message | 消息对象  |
| alt_message | String  | 消息文本  |
| user_id     | String  | 发送者 ID |
| group_id    | String  | 群 ID     |
