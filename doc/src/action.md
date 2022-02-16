# 动作 Action

## 发送消息 send_message

| 字段        | 类型                 | 说明                   |
| ----------- | -------------------- | ---------------------- |
| detail_type | String               | `private` 或者 `group` |
| group_id    | Option&lt;String&gt; | 群 ID                  |
| user_id     | Option&lt;String&gt; | 用户 ID                |
| message     | Message              | 消息内容               |