# (WIP) Walle-Q

> Walle Mk.Q

一个 QQ 平台的 OneBot 协议实现端

A qq platform OneBot Implementation

目前 [rs-qq](https://github.com/lz1998/rs-qq) 项目与 [Walle-core](https://github.com/abrahum/walle-core) 项目均未完全完成初版，本项目的开发优先级位于二者之后。

> 本项目采用 AGPLv3 开源协议，仅出于学习目的开发，不鼓励、不支持任何除此以外的任何其他用途。

## 登录方式

- [x] 账户密码登录
- [x] 扫码登录
- [x] Token 登录

## 已支持 Event

- [x] 群消息
- [x] 私聊消息
- [ ] ...
## 已支持消息段

- [x] 文本消息
- [x] AT消息
- [ ] ...

## 已支持 API

- [x] get_supported_actions
- [x] get_status
- [x] get_version
- [x] send_message
- [x] get_self_info
- [x] get_user_info
- [x] get_friend_list
- [x] get_group_info
- [x] get_group_list
- [x] get_group_member_list
- [x] get_group_member_info
- [ ] ...

## 已知问题

- Token 登录后 nickname 缺失