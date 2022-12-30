# 消息动作

## 发送消息 send_message

动作请求：

| 字段          | 类型    | 说明                                 |
| ------------- | ------- | ------------------------------------ |
| `detail_type` | String  | `private` \| `group` \| `group_temp` |
| `group_id`    | String  | 可选，群 ID                          |
| `user_id`     | String  | 可选，用户 ID                        |
| `message`     | Message | 消息内容                             |

动作响应：

| 字段         | 类型   | 说明             |
| ------------ | ------ | ---------------- |
| `message_id` | String | 消息 ID          |
| `time`       | f64    | 时间戳，单位：秒 |

## 删除消息 delete_message

动作请求：

| 字段         | 类型   | 说明    |
| ------------ | ------ | ------- |
| `message_id` | String | 消息 ID |

无动作响应数据

## * 获取消息 get_message

动作请求：

| 字段         | 类型   | 说明    |
| ------------ | ------ | ------- |
| `message_id` | String | 消息 ID |

响应数据:

同消息事件 MessageEvent