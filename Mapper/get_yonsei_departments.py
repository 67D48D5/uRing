# Mapper/get_yonsei_departments.py

from bs4 import BeautifulSoup

import requests
import json
import re


def crawl_single_campus(url, campus_name):
    headers = {
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
    }

    try:
        response = requests.get(url, headers=headers, timeout=10)
        response.raise_for_status()
        soup = BeautifulSoup(response.text, "html.parser")

        campus_data = {"campus": campus_name, "colleges": []}

        # Find main content area
        main = soup.find("main")
        if not main:
            print("Cannot find main content area")
            return campus_data

        # Extract all `h1` tags
        headers = main.find_all("h1")

        current_college = None
        college_pattern = re.compile(
            r"([가-힣]+대학)$"
        )  # Only those ending with "대학"

        for header in headers:
            text = header.get_text(strip=True)

            # Remove text related to faculty and homepage
            text = re.sub(r"교수진.*", "", text)
            text = re.sub(r"홈페이지.*", "", text).strip()

            if not text:
                continue

            # Find college if it ends with "대학"
            if college_pattern.search(text):
                college_name = text

                # Skip if the same college already exists
                if current_college and current_college["name"] == college_name:
                    continue

                current_college = {"name": college_name, "departments": []}
                campus_data["colleges"].append(current_college)

            # Find department page URL from "Homepage" link
            elif current_college and text and ("대학" not in text):
                # Remove duplicates
                if not any(d["name"] == text for d in current_college["departments"]):
                    # Find department page URL
                    dept_url = "NOT_FOUND"
                    parent = header.parent

                    # Look for "Homepage" link in the next sibling elements
                    for sibling in parent.find_next_siblings(
                        ["div", "p", "span"], limit=3
                    ):
                        # Find `<a>` tag containing "Homepage"
                        homepage_link = sibling.find(
                            "a", string=lambda x: x and "홈페이지" in x
                        )
                        if homepage_link:
                            href = homepage_link.get("href")
                            if href and not href.startswith("#"):
                                dept_url = href
                                break

                    # Generate department ID from subdomain
                    dept_id = f"yonsei_{text.lower().replace(' ', '_')}"
                    if dept_url:
                        if dept_url == "NOT_FOUND":
                            print(
                                "Homepage link found but URL is `NOT_FOUND` at department:",
                                text,
                            )

                        subdomain_match = re.match(
                            r"https?://([^.]+)\.yonsei\.ac\.kr", dept_url
                        )
                        if subdomain_match:
                            subdomain = subdomain_match.group(1).lower()
                            dept_id = f"yonsei_{subdomain}"

                    current_college["departments"].append(
                        {"id": dept_id, "name": text, "url": dept_url, "boards": []}
                    )

        return campus_data

    except Exception as e:
        print(f"Error occurred: {e}")
        import traceback

        traceback.print_exc()
        return None


def crawl_yonsei_departments():
    """Crawl both Sinchon and Mirae campuses"""
    campuses = [
        ("https://www.yonsei.ac.kr/sc/186/subview.do", "신촌캠퍼스"),
        ("https://mirae.yonsei.ac.kr/wj/1413/subview.do", "미래캠퍼스"),
    ]

    all_campus_data = []

    for url, campus_name in campuses:
        print(f"Crawling {campus_name}...")
        campus_data = crawl_single_campus(url, campus_name)
        if campus_data:
            all_campus_data.append(campus_data)
        else:
            print(f"Failed to crawl {campus_name}")
        print()

    return all_campus_data


if __name__ == "__main__":
    result = crawl_yonsei_departments()

    if result:
        # Save as JSON file
        with open("result/yonsei_departments.json", "w", encoding="utf-8") as f:
            json.dump(result, f, ensure_ascii=False, indent=4)

        print("Generated 'result/yonsei_departments.json' successfully.")
