# uRing Crawler

**Type:** Serverless · Event-Driven · Thick Client
**Core Philosophy:** "Dumb Backend, Smart Client"

## Executive Summary

uRing is a lightweight, high-reliability system designed to detect and distribute university announcements in near real-time. It adheres strictly to a **"Push Changes, Pull Data"** model.

The backend acts solely as a **change detection engine**—it does not store HTML bodies, manage user accounts, or perform keyword filtering. By offloading logic to the client and leveraging a **"Compare-and-Swap"** strategy with **FIFO enforcement**, the system achieves extreme cost efficiency while guaranteeing data correctness and idempotency.

## Design Goals

1. **Minimal Backend Intelligence:** The server only detects *changes*. Interpretation (filtering, read status) is client-side.
2. **Concurrency Safety:** Department-level serialization prevents race conditions during updates.
3. **Idempotency:** "Effectively once" delivery guarantees for push notifications.
4. **Cache Efficiency:** Heavy reliance on CDNs (CloudFront) and ETags to minimize S3 egress.
5. **Revision Awareness:** Capable of tracking updates to existing notices (e.g., title changes, pinned status).

## High-Level Architecture

The system decouples **Data Ingestion (Pull)** from **Notification Delivery (Push)**, using SQS FIFO queues to enforce order and isolation.

```mermaid
graph TD
    subgraph Scheduler
    EB[EventBridge\n(Every 10 min)] -->|Trigger| Orch[Orchestrator Lambda]
    end

    subgraph Ingestion Pipeline [Ordered Ingestion]
    Orch -->|Dispatch Jobs| SQS_W[SQS FIFO: Work Queue]
    SQS_W -.->|Group: dept_id| Worker[Worker Lambda]
    Worker -->|1. Fetch & Hash| Web[University Website]
    Worker -->|2. Read State| S3[(S3 Bucket)]
    end

    subgraph Storage Strategy
    Worker -->|3. Dual Write| S3
    S3 -->|Mutable/Hot| Hot[latest.json]
    S3 -->|Append-Only/Cold| Cold[archive/202601.jsonl]
    end

    subgraph Notification Pipeline [Idempotent Delivery]
    Worker -->|4. If Change Detected| SQS_N[SQS FIFO: Notify Queue]
    SQS_N -->|Trigger| Notifier[Notifier Lambda]
    Notifier -->|5. Dedup & Send| FCM[Firebase Cloud Messaging]
    end

    subgraph Distribution
    S3 -->|Cache & ETag| CF[CloudFront CDN]
    CF -->|Pull Data| App[Mobile Client]
    FCM -->|Push Signal| App
    end

```

## Data Storage Strategy

We employ a **Hot/Cold separation** strategy. This ensures O(1) read performance for the app dashboard while maintaining a complete, append-only history for auditing and pagination.

### Directory Structure

```text
S3/
 ├── config/
 │    └── sitemap.json           # Target configurations
 └── data/
      └── {department_id}/       # e.g., yonsei_cs
           ├── latest.json       # [Hot] Top 30 Items + Meta (Mutable)
           └── archive/          # [Cold] Historical Data
                ├── 202601.jsonl # Jan 2026 (Append-only Log)
                └── ...

```

### Data Schema (Revision-Aware)

To handle updates (e.g., a typo correction in a title), we track revisions and hashes.

**File:** `yonsei_cs/latest.json`

```json
{
  "meta": {
    "dept_name": "Computer Science",
    "base_url": "https://cs.yonsei.ac.kr",
    "last_updated": "2026-01-22T10:00:00Z"
  },
  "items": [
    {
      "id": "notice_12345",
      "title": "Scholarship Announcement (Revised)",
      "date": "2026-01-22",
      "category": "Undergraduate",
      "link": "/board/view?id=12345",
      "is_pinned": false,
      "revision": 2,
      "hash": "a1b2c3d4...", 
      "first_seen": "2026-01-22T09:00:00Z"
    }
    // ... Max 30 items
  ]
}

```

## Operational Workflows

### Ingestion (The Worker)

**Concurrency Control:** The Work Queue is **SQS FIFO**. We set `MessageGroupId = department_id`. This guarantees that multiple Lambdas can run in parallel for *different* departments, but *never* for the same department, eliminating race conditions on `latest.json`.

1. **Fetch & Hash:** Scrape Page 1. Generate a hash `sha1(title|date|category|link)` for every item.
2. **Load State:** Read the existing `latest.json` from S3.
3. **Diff Logic:**

* *New ID:* Treat as **New Item**.
* *Existing ID + New Hash:* Treat as **Update** (Increment revision).
* *Existing ID + Same Hash:* Ignore.

1. **Dual Write:**

* **Hot:** Prepend new/updated items to `latest.json`. Truncate to 30 items.
* **Cold:** Append the item to `archive/YYYYMM.jsonl` (JSON Lines format for efficient appending).

### Notification (The Notifier)

**Idempotency Strategy:** To prevent duplicate alerts (a common issue with distributed systems), we implement deduplication.

1. **Trigger:** Worker sends a payload to the Notification Queue only for meaningful changes.
2. **Deduplication:**

* Ideally handled via a **DynamoDB TTL table** acting as a lock.
* Key: `idempotency_key = dept_id + notice_id + revision`
* If key exists: Skip. If not: Write key (30 min TTL) and proceed.

1. **Send:** Dispatch a **Data Message** to the FCM Topic (e.g., `topic: yonsei_cs`).

## Client-Side Strategy (Thick Client)

The backend provides raw data; the client application owns the user experience.

| Feature | Implementation Logic |
| --- | --- |
| **Subscription** | App subscribes directly to FCM Topics (e.g., `subscribeToTopic('yonsei_cs')`). No backend user DB. |
| **Notification Display** | FCM sends a **Data-only** payload. The App wakes up in the background, checks the `title` against the user's **Local Keyword List** (e.g., "Scholarship"), and decides whether to show a system banner or suppress it. |
| **Pagination** | 1. Load `latest.json`. 2. On scroll end, calculate previous month (`202512`) and fetch `archive/202512.jsonl`. |
| **Caching (ETag)** | The App sends `If-None-Match` headers when fetching `latest.json`. CloudFront returns `304 Not Modified` if nothing changed, saving bandwidth. |
| **Read Status** | Stored locally (SQLite/SharedPreferences). |

## Infrastructure & Safety

* **Compute:** AWS Lambda (Python/Node.js).
* **Queues:** AWS SQS **FIFO** (Ensures ordering and exactly-once processing).
* **Storage:** AWS S3 (Standard).
* `latest.json`: **TTL 5 min** (via Cache-Control headers).
* `archive/*`: **Long-term storage**.

* **CDN:** Amazon CloudFront (Edge caching).
* **Observability:**
* Logs must include `department_id`, `new_items_count`, and `revision`.
* Alarms on `DLQ Depth > 0` (Worker failures).

### Architectural Summary: Why this works

1. **Cost:** We only pay for compute when the scheduler runs, and 99% of requests hit the CloudFront cache (cheap) rather than Lambda (expensive).
2. **Scale:** Adding a new university simply means adding a config entry. The FIFO queue automatically handles the load balancing.
3. **Reliability:** Even if the scraper crashes, the FIFO queue ensures the next attempt retries correctly without corrupting the JSON state.
