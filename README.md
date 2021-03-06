# selfoss-discord


[![Check Python syntax and typing](https://github.com/evroon/selfoss-discord/actions/workflows/mypy.yaml/badge.svg)](https://github.com/evroon/selfoss-discord/actions/workflows/mypy.yaml)

Send RSS updates from Selfoss to Discord.

The Python script fetches new RSS items from `<selfoss_url>/items` and sends a message to discord for each new RSS item. The messages are sent in specific Discord channels that are created based on the RSS feed's name.

Preview of an RSS update message in Discord: (using [msfs-rss](https://github.com/evroon/msfs-rss))

![Preview](https://raw.githubusercontent.com/evroon/selfoss-discord/main/etc/preview.png)

## Usage
Create a bot in Discord with permissions to send manages and manage channels for the server you want to send messages to.
Create a file called `.env` with the following content:
```bash
DISCORD_TOKEN="your discord bot token"
DISCORD_SERVER_ID="the ID of your server/guild"
```

You can update selfoss and send messages to discord hourly by opening crontab (`sudo crontab -e`) adding the following line:
```bash
0 * * * * sudo -Hu www-data php /var/www/selfoss/cliupdate.php && sudo -Hu <username> python3 /path/to/selfoss.py https://selfoss.domain.com /path/to/last-update
```

But using a systemd service and timer would be neater, such as this [service](https://github.com/evroon/concordia/blob/master/lib/systemd/system/selfoss-update.service) and [timer](https://github.com/evroon/concordia/blob/master/lib/systemd/system/selfoss-update.timer).
