# Mapper/src/crawlers/departments.py
"""Department crawler for Yonsei University."""

import re
from dataclasses import dataclass, field

from ..config import CAMPUSES
from ..utils import get_soup


@dataclass
class Department:
    """Represents a university department."""

    id: str
    name: str
    url: str
    boards: list = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "name": self.name,
            "url": self.url,
            "boards": self.boards,
        }


@dataclass
class College:
    """Represents a college containing departments."""

    name: str
    departments: list[Department] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "name": self.name,
            "departments": [d.to_dict() for d in self.departments],
        }


@dataclass
class Campus:
    """Represents a university campus."""

    campus: str
    colleges: list[College] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "campus": self.campus,
            "colleges": [c.to_dict() for c in self.colleges],
        }


def _extract_department_url(header) -> str:
    """Extract department homepage URL from header's siblings."""
    parent = header.parent

    for sibling in parent.find_next_siblings(["div", "p", "span"], limit=3):
        homepage_link = sibling.find("a", string=lambda x: x and "홈페이지" in x)
        if homepage_link:
            href = homepage_link.get("href")
            if href and not href.startswith("#"):
                return href

    return "NOT_FOUND"


def _generate_department_id(name: str, url: str) -> str:
    """Generate a unique department ID."""
    if url and url != "NOT_FOUND":
        subdomain_match = re.match(r"https?://([^.]+)\.yonsei\.ac\.kr", url)
        if subdomain_match:
            return f"yonsei_{subdomain_match.group(1).lower()}"

    return f"yonsei_{name.lower().replace(' ', '_')}"


def crawl_campus(url: str, campus_name: str) -> Campus | None:
    """
    Crawl a single campus to extract colleges and departments.

    Args:
        url: Campus page URL
        campus_name: Name of the campus

    Returns:
        Campus object or None if crawling failed
    """
    soup = get_soup(url)
    if not soup:
        print(f"Failed to fetch campus page: {url}")
        return None

    main = soup.find("main")
    if not main:
        print(f"Cannot find main content area for {campus_name}")
        return None

    campus = Campus(campus=campus_name)
    current_college: College | None = None
    college_pattern = re.compile(r"([가-힣]+대학)$")

    for header in main.find_all("h1"):
        text = header.get_text(strip=True)

        # Clean up text
        text = re.sub(r"교수진.*", "", text)
        text = re.sub(r"홈페이지.*", "", text).strip()

        if not text:
            continue

        # Check if this is a college header
        if college_pattern.search(text):
            if current_college and current_college.name == text:
                continue
            current_college = College(name=text)
            campus.colleges.append(current_college)

        # Otherwise, it's a department
        elif current_college and "대학" not in text:
            # Skip duplicates
            if any(d.name == text for d in current_college.departments):
                continue

            dept_url = _extract_department_url(header)
            dept_id = _generate_department_id(text, dept_url)

            if dept_url == "NOT_FOUND":
                print(f"  Warning: No homepage URL found for {text}")

            current_college.departments.append(
                Department(id=dept_id, name=text, url=dept_url)
            )

    return campus


def crawl_all_campuses() -> list[Campus]:
    """
    Crawl all configured campuses.

    Returns:
        List of Campus objects
    """
    campuses = []

    for url, name in CAMPUSES:
        print(f"Crawling {name}...")
        campus = crawl_campus(url, name)
        if campus:
            campuses.append(campus)
            print(
                f"  Found {sum(len(c.departments) for c in campus.colleges)} departments"
            )
        else:
            print(f"  Failed to crawl {name}")

    return campuses
