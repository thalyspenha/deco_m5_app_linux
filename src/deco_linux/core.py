"""Thin session wrapper around tplinkrouterc6u's Deco client.

The Deco only allows one logged-in admin session at a time, so every
operation logs in, runs, and logs out immediately to avoid kicking the
official app.
"""
import os
from contextlib import contextmanager

from dotenv import load_dotenv
from tplinkrouterc6u import TPLinkDecoClient

load_dotenv()


def _config() -> tuple[str, str]:
    host = os.environ.get("DECO_HOST", "192.168.68.1")
    password = os.environ.get("DECO_PASSWORD")
    if not password:
        raise SystemExit("DECO_PASSWORD not set (use .env or environment)")
    return host, password


@contextmanager
def session():
    host, password = _config()
    client = TPLinkDecoClient(f"http://{host}", password)
    client.authorize()
    try:
        yield client
    finally:
        client.logout()
