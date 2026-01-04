# Mapper/src/main.py
"""Main entry point for the Mapper."""

import json
from pathlib import Path

from .config import (
    DATA_DIR,
    DEPARTMENTS_FILE,
    DEPARTMENTS_BOARDS_FILE,
    MANUAL_REVIEW_FILE,
)
from .crawlers import crawl_all_campuses, discover_boards


def ensure_data_dir():
    """Ensure the data directory exists."""
    DATA_DIR.mkdir(parents=True, exist_ok=True)


def save_json(path: Path, data: list | dict):
    """Save data to a JSON file."""
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=4)


def main():
    """
    Main function to crawl Yonsei departments and discover boards.

    Steps:
    1. Crawl all campuses to get departments
    2. Discover boards for each department
    3. Save results to JSON files
    """
    ensure_data_dir()

    # Step 1: Crawl departments
    print("=" * 60)
    print("Step 1: Crawling Yonsei departments...")
    print("=" * 60)

    campuses = crawl_all_campuses()
    if not campuses:
        print("Failed to crawl any campus. Exiting.")
        return 1

    # Convert to dictionaries for JSON serialization
    departments_data = [c.to_dict() for c in campuses]
    save_json(DEPARTMENTS_FILE, departments_data)
    print(f"\nSaved department data to {DEPARTMENTS_FILE}")

    # Step 2: Discover boards
    print("\n" + "=" * 60)
    print("Step 2: Discovering boards for each department...")
    print("=" * 60)

    manual_review_items = []

    for campus_data in departments_data:
        campus_name = campus_data["campus"]
        print(f"\n[{campus_name}]")

        for college in campus_data["colleges"]:
            for dept in college["departments"]:
                print(f"  {dept['name']}...")

                dept_info = {"campus": campus_name, "name": dept["name"]}
                result = discover_boards(dept_info, dept["url"])

                # Update department with boards
                dept["boards"] = [b.to_dict() for b in result.boards]

                if result.manual_review:
                    manual_review_items.append(result.manual_review.to_dict())

                if result.boards:
                    print(f"    Found {len(result.boards)} board(s)")

    # Save results
    save_json(DEPARTMENTS_BOARDS_FILE, departments_data)
    print(f"\nSaved departments with boards to {DEPARTMENTS_BOARDS_FILE}")

    save_json(MANUAL_REVIEW_FILE, manual_review_items)
    print(
        f"Saved {len(manual_review_items)} items needing manual review to {MANUAL_REVIEW_FILE}"
    )

    # Summary
    print("\n" + "=" * 60)
    print("Summary")
    print("=" * 60)

    total_depts = sum(
        len(dept)
        for campus in departments_data
        for college in campus["colleges"]
        for dept in [college["departments"]]
    )
    total_boards = sum(
        len(dept.get("boards", []))
        for campus in departments_data
        for college in campus["colleges"]
        for dept in college["departments"]
    )

    print(f"  Total departments: {total_depts}")
    print(f"  Total boards found: {total_boards}")
    print(f"  Needs manual review: {len(manual_review_items)}")

    return 0


if __name__ == "__main__":
    exit(main())
