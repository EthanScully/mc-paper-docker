```shell
docker run --name minecraft -v $(pwd):/minecraft -itd ethanscully/papermc:latest
```

### Optional Environmental Variables
| Env Vars      | Example          |
| :------------ | :--------------- |
| `CRON`        | `0 45 5 * * * *` |
| `JAVACMDADD`  | `-Xmx8G`         |
