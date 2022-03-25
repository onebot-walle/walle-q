## 开始使用

```zsh
# 直接运行
./walle-q
# or
./walle-q -h # for help
```

> 默认时间戳为+8时区，如需设置，请配置 --time-zone

## 配置列表

```yaml
onebot: # Onebot 协议相关配置
  heartbeat:
    enabled: true           # 是否启用心跳
    interval: 4             # 心跳间隔，单位秒
  http:                     # http 配置
    - host: "your.host"
      port: 6700
      access_token: "..."   # 可选，默认为空
      event_enable: true    # 是否推送事件
      event_buffer_size: 16 # 事件推送缓存区大小
  http_webhook: []          # http webhook 配置
    - url: "webhook.url"
      access_token: "..."   # 可选，默认为空
      time_out: 4           # 超时时间，单位秒
  websocket:                # websocket 配置
    - host: "your.host"
      port: 8844
      access_token: "..."   # 可选，默认为空
  websocket_rev:            # websocket 反向配置
    - url: "your.url"       
      access_token: "..."   # 可选，默认为空
      reconnect_interval: 4 # 重连间隔，单位秒

uin: ~                      # 可选，为空则使用扫码登陆
password: ~                 # 可选，为空则使用扫码登陆
protocol: ~                 # 可选，0..5，默认为0
str_protocol: ~             # 可选，默认 IPad，优先级低于protcol
```

已支持协议设备：

- 0 -> `IPad`
- 1 -> `AndroidPhone`
- 2 -> `AndroidWatch`
- 3 -> `MacOS`
- 4 -> `QiDian`
- 5 -> `IPad`
