# selfoss-discord

Send RSS updates from Selfoss to Discord.

Fetches new RSS items from `/items` and sends a message to discord for each new RSS item. The messages are sent in specific Discord channels that are created based on the RSS feed's name.

## Usage
You can update selfoss and send messages to discord hourly by opening crontab (`sudo crontab -e`) adding the following line:
```bash
0 * * * * sudo -Hu www-data php /var/www/selfoss/cliupdate.php && sudo -Hu <username> python3 /path/to/selfoss.py https://selfoss.domain.com /path/to/last-update
```

But using a systemd service and timer would be neater.
