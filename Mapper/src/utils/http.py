# Mapper/src/utils/http.py
"""HTTP utility functions."""

import requests
from bs4 import BeautifulSoup

from ..config import USER_AGENT, REQUEST_TIMEOUT


def get_headers() -> dict:
    """Get default HTTP headers."""
    return {"User-Agent": USER_AGENT}


def fetch_page(url: str, timeout: int = REQUEST_TIMEOUT) -> requests.Response | None:
    """
    Fetch a page and return the response.

    Args:
        url: The URL to fetch
        timeout: Request timeout in seconds

    Returns:
        Response object or None if request failed
    """
    try:
        response = requests.get(url, headers=get_headers(), timeout=timeout)
        response.raise_for_status()
        return response
    except requests.RequestException:
        return None


def get_soup(url: str, timeout: int = REQUEST_TIMEOUT) -> BeautifulSoup | None:
    """
    Fetch a page and return a BeautifulSoup object.

    Args:
        url: The URL to fetch
        timeout: Request timeout in seconds

    Returns:
        BeautifulSoup object or None if request failed
    """
    response = fetch_page(url, timeout)
    if response:
        return BeautifulSoup(response.text, "html.parser")
    return None
