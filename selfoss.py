#!/usr/bin/env python3

import argparse
import json
import requests
import discord
import asyncio
import os
import datetime
import pytz
from bs4 import BeautifulSoup
from dotenv import load_dotenv
from typing import Dict, List, Any, cast, Optional, TypeVar

timezone = pytz.timezone('Europe/Amsterdam')
time_format = "%Y-%m-%d %H:%M:%S%z"
time_format_display = "%d %b %Y at %H:%M"
max_message_chars = 2000

# Type cast helper
T = TypeVar('T')
def assert_type(arg: Optional[T]) -> T:
    assert arg is not None
    return arg


def parse_datetime(time_str: str) -> datetime.datetime:
    return datetime.datetime.strptime(time_str, time_format)


def utc_to_local(utc_dt: datetime.datetime) -> datetime.datetime:
    return utc_dt.astimezone(tz=timezone)


def get_tree(feed: str, disable_verify_ssl: bool) -> List[Dict[str, Any]]:
    req = requests.get(feed, verify=not disable_verify_ssl)
    json_data = json.loads(req.content)
    return cast(List[Dict[str, Any]], json_data)


def new_items(json_dict: List[Dict[str, Any]]) -> List[Any]:
    items = []

    for item in json_dict:
        item['timestamp'] = parse_datetime(item['datetime'] + "00")
        items.append(item)

    return items


def mark_as_read(args: Any, items: List[int]) -> None:
    url = f'{args.selfoss}/mark'

    # TODO: Send one request for all items, in Selfoss 2.18 an array of items does not work.
    for item in items:
        data = {
            'username': args.username,
            'password': args.password,
            'ids': item
        }
        response = requests.post(url, verify=not args.disable_verify_ssl, data=data)

        if not response.ok:
            print(response.status_code, response.content)
            break


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Checks selfoss instance and sends notification messages to discord.',
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        'selfoss',
        help='Selfoss instance URL to fetch data from.'
    )
    parser.add_argument(
        '--token',
        type=str,
        help='discord bot token'
    )
    parser.add_argument(
        '--server-id',
        type=str,
        help='discord server id'
    )
    parser.add_argument(
        '--disable-verify-ssl',
        action='store_true',
        help='disable verification of ssl certs when accessing selfoss'
    )
    parser.add_argument(
        '--username',
        type=str,
        help='selfoss username'
    )
    parser.add_argument(
        '--password',
        type=str,
        help='selfoss password'
    )
    args = parser.parse_args()

    items_url = f"{args.selfoss}/items?type=unread&items=200"
    print(f'Fetching items from: {items_url}')

    root = get_tree(items_url, args.disable_verify_ssl)
    items = new_items(root)

    if not items:
        raise SystemExit

    client = discord.Client()
    load_dotenv()

    os.environ['DISCORD_TOKEN'] = args.token
    os.environ['DISCORD_SERVER_ID'] = args.server_id
    token = os.getenv('DISCORD_TOKEN')
    server_id = os.getenv('DISCORD_SERVER_ID')

    print(f'Processing {len(items)} items.')

    async def my_background_task(items: List[Any]) -> None:
        await client.wait_until_ready()
        print(f'Sending {len(items)} messages.')

        for item in items:
            source = item['sourcetitle']
            guild = client.get_guild(int(assert_type(server_id)))
            channel_name = source.lower().replace(' ', '-')
            channel = discord.utils.get(guild.channels, name=channel_name)

            if channel == None:
                channel = await guild.create_text_channel(channel_name)

            soup = BeautifulSoup(item['content'], features="html.parser")
            content = soup.get_text().replace('\n\n', '\n')
            pub_datetime = utc_to_local(item['timestamp']).strftime(time_format_display)

            if len(content) > max_message_chars:
                content = content[:max_message_chars-1]

            embed = discord.Embed(title=item['title'], url=item['link'], description=content)
            embed.set_author(name=source)
            embed.set_thumbnail(url=f"{args.selfoss}/favicons/{item['icon']}")
            embed.set_footer(text=pub_datetime)
            await channel.send(embed=embed)

        await client.close()

    @client.event
    async def on_ready() -> None:
        print(f'Logged in as: {client.user.name}')


    client.loop.create_task(my_background_task(items))
    client.run(token)

    mark_as_read(
        args,
        [x['id'] for x in items],
    )
