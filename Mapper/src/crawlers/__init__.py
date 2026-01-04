# Mapper/src/crawlers/__init__.py
"""Crawler modules for Yonsei University data."""

from .departments import crawl_all_campuses, crawl_campus
from .boards import discover_boards, BoardDiscoveryResult

__all__ = [
    "crawl_all_campuses",
    "crawl_campus",
    "discover_boards",
    "BoardDiscoveryResult",
]
