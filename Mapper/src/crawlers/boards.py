# Mapper/src/crawlers/boards.py
"""Board discovery for department websites."""

import re
from dataclasses import dataclass, field
from urllib.parse import urljoin, urlparse

from bs4 import BeautifulSoup

from ..config import KEYWORD_MAP
from ..utils import get_soup, detect_cms_selectors


@dataclass
class Board:
    """Represents a notice board."""

    id: str
    name: str
    url: str
    row_selector: str
    title_selector: str
    date_selector: str
    attr_name: str

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "name": self.name,
            "url": self.url,
            "row_selector": self.row_selector,
            "title_selector": self.title_selector,
            "date_selector": self.date_selector,
            "attr_name": self.attr_name,
        }


@dataclass
class ManualReviewItem:
    """Represents a department that needs manual review."""

    campus: str
    name: str
    url: str
    reason: str

    def to_dict(self) -> dict:
        return {
            "campus": self.campus,
            "name": self.name,
            "url": self.url,
            "reason": self.reason,
        }


@dataclass
class BoardDiscoveryResult:
    """Result of board discovery for a department."""

    boards: list[Board] = field(default_factory=list)
    manual_review: ManualReviewItem | None = None


def _is_valid_board_link(text: str, href: str) -> bool:
    """Check if a link is likely a valid board link."""
    # Blacklist patterns that indicate article views, not board listings
    blacklist = ["articleNo", "article_no", "mode=view", "seq", "view.do", "board_seq"]
    if any(word in href for word in blacklist):
        return False

    # Long text is likely a notice title, not a board name
    if len(text) > 20:
        return False

    return True


def _find_sitemap(soup: BeautifulSoup, base_url: str) -> BeautifulSoup | None:
    """Try to find and fetch the sitemap page."""
    sitemap_link = soup.find("a", string=re.compile(r"사이트맵|Sitemap", re.I))

    if sitemap_link and sitemap_link.get("href"):
        sitemap_url = urljoin(base_url, sitemap_link["href"])
        sitemap_soup = get_soup(sitemap_url, timeout=5)
        if sitemap_soup:
            print(f"    Found sitemap: {sitemap_url}")
            return sitemap_soup

    return None


def _extract_boards_from_soup(
    soup: BeautifulSoup,
    base_url: str,
    default_selectors: dict | None,
) -> list[Board]:
    """Extract board information from a BeautifulSoup object."""
    boards = []
    seen_urls = set()
    id_counts = {}
    base_domain = urlparse(base_url).netloc.lower()

    for link in soup.find_all("a", href=True):
        text = link.get_text(strip=True)
        href = link["href"]

        if not _is_valid_board_link(text, href):
            continue

        full_url = urljoin(base_url, href)

        # Skip invalid URLs
        if full_url in seen_urls or "javascript" in href or "#" in href:
            continue

        # Skip external links
        link_domain = urlparse(full_url).netloc.lower()
        if link_domain != base_domain:
            continue

        # Match against keywords
        for keyword, meta in KEYWORD_MAP.items():
            if keyword not in text:
                continue

            # Try to detect CMS selectors
            selectors = detect_cms_selectors(soup, full_url) or default_selectors
            if not selectors:
                continue

            # Generate unique ID
            base_id = meta["id"]
            id_counts[base_id] = id_counts.get(base_id, 0) + 1
            final_id = (
                f"{base_id}_{id_counts[base_id]}" if id_counts[base_id] > 1 else base_id
            )

            boards.append(
                Board(
                    id=final_id,
                    name=text or meta["name"],
                    url=full_url,
                    **selectors,
                )
            )
            seen_urls.add(full_url)
            break

    return boards


def discover_boards(dept_info: dict, dept_url: str) -> BoardDiscoveryResult:
    """
    Discover useful boards from a department homepage.

    Args:
        dept_info: Dictionary with 'campus' and 'name' keys
        dept_url: Department homepage URL

    Returns:
        BoardDiscoveryResult containing found boards and optional manual review item
    """
    result = BoardDiscoveryResult()

    # Validate URL
    if dept_url == "NOT_FOUND" or not dept_url.startswith("http"):
        result.manual_review = ManualReviewItem(
            campus=dept_info["campus"],
            name=dept_info["name"],
            url=dept_url,
            reason="Homepage URL is invalid",
        )
        return result

    # Fetch department homepage
    soup = get_soup(dept_url, timeout=7)
    if not soup:
        result.manual_review = ManualReviewItem(
            campus=dept_info["campus"],
            name=dept_info["name"],
            url=dept_url,
            reason="Failed to fetch homepage",
        )
        return result

    print(f"    Accessed: {dept_url}")

    # Detect default CMS selectors
    default_selectors = detect_cms_selectors(soup, dept_url)

    # Try sitemap first
    sitemap_soup = _find_sitemap(soup, dept_url)
    if sitemap_soup:
        boards = _extract_boards_from_soup(sitemap_soup, dept_url, default_selectors)
        if boards:
            result.boards = boards
            return result
        print("    Sitemap yielded no results, falling back to homepage")

    # Fall back to homepage
    boards = _extract_boards_from_soup(soup, dept_url, default_selectors)
    result.boards = boards

    return result
