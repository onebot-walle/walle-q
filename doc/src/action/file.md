# 文件动作

## 上传文件 upload_file

动作请求：

| 字段        | 类型                  | 说明                                   |
| ----------- | --------------------- | -------------------------------------- |
| `type`      | String                | 上传方式：`url` \| `path` \| `data`    |
| `name`      | String                | 文件名称                               |
| `url`       | String                | 可选，上传方式为 url 时需要提供的 url  |
| `data`      | Map\<String, String\> | 可选，url 可选 headers                 |
| `path`      | String                | 可选，上传方式为 path 时需要提供的路径 |
| `data`      | String                | 可选，上传方式为 data 时需要提供的数据 |
| `sha256`    | String                | 可选，文件的 sha256 值                 |
| `file_type` | String                | 可选，文件类型，缺省值为 `image`       |

动作响应：

| 字段      | 类型   | 说明    |
| --------- | ------ | ------- |
| `file_id` | String | 文件 ID |

## 获取文件 get_file

动作请求：

| 字段        | 类型   | 说明                                |
| ----------- | ------ | ----------------------------------- |
| `file_id`   | String | 文件 ID                             |
| `type`      | String | 上传方式：`url` \| `path` \| `data` |
| `file_type` | String | 可选，文件类型，缺省值为 `image`    |

动作响应：

| 字段     | 类型                  | 说明                                   |
| -------- | --------------------- | -------------------------------------- |
| `type`   | String                | 上传方式：`url` \| `path` \| `data`    |
| `name`   | String                | 文件名称                               |
| `url`    | String                | 可选，上传方式为 url 时需要提供的 url  |
| `data`   | Map\<String, String\> | 可选，url 可选 headers                 |
| `path`   | String                | 可选，上传方式为 path 时需要提供的路径 |
| `data`   | String                | 可选，上传方式为 data 时需要提供的数据 |
| `sha256` | String                | 可选，文件的 sha256 值                 |

## 分片上传文件 upload_file_fragmented

> 累了，看 Onebot12 文档吧，一样的 ╯︿╰

## 分片获取文件 get_file_fragmented

> 累了，看 Onebot12 文档吧，一样的 ╯︿╰