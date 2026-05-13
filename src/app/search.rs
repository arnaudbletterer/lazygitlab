use gitlab::api::{AsyncQuery, paged};
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn trigger_global_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let query = self.search_query.clone();
        if query.is_empty() { 
            self.search_results.clear();
            return Ok(()); 
        }

        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));

        tokio::spawn(async move {
            let mut all_results = Vec::new();

            // Scopes to search: Mixed results as requested in roadmap
            let scopes = vec!["projects", "issues", "merge_requests"];

            for scope in scopes {
                let ep = GlobalSearch { scope: scope.to_string(), search: query.clone() };
                // Using a smaller limit for rapid mixed results
                let res: Result<Result<Vec<serde_json::Value>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    paged(ep, gitlab::api::Pagination::Limit(20)).query_async(&client)
                ).await;

                if let Ok(Ok(data)) = res {
                    for val in data {
                        match scope {
                            "projects" => {
                                if let Ok(p) = serde_json::from_value::<Project>(val) {
                                    all_results.push(SearchResult::Project(p));
                                }
                            }
                            "issues" => {
                                if let Ok(i) = serde_json::from_value::<Issue>(val) {
                                    // Basic validation/enrichment if needed
                                    all_results.push(SearchResult::Issue(i));
                                }
                            }
                            "merge_requests" => {
                                if let Ok(m) = serde_json::from_value::<MergeRequest>(val) {
                                    all_results.push(SearchResult::MergeRequest(m));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            let _ = tx.send(DataUpdate::SearchResults { data: all_results });
            let _ = tx.send(DataUpdate::Loading(false));
        });

        Ok(())
    }
}
