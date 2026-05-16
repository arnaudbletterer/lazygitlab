use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn trigger_global_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let query = self.search_query.clone();
        if query.len() < 3 { 
            self.search_results.clear();
            self.search_state.select(None);
            self.status_message = Some("Type at least 3 characters...".into());
            let _ = self.update_preview().await;
            return Ok(()); 
        }

        let current_search_id = self.search_id;
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let member_ids = self.member_project_ids.clone();

        self.search_results.clear();
        self.search_state.select(None);
        let _ = tx.send(DataUpdate::Loading(true));
        self.status_message = Some(format!("Aggressive search for '{}'...", query));

        tokio::spawn(async move {
            let scopes = vec!["projects", "issues", "merge_requests"];
            let mut handles = Vec::new();

            for scope in scopes {
                let query_inner = query.clone();
                let client_inner = client.clone();
                let tx_inner = tx.clone();
                let member_ids_inner = member_ids.clone();
                
                // Strategy 1: Global Search (Fast, Elasticsearch-backed)
                let s1_scope = scope.to_string();
                let s1_query = query_inner.clone();
                let s1_client = client_inner.clone();
                let s1_tx = tx_inner.clone();
                let s1_member_ids = member_ids_inner.clone();
                handles.push(tokio::spawn(async move {
                    let ep = GlobalSearch { scope: s1_scope.clone(), search: s1_query, per_page: Some(100), page: Some(1) };
                    let res: Result<Vec<serde_json::Value>, _> = ep.query_async(&s1_client).await;
                    if let Ok(data) = res {
                        let mut results = Vec::new();
                        for val in data {
                            match s1_scope.as_str() {
                                "projects" => {
                                    if let Ok(p) = serde_json::from_value::<Project>(val.clone()) {
                                        if s1_member_ids.is_empty() || s1_member_ids.contains(&p.id) { results.push(SearchResult::Project(p)); }
                                    }
                                }
                                "issues" => {
                                    match serde_json::from_value::<Issue>(val.clone()) {
                                        Ok(i) => { if s1_member_ids.is_empty() || s1_member_ids.contains(&i.project_id) { results.push(SearchResult::Issue(i)); } }
                                        Err(e) => tracing::error!("Deserialization failure for Global Issue: {}. Data: {}", e, val),
                                    }
                                }
                                "merge_requests" => {
                                    match serde_json::from_value::<MergeRequest>(val.clone()) {
                                        Ok(m) => { if s1_member_ids.is_empty() || s1_member_ids.contains(&m.project_id) { results.push(SearchResult::MergeRequest(m)); } }
                                        Err(e) => tracing::error!("Deserialization failure for Global MR: {}. Data: {}", e, val),
                                    }
                                }
                                _ => {}
                            }
                        }
                        if !results.is_empty() { let _ = s1_tx.send(DataUpdate::AppendSearchResults { data: results, search_id: current_search_id }); }
                    }
                }));

                // Strategy 2: Direct Scoped Search (Accurate, Membership-restricted)
                let s2_scope = scope.to_string();
                let s2_query = query_inner.clone();
                let s2_client = client_inner.clone();
                let s2_tx = tx_inner.clone();
                handles.push(tokio::spawn(async move {
                    let timeout = std::time::Duration::from_secs(12);
                    let res: Result<Vec<serde_json::Value>, String> = match s2_scope.as_str() {
                        "projects" => {
                            let ep = PagedProjects { membership: true, search: Some(s2_query), page: 1, per_page: 50, simple: Some(true), archived: Some(false), order_by: None };
                            tokio::time::timeout(timeout, ep.query_async(&s2_client)).await.map_err(|_| "T".into()).and_then(|r| r.map_err(|e| e.to_string()))
                        }
                        "issues" => {
                            let ep = GlobalIssues { scope: None, state: None, membership: Some(true), search: Some(s2_query), author_id: None, assignee_id: None, page: None, per_page: Some(50), order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(false) };
                            tokio::time::timeout(timeout, ep.query_async(&s2_client)).await.map_err(|_| "T".into()).and_then(|r| r.map_err(|e| e.to_string()))
                        }
                        "merge_requests" => {
                            let ep = GlobalMergeRequests { scope: None, state: None, membership: Some(true), search: Some(s2_query), author_id: None, assignee_id: None, reviewer_id: None, page: None, per_page: Some(50), order_by: Some("created_at".into()), sort: Some("desc".into()), with_labels_details: Some(false) };
                            tokio::time::timeout(timeout, ep.query_async(&s2_client)).await.map_err(|_| "T".into()).and_then(|r| r.map_err(|e| e.to_string()))
                        }
                        _ => return,
                    };
                    if let Ok(data) = res {
                        let mut results = Vec::new();
                        for val in data {
                            match s2_scope.as_str() {
                                "projects" => { if let Ok(p) = serde_json::from_value::<Project>(val) { results.push(SearchResult::Project(p)); } }
                                "issues" => { if let Ok(i) = serde_json::from_value::<Issue>(val) { results.push(SearchResult::Issue(i)); } }
                                "merge_requests" => { if let Ok(m) = serde_json::from_value::<MergeRequest>(val) { results.push(SearchResult::MergeRequest(m)); } }
                                _ => {}
                            }
                        }
                        if !results.is_empty() { let _ = s2_tx.send(DataUpdate::AppendSearchResults { data: results, search_id: current_search_id }); }
                    }
                }));
            }

            for handle in handles { let _ = handle.await; }
            let _ = tx.send(DataUpdate::Loading(false));
            let _ = tx.send(DataUpdate::TriggerRefresh); 
        });

        Ok(())
    }
}
