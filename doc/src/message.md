# 消息 Message

根据 OneBot 协议规定，消息是由不定数个消息段 (MessageSegment) 组成的 List。

以下列举支持的消息段：

## 文本消息 text

| 字段 | 类型   | 说明     |
| ---- | ------ | -------- |
| text | String | 消息文本 |

## 提及消息 mention

| 字段    | 类型   | 说明    |
| ------- | ------ | ------- |
| user_id | String | 用户 ID |

## 提及所有人消息 mention_all

无字段

## 表情消息 face

| 字段 | 类型   | 说明     |
| ---- | ------ | -------- |
| id   | Int    | 表情 ID  |
| file | String | 表情名称 |

> 发送时优先根据 id 确认表情当 id 不存在时，可以根据 file 确认表情。
> 二者均无法匹配时，将忽略该消息段。

## 骰子消息 dice

> *未支持发送该消息段*

| 字段  | 类型 | 说明   |
| ----- | ---- | ------ |
| value | Int  | 骰子值 |

## 石头剪刀布消息 rps

> *未支持发送该消息段*

| 字段  | 类型 | 说明                          |
| ----- | ---- | ----------------------------- |
| value | Int  | 石头 => 0，剪刀 => 1，布 => 2 |

## 图片消息 image

| 字段    | 类型   | 说明             |
| ------- | ------ | ---------------- |
| file_id | String | 图片文件 ID      |
| url     | String | 可选，图片 url   |
| flash   | bool   | 可选，是否为闪照 |

> url支持协议：
> 
> - http: `http://example.com/image.png`
> - https: `https://example.com/image.png`
> - file: `file:///path/to/image.png`
> - base64: `base64://image_base64_str`

## 语音消息 voice

| 字段    | 类型   | 说明        |
| ------- | ------ | ----------- |
| file_id | String | 语音文件 ID |

## 富文本消息 json

> *未支持发送该消息段*

| 字段 | 类型   | 说明      |
| ---- | ------ | --------- |
| data | String | json 内容 |

## 回复消息 reply

| 字段       | 类型   | 说明             |
| ---------- | ------ | ---------------- |
| message_id | String | 回复引用的消息ID |
| user_id    | String | 回复引用的用户ID |

## 富文本消息 xml

| 字段       | 类型   | 说明     |
| ---------- | ------ | -------- |
| service_id | i64    | 服务 ID  |
| data       | String | xml 内容 |

## 合并转发 forward

> forward 消息段仅支持单独发送，与其他 MessageSegment 混合时将被忽略
> 消息段接收时合并转发将被视为 xml 富文本消息段

| 字段  | 类型        | 说明           |
| ----- | ----------- | -------------- |
| nodes | Vec\<Node\> | 转发的消息节点 |

## 合并转发节点 node

> 无法单独发送，请使用 forward 消息段包含 node 发送

| 字段      | 类型                                 | 说明    |
| --------- | ------------------------------------ | ------- |
| user_id   | String                               | 用户 ID |
| time      | f64                                  | 时间    |
| user_name | String                               | 用户名  |
| message   | Vec\<MessageSegment\> \| Vec\<Node\> | 消息    |