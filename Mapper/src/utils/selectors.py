# Mapper/src/utils/selectors.py
"""CMS detection and selector utilities."""

from bs4 import BeautifulSoup


def detect_cms_selectors(soup: BeautifulSoup, url: str) -> dict | None:
    """
    Detect CMS type and return appropriate selectors for scraping.

    Args:
        soup: BeautifulSoup object of the page
        url: URL of the page

    Returns:
        Dictionary with CSS selectors or None if CMS not recognized
    """
    html_str = str(soup).lower()

    # Standard Yonsei CMS
    if ".do" in url or "c-board-title" in html_str:
        return {
            "row_selector": "tr:has(a.c-board-title)",
            "title_selector": "a.c-board-title",
            "date_selector": "td:nth-last-child(1)",
            "attr_name": "href",
        }

    # XE board system
    if "xe-list-board" in html_str or "xe_board" in html_str:
        return {
            "row_selector": "li.xe-list-board-list--item:not(.xe-list-board-list--header)",
            "title_selector": "a.xe-list-board-list__title-link",
            "date_selector": ".xe-list-board-list__created_at",
            "attr_name": "href",
        }

    return None
