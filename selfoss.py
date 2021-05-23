#!/usr/bin/env python3

import argparse
import json
import requests
import discord
import asyncio
import os
import datetime
from bs4 import BeautifulSoup
from dotenv import load_dotenv
from typing import Dict, List, Any, cast, Optional, TypeVar

time_format = "%Y-%m-%d %H:%M:%S"
max_message_chars = 2000

# Type cast helper
T = TypeVar('T')
def assert_type(arg: Optional[T]) -> T:
    assert arg is not None
    return arg


def get_tree(feed: str) -> List[Dict[str, Any]]:
    req = requests.get(feed)
    json_data = json.loads(req.content)
    return cast(List[Dict[str, Any]], json_data)


def new_items(json_dict: List[Dict[str, Any]], last_update_filename: str, oldpubdate: datetime.datetime) -> List[Any]:
    items = []
    latest = datetime.datetime.now() - datetime.timedelta(days=3*365)

    for item in json_dict:
        timestamp = datetime.datetime.strptime(item['datetime'][:-3], time_format)

        if timestamp > oldpubdate:
            items.append(item)

        latest = max(latest, timestamp)

    if items:
        with open(last_update_filename, 'w') as f:
            f.write(latest.strftime(time_format) + '\n')

    return items


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Checks selfoss instance and sends notification messages to discord.',
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        'selfoss',
        help='Selfoss intance to fetch data from.'
    )
    parser.add_argument(
        'last_update_filename',
        help='File to save last run time in.'
    )
    args = parser.parse_args()

    with open(args.last_update_filename, 'r') as handle:
        oldpubdate_formatted = handle.read().strip()
        oldpubdate = datetime.datetime.strptime(oldpubdate_formatted, time_format)

    root = get_tree(f"{args.selfoss}/items?updatedsince={oldpubdate_formatted}&items=200")
    items = new_items(root, args.last_update_filename, oldpubdate)

    if not items:
        raise SystemExit

    client = discord.Client()
    load_dotenv()
    token = os.getenv('DISCORD_TOKEN')
    server_id = os.getenv('DISCORD_SERVER_ID')


    async def my_background_task(items: List[Any]) -> None:
        await client.wait_until_ready()

        for item in items:
            source = item['sourcetitle']
            guild = client.get_guild(int(assert_type(server_id)))
            channel_name = source.lower().replace(' ', '-')
            channel = discord.utils.get(guild.channels, name=channel_name)

            if channel == None:
                channel = await guild.create_text_channel(channel_name)

            soup = BeautifulSoup(item['content'], features="html.parser")
            content = soup.get_text().replace('\n\n', '\n')

            if len(content) > max_message_chars:
                content = content[:max_message_chars-1]

            embed = discord.Embed(title=item['title'], url=item['link'], description=content)
            embed.set_author(name=source)
            embed.set_thumbnail(url=f"{args.selfoss}/favicons/{item['icon']}")
            embed.set_footer(text=item['datetime'][:-3])
            await channel.send(embed=embed)

        await client.close()

    @client.event
    async def on_ready() -> None:
        print(f'Logged in as: {client.user.name}')


    client.loop.create_task(my_background_task(items))
    client.run(token)
