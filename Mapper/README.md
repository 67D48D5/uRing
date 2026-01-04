# Mapper

Crawls Yonsei University website to discover announcement boards for each department.

## Project Structure

```shell
Mapper/
├── src/
│   ├── __init__.py
│   ├── main.py          # Entry point
│   ├── config.py        # Configuration & constants
│   ├── crawlers/
│   │   ├── __init__.py
│   │   ├── departments.py  # Campus/college/department crawler
│   │   └── boards.py       # Board discovery logic
│   └── utils/
│       ├── __init__.py
│       ├── http.py         # HTTP utilities
│       └── selectors.py    # CMS detection & selectors
├── data/                # Output directory
├── requirements.txt
└── README.md
```

## Setup

```bash
pip install -r requirements.txt
```

## Usage

Run as a module from the `Mapper` directory:

```bash
python -m src.main
```

## Output Files

Generated in the `data/` directory:

| File | Description |
| ---- | ----------- |
| `yonsei_departments.json` | All departments with basic info |
| `yonsei_departments_boards.json` | Departments with discovered boards |
| `manual_review_needed.json` | Departments requiring manual review |
