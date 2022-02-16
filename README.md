# (WIP) Walle-Q

![OneBot12](https://img.shields.io/badge/OneBot-12-black?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAHAAAABwCAMAAADxPgR5AAAAGXRFWHRTb2Z0d2FyZQBBZG9iZSBJbWFnZVJlYWR5ccllPAAAAAxQTFRF////29vbr6+vAAAAk1hCcwAAAAR0Uk5T////AEAqqfQAAAKcSURBVHja7NrbctswDATQXfD//zlpO7FlmwAWIOnOtNaTM5JwDMa8E+PNFz7g3waJ24fviyDPgfhz8fHP39cBcBL9KoJbQUxjA2iYqHL3FAnvzhL4GtVNUcoSZe6eSHizBcK5LL7dBr2AUZlev1ARRHCljzRALIEog6H3U6bCIyqIZdAT0eBuJYaGiJaHSjmkYIZd+qSGWAQnIaz2OArVnX6vrItQvbhZJtVGB5qX9wKqCMkb9W7aexfCO/rwQRBzsDIsYx4AOz0nhAtWu7bqkEQBO0Pr+Ftjt5fFCUEbm0Sbgdu8WSgJ5NgH2iu46R/o1UcBXJsFusWF/QUaz3RwJMEgngfaGGdSxJkE/Yg4lOBryBiMwvAhZrVMUUvwqU7F05b5WLaUIN4M4hRocQQRnEedgsn7TZB3UCpRrIJwQfqvGwsg18EnI2uSVNC8t+0QmMXogvbPg/xk+Mnw/6kW/rraUlvqgmFreAA09xW5t0AFlHrQZ3CsgvZm0FbHNKyBmheBKIF2cCA8A600aHPmFtRB1XvMsJAiza7LpPog0UJwccKdzw8rdf8MyN2ePYF896LC5hTzdZqxb6VNXInaupARLDNBWgI8spq4T0Qb5H4vWfPmHo8OyB1ito+AysNNz0oglj1U955sjUN9d41LnrX2D/u7eRwxyOaOpfyevCWbTgDEoilsOnu7zsKhjRCsnD/QzhdkYLBLXjiK4f3UWmcx2M7PO21CKVTH84638NTplt6JIQH0ZwCNuiWAfvuLhdrcOYPVO9eW3A67l7hZtgaY9GZo9AFc6cryjoeFBIWeU+npnk/nLE0OxCHL1eQsc1IciehjpJv5mqCsjeopaH6r15/MrxNnVhu7tmcslay2gO2Z1QfcfX0JMACG41/u0RrI9QAAAABJRU5ErkJggg==)
<a href="https://github.com/abrahum/walle-q/blob/master/LICENSE">
  <img src="https://img.shields.io/github/license/abrahum/walle-q" alt="license">
</a>

> Walle Mk.Q

一个 QQ 平台的 OneBot 协议实现端

A qq platform OneBot Implementation

本项目使用 [rs-qq](https://github.com/lz1998/rs-qq) 协议库与 [Walle-core](https://GitHub.com/abrahum/walle-core) LibOnebot 构建。

在线文档地址：[Walle-Mk.Q 使用手册](https://walle-q.abrahum.link)

> 本项目采用 AGPLv3 开源协议，仅出于学习目的开发，不鼓励、不支持任何除此以外的任何其他用途。

## 登录方式

- [x] 账户密码登录
- [x] 扫码登录
- [x] Token 登录

## 已支持 Event

- [x] 私聊消息
- [x] 群消息
- [x] 私聊消息撤回
- [x] 好友增加
- [x] 群成员增加
- [x] 群成员减少
- [x] 群成员禁言
- [x] 群管理员设置
- [x] 群管理取消设置
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

## OneBot-v11 协议支持

*Comming soon*

## 已知问题

- Token 登录后 nickname 缺失
- 群管理设置 `operator_id` 缺失