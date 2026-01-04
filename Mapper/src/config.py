# Mapper/src/config.py
"""Configuration constants for the Mapper."""

from pathlib import Path

# Paths
ROOT_DIR = Path(__file__).parent.parent
DATA_DIR = ROOT_DIR / "data"

# Output files
DEPARTMENTS_FILE = DATA_DIR / "yonsei_departments.json"
DEPARTMENTS_BOARDS_FILE = DATA_DIR / "yonsei_departments_boards.json"
MANUAL_REVIEW_FILE = DATA_DIR / "manual_review_needed.json"

# Campus URLs
CAMPUSES = [
    ("https://www.yonsei.ac.kr/sc/186/subview.do", "신촌캠퍼스"),
    ("https://mirae.yonsei.ac.kr/wj/1413/subview.do", "미래캠퍼스"),
]

# Board keyword mappings
KEYWORD_MAP = {
    "학부공지": {"id": "academic", "name": "학사공지"},
    "대학원공지": {"id": "grad_notice", "name": "대학원공지"},
    "장학": {"id": "scholarship", "name": "장학공지"},
    "취업": {"id": "career", "name": "취업/진로"},
    "공지사항": {"id": "notice", "name": "일반공지"},
    "학사공지": {"id": "academic", "name": "학사공지"},
}

# HTTP settings
REQUEST_TIMEOUT = 10
USER_AGENT = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
