## 开始使用

```zsh
# 直接运行
./walle-q
# or
./walle-q -h # for help
```

> 默认时间戳为+8时区，如需设置，请配置 --time-zone

## 配置列表

```toml
[qq.name]                     # name在密码登录时必须与 QQ 号一致
password = "your password"    # password
protocol = 0                  # 0..=5，默认为0

[qq.other]                    # 缺失密码时默认使用扫码登录，扫码登录仅手表或MacOS可用

[meta]
log = "Info"                  # 日志级别，可选：Trace, Debug, Info, Warn, Error
event_cache_size = 16         # 事件缓存区大小
v11 = false                   # v11 协议模式（暂时失效）
time_zone = +8                # log 时区，默认为+8
sled = false                  # 启用 sled 数据库
disable_leveldb = false       # 禁用 level_db 数据库

[[onebot.http]]
host = "127.0.0.1"
port = 6700
access_token = "token"        # 默认为空
event_enable =  true          # 是否推送事件
event_buffer_size = 16        # 事件推送缓存区大小

[[onebot.http_webhook]]
url = "http://127.0.0.1:6700" 
access_token = "token"        # 默认为空
timeout = 4                   # 超时时间，单位秒

[[onebot.websocket]]
host = "127.0.0.1"
port = 8844
access_token = "token"        # 默认为空

[[onebot.websocket_rev]]
url = "ws://127.0.0.1:8844"
access_token = "token"        # 默认为空
reconnect_interval = 4

[onebot.heartbeat]
enabled = true
interval = 4
```

已支持协议设备：

- 0: `IPad`
- 1: `AndroidPhone`
- 2: `AndroidWatch`
- 3: `MacOS`
- 4: `QiDian`
- 5: `IPad`
