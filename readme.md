```shell
docker run --name minecraft -v $(pwd):/minecraft -itd ethanscully:papermc
```

| Env Vars      | Example          |
| :------------ | :--------------- |
| `CRON`        | `0 45 5 * * * *` |
| `JAVACMDADD`  | `-Xmx8G`         |
