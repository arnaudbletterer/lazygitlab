use crate::api::*;
use crate::models::*;
use gitlab::api::AsyncQuery;

impl App {
    pub async fn fetch_mr_diffs(
        &mut self,
        project_id: u64,
        mr_iid: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.is_fetching_diffs = true;
        self.status_message = Some(format!("Fetching diffs for MR !{}...", mr_iid));
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = MergeRequestDiffs {
                project: project_id,
                merge_request: mr_iid,
            };
            let res: Result<Result<Vec<MRDiff>, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                gitlab::api::paged(endpoint, gitlab::api::Pagination::All).query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(data)) => {
                    let _ = tx.send(DataUpdate::Diffs {
                        project_id,
                        iid: mr_iid,
                        data,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Diffs fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Diffs fetch timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_single_file_diff(
        &mut self,
        project_id: u64,
        iid: u64,
        old_path: String,
        new_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.status_message = Some(format!("Fetching diff for {}...", new_path));
        let client = self.gitlab.clone();
        let tx = self.tx.clone();

        let mr = if let Some(mr) = &self.selected_mr {
            mr.clone()
        } else {
            return Ok(());
        };

        let _ = tx.send(DataUpdate::Loading(true));
        let sem = self.diff_fetch_semaphore.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let mut current_mr = mr;
            if current_mr.diff_refs.is_none() {
                let mr_endpoint = crate::api::SingleMergeRequest {
                    project: project_id,
                    merge_request: iid,
                };
                if let Ok(full_mr) = mr_endpoint.query_async(&client).await {
                    current_mr = full_mr;
                    let _ = tx.send(DataUpdate::MRDetail(current_mr.clone()));
                }
            }

            let refs = if let Some(r) = &current_mr.diff_refs {
                r.clone()
            } else {
                let _ = tx.send(DataUpdate::Error(
                    "Could not retrieve MR diff refs (SHAs)".into(),
                ));
                let _ = tx.send(DataUpdate::Loading(false));
                return;
            };

            let base_sha = refs.base_sha;
            let head_sha = refs.head_sha;

            let old_endpoint = gitlab::api::projects::repository::files::File::builder()
                .project(project_id)
                .file_path(old_path.clone())
                .ref_(base_sha)
                .build()
                .unwrap();
            let new_endpoint = gitlab::api::projects::repository::files::File::builder()
                .project(project_id)
                .file_path(new_path.clone())
                .ref_(head_sha)
                .build()
                .unwrap();

            let old_res: Result<crate::models::RepositoryFile, _> =
                gitlab::api::AsyncQuery::query_async(&old_endpoint, &client).await;
            let new_res: Result<crate::models::RepositoryFile, _> =
                gitlab::api::AsyncQuery::query_async(&new_endpoint, &client).await;

            use base64::{Engine as _, engine::general_purpose};

            let old_text = if let Ok(f) = old_res {
                let clean = f.content.replace(['\n', '\r'], "");
                if let Ok(dec) = general_purpose::STANDARD.decode(&clean) {
                    String::from_utf8_lossy(&dec).into_owned()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let new_text = if let Ok(f) = new_res {
                let clean = f.content.replace(['\n', '\r'], "");
                if let Ok(dec) = general_purpose::STANDARD.decode(&clean) {
                    String::from_utf8_lossy(&dec).into_owned()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let diff = similar::TextDiff::from_lines(&old_text, &new_text)
                .unified_diff()
                .to_string();

            let _ = tx.send(DataUpdate::SingleDiffFetched {
                project_id,
                iid,
                old_path,
                new_path,
                diff,
            });
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_all_missing_diffs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid) = if let Some(mr) = &self.selected_mr {
            (mr.project_id, mr.iid)
        } else {
            return Ok(());
        };
        let missing: Vec<(String, String)> = self
            .mr_diffs
            .iter()
            .filter(|d| d.collapsed && d.diff.is_empty())
            .map(|d| (d.old_path.clone(), d.new_path.clone()))
            .collect();

        if missing.is_empty() {
            return Ok(());
        }

        for (old, new) in missing {
            let _ = self.fetch_single_file_diff(pid, iid, old, new).await;
        }
        Ok(())
    }
}

pub fn parse_diff_context(
    diff: &str,
    new_line: Option<u64>,
    old_line: Option<u64>,
) -> Option<Vec<(u64, String, LineType, bool)>> {
    let target = new_line.or(old_line)?;
    let mut all_diff_lines = Vec::new();
    let mut current_old_line = 0;
    let mut current_new_line = 0;
    for line in diff.lines() {
        if line.starts_with("@@") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let old_info = parts[1].trim_start_matches('-');
                let new_info = parts[2].trim_start_matches('+');
                current_old_line = old_info
                    .split(',')
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                current_new_line = new_info
                    .split(',')
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            }
            continue;
        }
        let (ltype, ln) = if line.starts_with('+') {
            let l = (LineType::Addition, current_new_line);
            current_new_line += 1;
            l
        } else if line.starts_with('-') {
            let l = (LineType::Removal, current_old_line);
            current_old_line += 1;
            l
        } else {
            let l = (
                LineType::Unchanged,
                if new_line.is_some() {
                    current_new_line
                } else {
                    current_old_line
                },
            );
            current_old_line += 1;
            current_new_line += 1;
            l
        };
        all_diff_lines.push((ln, line.to_string(), ltype));
    }
    if let Some(target_idx) = all_diff_lines.iter().position(|(ln, _, lt)| {
        *ln == target
            && ((new_line.is_some() && *lt == LineType::Addition)
                || (old_line.is_some() && *lt == LineType::Removal)
                || (*lt == LineType::Unchanged))
    }) {
        let start = target_idx.saturating_sub(3);
        let end = (target_idx + 4).min(all_diff_lines.len());
        let mut context = Vec::new();
        for (i, (ln, content, ltype)) in all_diff_lines[start..end].iter().enumerate() {
            let is_target = (start + i) == target_idx;
            let clean_content =
                if content.starts_with('+') || content.starts_with('-') || content.starts_with(' ')
                {
                    &content[1..]
                } else {
                    content
                };
            context.push((*ln, clean_content.to_string(), ltype.clone(), is_target));
        }
        Some(context)
    } else {
        None
    }
}
