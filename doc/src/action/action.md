# 动作 Action

> \* 标记表示扩展字段或动作

## 错误代码 Error Code

| Code  | 名称                     | 说明                   | 备注                      |
| ----- | ------------------------ | ---------------------- | ------------------------- |
| 10001 | Bad Request              | 无效的动作请求         | 格式错误等                |
| 10002 | Unsupported Action       | 不支持的动作请求       | 不支持的动作              |
| 10003 | Bad Param                | 无效的请求参数         | 参数缺失或参数类型错误    |
| 10004 | Unsupported Param        | 不支持的请求参数       | 请求参数未实现            |
| 10005 | Unsupported Segment      | 不支持的消息段类型     | 消息段类型未实现          |
| 10006 | Bad Segment Data         | 无效的消息段数据       | 消息段数据格式错误        |
| 10007 | Unsupported Segment Data | 不支持的消息段参数     | 消息段参数未实现          |
| 10101 | Who Am I                 | 缺失 self 字段         | 无法识别执行的 bot        |
| 20001 | Bad Handler              | 动作处理器实现错误     | 没有正确设置响应状态等    |
| 20002 | Internal Handler Error   | 动作处理器运行时异常   | 未捕获的意料之外的异常    |
| 20003 | Prepare File Error       | 分片上传文件未 prepare | 分片上传文件未 prepare    |
| 20004 | File Size Error          | 分片上传文件大小错误   | 文件大小与预先约定不匹配  |
| 20005 | File Sha256 Error        | 分片上传文件哈希错误   | Sha256 值与预先约定不匹配 |
| 32001 | File Open Error          | 文件打开失败           | 文件打开失败              |
| 32002 | File Read Error          | 文件读取失败           | 文件读取失败              |
| 32003 | File Create Error        | 文件创建失败           | 文件创建失败              |
| 32004 | File Write Error         | 文件写入失败           | 文件写入失败              |
| 32005 | File Not Found Error     | 文件不存在             | 文件不存在                |
| 33001 | Net Download Error       | 网络下载错误           | 网络下载错误              |
| 34001 | Ricq Error               | ricq 未处理报错        | ricq 报错                 |
| 34002 | login failed             | 登录失败               | 登录失败                  |
| 34003 | risk_controlled          | 可能被风控             | 可能被风控                |
| 35001 | Message Not Exist        | 消息不存在             | 消息不存在                |
| 35002 | Friend Not Exist         | 好友不存在             | 好友不存在                |
| 35003 | Group Not Exist          | 群不存在               | 群不存在                  |
| 35004 | Group Member Not Exist   | 群成员不存在           | 群成员不存在              |
| 35005 | Permission Denied        | 权限不足               | 不是群管理员等            |
| 61001 | ImageInfo Decode Error   | 图片信息解码错误       | 图片信息解码错误          |
| 61002 | Image Url Error          | 图片URL错误            | 图片 URL 不存在或解析错误 |
| 61003 | Image Path Error         | 图片路径错误           | 图片路径不存在或解析错误  |
| 61004 | Image Data Error         | 图片内容错误           | 图片文件下载或读取失败    |
