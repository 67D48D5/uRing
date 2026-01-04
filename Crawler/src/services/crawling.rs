// src/services/crawling.rs

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use tokio::sync::{Mutex, Semaphore};

use reqwest::Client;
use scraper::{Html, Selector};

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use crate::models::config::Config;
use crate::models::crawler::{BoardConfig, Campus, Notice};

use crate::utils::text_utils::{clean_date, clean_title};
use crate::utils::url::resolve_url;

#[async_trait]
pub trait HtmlFetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
}

pub struct ReqwestHtmlFetcher {
    client: Client,
}

impl ReqwestHtmlFetcher {
    pub fn new(config: &Config) -> Self {
        let client = Client::builder()
            .user_agent(&config.crawler.user_agent)
            .timeout(Duration::from_secs(config.crawler.timeout_secs))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }
}

#[async_trait]
impl HtmlFetcher for ReqwestHtmlFetcher {
    async fn fetch(&self, url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let html_content = self.client.get(url).send().await?.text().await?;
        Ok(html_content)
    }
}

pub struct Crawler<T: HtmlFetcher> {
    config: Arc<Config>,
    html_fetcher: Arc<T>,
}

struct BoardContext<'a> {
    campus: &'a str,
    college: &'a str,
    department_id: &'a str,
    department_name: &'a str,
    board: &'a BoardConfig,
    config: &'a Config,
}

impl<T: HtmlFetcher> Crawler<T> {
    pub fn new(config: Arc<Config>, html_fetcher: Arc<T>) -> Self {
        Self {
            config,
            html_fetcher,
        }
    }

    pub async fn fetch_all_notices(
        &self,
        campuses: &[Campus],
    ) -> Result<Vec<Notice>, Box<dyn Error>> {
        let all_notices = Arc::new(Mutex::new(Vec::new()));
        let semaphore = Arc::new(Semaphore::new(self.config.crawler.max_concurrent));
        let delay = Duration::from_millis(self.config.crawler.request_delay_ms);

        let mut boards_to_crawl = Vec::new();
        for campus in campuses {
            for (college_name, dept) in campus.all_departments() {
                for board in &dept.boards {
                    boards_to_crawl.push((
                        campus.campus.clone(),
                        college_name.to_string(),
                        dept.id.clone(),
                        dept.name.clone(),
                        board.clone(),
                    ));
                }
            }
        }

        let concurrency = if self.config.crawler.max_concurrent == 0 {
            1
        } else {
            self.config.crawler.max_concurrent
        };
        stream::iter(boards_to_crawl)
            .for_each_concurrent(
                concurrency,
                |(campus_name, college_name, dept_id, dept_name, board)| {
                    let html_fetcher = Arc::clone(&self.html_fetcher);
                    let config = Arc::clone(&self.config);
                    let all_notices = Arc::clone(&all_notices);
                    let semaphore = Arc::clone(&semaphore);

                    async move {
                        let _permit = semaphore
                            .acquire()
                            .await
                            .expect("Failed to acquire semaphore permit");

                        let context = BoardContext {
                            campus: &campus_name,
                            college: &college_name,
                            department_id: &dept_id,
                            department_name: &dept_name,
                            board: &board,
                            config: &config,
                        };

                        match self
                            .fetch_board_notices(html_fetcher.as_ref(), context)
                            .await
                        {
                            Ok(notices) => {
                                let mut all_notices_lock = all_notices.lock().await;
                                all_notices_lock.extend(notices);
                            }
                            Err(e) => {
                                eprintln!("Error fetching board {}: {}", board.name, e);
                            }
                        }

                        if config.crawler.request_delay_ms > 0 {
                            tokio::time::sleep(delay).await;
                        }
                    }
                },
            )
            .await;

        let notices = Arc::try_unwrap(all_notices)
            .expect("Mutex still has multiple owners")
            .into_inner();
        Ok(notices)
    }

    async fn fetch_board_notices(
        &self,
        html_fetcher: &dyn HtmlFetcher,
        context: BoardContext<'_>,
    ) -> Result<Vec<Notice>, Box<dyn Error + Send + Sync>> {
        let html_content = html_fetcher.fetch(&context.board.url).await?;
        let document = Html::parse_document(&html_content);

        let row_sel = Selector::parse(&context.board.row_selector)
            .map_err(|e| format!("Invalid row selector: {}", e))?;
        let title_sel = Selector::parse(&context.board.title_selector)
            .map_err(|e| format!("Invalid title selector: {}", e))?;
        let date_sel = Selector::parse(&context.board.date_selector)
            .map_err(|e| format!("Invalid date selector: {}", e))?;
        let base_url = url::Url::parse(&context.board.url)?;

        let mut notices = Vec::new();

        for row in document.select(&row_sel) {
            let title_elem = row.select(&title_sel).next();
            let date_elem = row.select(&date_sel).next();

            if let (Some(t), Some(d)) = (title_elem, date_elem) {
                let title = clean_title(
                    &t.text().collect::<Vec<_>>().join(" "),
                    &context.config.cleaning,
                );
                let date = clean_date(
                    &d.text().collect::<Vec<_>>().join(" "),
                    &context.config.cleaning,
                );

                let link_elem = if let Some(ref link_sel_str) = context.board.link_selector {
                    let link_sel = Selector::parse(link_sel_str)
                        .map_err(|e| format!("Invalid link selector: {}", e))?;
                    row.select(&link_sel).next()
                } else {
                    Some(t)
                };

                let raw_link = link_elem
                    .and_then(|e| e.value().attr(&context.board.attr_name))
                    .unwrap_or("");
                let link = resolve_url(&base_url, raw_link);

                if !title.is_empty() {
                    notices.push(Notice {
                        campus: context.campus.to_string(),
                        college: context.college.to_string(),
                        department_id: context.department_id.to_string(),
                        department_name: context.department_name.to_string(),
                        board_id: context.board.id.clone(),
                        board_name: context.board.name.clone(),
                        title,
                        date,
                        link,
                    });
                }
            }
        }
        Ok(notices)
    }
}
