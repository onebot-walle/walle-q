## 开始使用

```zsh
# 直接运行
./walle-q
# or
./walle-q -h # for help
```

> 默认时间戳为本地时区，如需设置，请配置 --time-zone

## 配置列表

```toml
[qq.name]                     # name 在密码登录时必须与 QQ 号一致
password = "your password"    # password
protocol = 0                  # 0..=5，默认为0

[qq.other]                    # 缺失密码时默认使用扫码登录，扫码登录仅手表或 MacOS 可用

[meta]
log = "info"                  # 日志级别，可选：trace, debug, info, warn, error
event_cache_size = 10         # 事件缓存区大小
sled = false                  # 启用 sled 数据库
leveldb = true                # 启用 leveldb 数据库
data_path = "./data"          # 数据文件路径
log_path = "./log"            # log 文件保存路径
super_token =                 # 超级管理 token 默认为未设置，必须设置才可以使用 super manager action

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
