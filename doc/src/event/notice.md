# 通知事件 notice

## 好友消息撤回 notice.private_message_delete

| 字段         | 类型   | 说明      |
| ------------ | ------ | --------- |
| `message_id` | String | 消息 ID   |
| `user_id`    | String | 发送者 ID |

## 好友增加 notice.friend_increase

| 字段          | 类型   | 说明     |
| ------------- | ------ | -------- |
| `user_id`     | String | 好友 ID  |
| * `user_name` | String | 好友昵称 |

## 好友减少 notice.friend_decrease

| 字段      | 类型   | 说明    |
| --------- | ------ | ------- |
| `user_id` | String | 好友 ID |

## * 好友戳一戳 notice.friend_poke

> 此事件为扩展事件

| 字段          | 类型   | 说明      |
| ------------- | ------ | --------- |
| `user_id`     | String | 发送者 ID |
| `receiver_id` | String | 接收者 ID |

## * 群友戳一戳 notice.group_poke

> 此事件为扩展事件

| 字段          | 类型   | 说明      |
| ------------- | ------ | --------- |
| `group_id`    | String | 群 ID     |
| `user_id`     | String | 发送者 ID |
| `receiver_id` | String | 接收者 ID |

## 群成员增加 notice.group_member_increase

| 字段          | 类型   | 说明             |
| ------------- | ------ | ---------------- |
| `sub_type`    | String | 加群类型         |
| `user_id`     | String | 成员 ID          |
| `group_id`    | String | 群 ID            |
| `operator_id` | String | 操作者 ID (暂缺) |

## 群成员减少 notice.group_member_decrease

| 字段          | 类型   | 说明                                    |
| ------------- | ------ | --------------------------------------- |
| `sub_type`    | String | 退群类型 `leave` \| `kick` \| `disband` |
| `user_id`     | String | 成员 ID                                 |
| `group_id`    | String | 群 ID                                   |
| `operator_id` | String | 操作者 ID                               |

## 群成员禁言 notice.group_member_ban

| 字段          | 类型   | 说明               |
| ------------- | ------ | ------------------ |
| `user_id`     | String | 成员 ID            |
| `group_id`    | String | 群 ID              |
| `operator_id` | String | 操作者 ID          |
| * `duration`  | i64    | 禁言时长，单位：秒 |

## 群消息撤回 notice.group_message_delete

| 字段          | 类型   | 说明                          |
| ------------- | ------ | ----------------------------- |
| `sub_type`    | String | 撤回类型 `recall` 或 `delete` |
| `message_id`  | String | 消息 ID                       |
| `user_id`     | String | 成员 ID                       |
| `group_id`    | String | 群 ID                         |
| `operator_id` | String | 操作者 ID                     |

## 群管理员设置 notice.group_admin_set

| 字段          | 类型   | 说明      |
| ------------- | ------ | --------- |
| `user_id`     | String | 成员 ID   |
| `group_id`    | String | 群 ID     |
| `operator_id` | String | 操作者 ID |

## 群管理员取消 notice.group_admin_unset

| 字段          | 类型   | 说明      |
| ------------- | ------ | --------- |
| `user_id`     | String | 成员 ID   |
| `group_id`    | String | 群 ID     |
| `operator_id` | String | 操作者 ID |

## * 群名称更新 notice.group_name_update

| 字段          | 类型   | 说明      |
| ------------- | ------ | --------- |
| `group_id`    | String | 群 ID     |
| `group_name`  | String | 群名称    |
| `operator_id` | String | 操作者 ID |