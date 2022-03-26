# 动作 Action

## 获取近期事件 get_latest_events

动作请求：

| 字段    | 类型 | 说明                                             |
| ------- | ---- | ------------------------------------------------ |
| limit   | i64  | 获取的事件数量上限，0 使用默认值 10              |
| timeout | i64  | 没有事件时要等待的秒数，0 表示使用短轮询，不等待 |

动作响应为 Vec&lt;Event&gt; 

## 获取支持的动作列表 get_supported_actions

无动作请求数据

动作响应为 Vec&lt;String&gt; 

## 获取运行状态 get_status

无动作请求数据

动作响应：

| 字段   | 类型 | 说明                                            |
| ------ | ---- | ----------------------------------------------- |
| good   | bool | 是否各项状态都符合预期，OneBot 实现各模块均正常 |
| online | bool | 是否在线                                        |

## 获取版本信息 get_version

无动作请求数据

动作响应：

| 字段           | 类型   | 说明           |
| -------------- | ------ | -------------- |
| impl           | String | `Walle-Q`      |
| platform       | String | `qq`           |
| version        | String | Walle-Q 版本号 |
| onebot_version | String | `12`           |

## 发送消息 send_message

动作请求：

| 字段        | 类型                 | 说明                   |
| ----------- | -------------------- | ---------------------- |
| detail_type | String               | `private` 或者 `group` |
| group_id    | Option&lt;String&gt; | 群 ID                  |
| user_id     | Option&lt;String&gt; | 用户 ID                |
| message     | Message              | 消息内容               |

动作响应：

| 字段       | 类型   | 说明             |
| ---------- | ------ | ---------------- |
| message_id | String | 消息 ID          |
| time       | f64    | 时间戳，单位：秒 |

## 删除消息 delete_message

动作请求：

| 字段       | 类型   | 说明    |
| ---------- | ------ | ------- |
| message_id | String | 消息 ID |

无动作响应数据

## 获取机器人自身信息 get_self_info

无动作请求数据

动作响应：

| 字段     | 类型   | 说明            |
| -------- | ------ | --------------- |
| user_id  | String | 机器人用户 ID   |
| nickname | String | 机器人名称/昵称 |

## 获取用户信息 get_user_info

动作请求:

| 字段    | 类型   | 说明    |
| ------- | ------ | ------- |
| user_id | String | 用户 ID |

动作响应：

| 字段     | 类型   | 说明      |
| -------- | ------ | --------- |
| user_id  | String | 用户 ID   |
| nickname | String | 名称/昵称 |

## 获取好友列表 get_friend_list

无动作请求数据

好友信息列表，每一个元素的字段同 `get_user_info` 的响应数据。

## 获取群信息 get_group_info

动作请求:

| 字段     | 类型   | 说明  |
| -------- | ------ | ----- |
| group_id | string | 群 ID |

动作响应：

| 字段名     | 数据类型 | 说明   |
| ---------- | -------- | ------ |
| group_id   | string   | 群 ID  |
| group_name | string   | 群名称 |

## 获取群列表 get_group_list

无动作请求数据

群信息列表，每一个元素的字段同 `get_group_info` 的响应数据。

## 获取群成员列表 get_group_member_info

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

动作响应：

| 字段名   | 数据类型 | 说明          |
| -------- | -------- | ------------- |
| user_id  | string   | 用户 ID       |
| nickname | string   | 用户名称/昵称 |

## 获取群成员列表 get_group_member_list

动作请求:

| 字段     | 类型   | 说明  |
| -------- | ------ | ----- |
| group_id | string | 群 ID |

群信息列表，每一个元素的字段同 `get_group_member_info` 的响应数据。

## 设置群名称 set_group_name

动作请求:

| 字段       | 类型   | 说明   |
| ---------- | ------ | ------ |
| group_id   | string | 群 ID  |
| group_name | string | 群名称 |

无动作响应数据

## 退出群 leave_group

动作请求:

| 字段     | 类型   | 说明  |
| -------- | ------ | ----- |
| group_id | string | 群 ID |

无动作响应数据

## 踢出群成员 kick_group_member

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据

## 禁言群成员 ban_group_member

动作请求:

| 字段     | 类型   | 说明           |
| -------- | ------ | -------------- |
| group_id | string | 群 ID          |
| user_id  | string | 用户 ID        |
| duration | i64    | 时长，单位：秒 |

无动作响应数据

## 解禁群成员 unban_group_member

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据

## 设置群管理员 set_group_admin

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据

## 取消群管理员 unset_group_admin

动作请求:

| 字段     | 类型   | 说明    |
| -------- | ------ | ------- |
| group_id | string | 群 ID   |
| user_id  | string | 用户 ID |

无动作响应数据

# 错误代码 Error Code

| Code  | 名称                     | 说明                 | 备注                     |
| ----- | ------------------------ | -------------------- | ------------------------ |
| 10001 | Bad Request              | 无效的动作请求       | 格式错误等               |
| 10002 | Unsupported Action       | 不支持的动作请求     | 不支持的动作             |
| 10003 | Bad Param                | 无效的请求参数       | 参数缺失或参数类型错误   |
| 10004 | Unsupported Param        | 不支持的请求参数     | 请求参数未实现           |
| 10005 | Unsupported Segment      | 不支持的消息段类型   | 消息段类型未实现         |
| 10006 | Bad Segment Data         | 无效的消息段数据     | 消息段数据格式错误       |
| 10007 | Unsupported Segment Data | 不支持的消息段参数   | 消息段参数未实现         |
| 20001 | Bad Handler              | 动作处理器实现错误   | 没有正确设置响应状态等   |
| 20002 | Internal Handler Error   | 动作处理器运行时异常 | 未捕获的意料之外的异常   |
| 32001 | File Open Error          | 文件打开失败         | 文件打开失败             |
| 32002 | File Read Error          | 文件读取失败         | 文件读取失败             |
| 32003 | File Create Error        | 文件创建失败         | 文件创建失败             |
| 32004 | File Write Error         | 文件写入失败         | 文件写入失败             |
| 33001 | Net Download Error       | 网络下载错误         | 网络下载错误             |
| 34001 | Rs-qq Error              | rs-qq 未处理报错     | rs-qq 报错               |
| 35001 | Message Not Exist        | 消息不存在           | 消息不存在               |
| 35002 | Friend Not Exist         | 好友不存在           | 好友不存在               |
| 35003 | Group Not Exist          | 群不存在             | 群不存在                 |
| 35004 | Group Member Not Exist   | 群成员不存在         | 群成员不存在             |
| 41001 | ImageInfo Decode Error   | 图片信息解码错误     | 图片信息解码错误         |
| 41002 | Image Url Error          | 图片URL错误          | 图片URL不存在或解析错误  |
| 41003 | Image Path Error         | 图片路径错误         | 图片路径不存在或解析错误 |
| 41004 | Image Data Error         | 图片内容错误         | 图片文件下载或读取失败   |
