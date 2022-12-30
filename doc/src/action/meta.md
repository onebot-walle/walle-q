# 元动作

## 获取近期事件 get_latest_events

动作请求：

| 字段      | 类型 | 说明                                             |
| --------- | ---- | ------------------------------------------------ |
| `limit`   | i64  | 获取的事件数量上限，0 使用默认值 10              |
| `timeout` | i64  | 没有事件时要等待的秒数，0 表示使用短轮询，不等待 |

动作响应为 Vec&lt;Event&gt; 

## 获取支持的动作列表 get_supported_actions

无动作请求数据

动作响应为 Vec&lt;String&gt; 

## 获取运行状态 get_status

无动作请求数据

动作响应：Status

## 获取版本信息 get_version

无动作请求数据

动作响应：

| 字段             | 类型   | 说明           |
| ---------------- | ------ | -------------- |
| `platform`       | String | `qq`           |
| `version`        | String | Walle-Q 版本号 |
| `onebot_version` | String | `12`           |

## * 关闭应用 shutdown

动作请求:

| 字段          | 类型   | 说明             |
| ------------- | ------ | ---------------- |
| `super_token` | String | 超级管理员 token |

无动作响应

## * [Plan to add] 登录新账号 login_client

动作请求:

| 字段           | 类型   | 说明            |
| -------------- | ------ | --------------- |
| `uin`          | u32    | QQ 号           |
| `password`     | String | 可选，明文密码  |
| `password_md5` | String | 可选，MD5 密码  |
| `protcol`      | u8     | client 使用协议 |

动作响应:

| 字段      | 类型   | 说明                         |
| --------- | ------ | ---------------------------- |
| `user_id` | String | user_id                      |
| `url`     | String | 可选，ticket_url             |
| `qrcode`  | bytes  | 可选，密码缺失将会返回二维码 |

## * [Plan to add] 提交ticket submit_ticket

动作请求:

| 字段      | 类型   | 说明   |
| --------- | ------ | ------ |
| `user_id` | String | -      |
| `ticket`  | String | ticket |

无动作响应字段