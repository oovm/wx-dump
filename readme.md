

## 指令

```yaml
微信聊天记录导出工具

Usage: wxdump.exe [OPTIONS] [COMMAND]

Commands:
  info     显示当前登录的微信用户的信息
  decrypt  解密聊天记录数据库
  search   从内存中搜索指定信息
  read     从内存中指定的位置搜索信息
  help     Print this message or the help of the given subcommand(s)

Options:
  -m, --offset-map <json 文件>
          指定偏移量文件

          [default: version-list.json]

  -d, --wechat-path <微信文件夹>
          指定微信聊天记录的文件夹，不填写时会默认指定系统文档文件夹下的 WeChat Files 文件夹

      --decrypt-path <解密文件夹>
          解密后的数据将存放于该文件夹

      --process-id <PROCESS_ID>
          指定微信进程号

      --process-name <PROCESS_NAME>
          指定微信进程名

          [default: WeChat.exe]

      --module-name <MODULE_NAME>
          指定模块名

          [default: WeChatWin.dll]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```