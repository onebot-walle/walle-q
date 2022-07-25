## 0.2.0

### Fixes

- #12: 群操作权限检查，群成员信息获取检查
- #19: at 显示缺失 @ 字符

### Dependencies

- walle-core: 0.6.0-a1 -> 0.6.0
- ricq: 0.1.15 -> 0.1.16

### Breaking Changes

- forward 消息段被移除，直接发送 Vec\<node\> 即视为 forward 消息段（优先级低于普通其他消息段）

### Others

- 感谢 [Itsusinn](https://github.com/Itsusinn) 对项目 ci 的优化 pr

## 0.2.0-a1

### Features

- 支持多 Bot 登录

### Dependencies

- walle-core: 0.6.0-a1
- 移除 qrcode 相关依赖

### Others

- 配置文件切换为 toml 格式
- 优化编译后文件大小

## 0.1.4

### Features

- 添加 event.message.group_temp
- 添加 event.notice.friend_poke
- 添加 event.notice.group_name_update
- 添加 event.request.join_group
- 添加 event.request.group_invited
- 添加 set_join_group 动作
- 添加 get_join_group_requests 动作
- 添加 set_group_invited 动作
- 添加 get_group_inviteds 动作

### Dependencies

- walle-core: 0.5.4

### Others

- 暂时移除 Onebot V11 支持

## 0.1.3

### Features

- 添加 event.request.new_friend
- 添加 set_new_friend 动作
- 添加 delete_friend 动作
- 添加 get_new_friend_requests 动作
- 添加 upload_file_fragmented 动作支持
- 添加 get_file_fragmented 动作支持
- 支持收发 Reply 消息段
- 支持发送 forward 消息段

## 0.1.2

### Features

- 命令行二维码打印
- 添加部分扩展字段

### Internal

- ricq update to 0.1.8
- walle-core update to 0.5.0

## 0.1.1

- 修复 v11 模式 url file 协议解析
- 新增 log 文件，将会记录 WARN Level 以上的错误信息

## 0.1.0

Nothing new

## 0.0.3

- 支持图片消息发送
- Onebot v11 支持

## 0.0.2

- 支持更多 action
- 修复 websocket 连接断开问题

## 0.0.1 

- 第一个 pre release 版本