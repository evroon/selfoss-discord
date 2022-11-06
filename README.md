# selfoss-discord

Send RSS updates from Selfoss to Discord.

The Python script fetches new RSS items from `<selfoss_url>/items` and sends a message to discord for each new RSS item. The messages are sent in specific Discord channels that are created based on the RSS feed's name.

Preview of an RSS update message in Discord: (using [msfs-rss](https://github.com/evroon/msfs-rss))

![Preview](https://raw.githubusercontent.com/evroon/selfoss-discord/main/etc/preview.png)

## Usage
First, install openssl:
```
sudo apt-get install pkg-config libssl-dev
```

Create a bot in Discord with permissions to send manages and manage channels for the server you want to send messages to.
Create a file called `.env` with the following content:
```bash
DISCORD_TOKEN="your discord bot token"
DISCORD_SERVER_ID="the ID of your server/guild"
SELFOSS_BASE_URL="url where selfoss lives"
SELFOSS_USERNAME="selfoss username"
SELFOSS_PASSWORD="selfoss password"
```

Use systemd to run the update periodically, like this [service](https://github.com/evroon/concordia/blob/master/lib/systemd/system/selfoss-update.service) and [timer](https://github.com/evroon/concordia/blob/master/lib/systemd/system/selfoss-update.timer).
