use gitlab::api::{AsyncQuery, paged};
use crate::models::*;
use crate::api::*;
use regex::Regex;

impl App {
    pub async fn fetch_pipelines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pipelines: Vec<Pipeline> = if let Some(project) = &self.selected_project {
            let endpoint = ProjectPipelines { project: project.id };
            paged(endpoint, gitlab::api::Pagination::Limit(100)).query_async(&self.gitlab).await?
        } else {
            Vec::new()
        };

        self.pipelines = pipelines;
        Ok(())
    }

    pub async fn fetch_jobs(&mut self, project_id: u64, pipeline_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = ProjectPipelineJobs { project: project_id, pipeline: pipeline_id };
        let jobs: Vec<Job> = paged(endpoint, gitlab::api::Pagination::Limit(100)).query_async(&self.gitlab).await?;
        self.jobs = jobs;
        Ok(())
    }

    pub async fn fetch_job_log(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.is_loading = true;
        let endpoint = JobTrace { project: project_id, job: job_id };
        let data: Vec<u8> = gitlab::api::raw(endpoint).query_async(&self.gitlab).await?;
        let raw_log = String::from_utf8_lossy(&data).to_string();
        self.job_log_lines = self.parse_job_log(&raw_log);

        self.is_loading = false;
        Ok(())
    }

    fn parse_job_log(&self, raw_log: &str) -> Vec<LogLine> {
        let prefix_re = Regex::new(r"^([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]+Z)\s[0-9]{2}O\+?\s?").unwrap();
        let section_start_re = Regex::new(r"section_start:[0-9]+:([a-z_]+)\r?").unwrap();
        let section_end_re = Regex::new(r"section_end:[0-9]+:([a-z_]+)\r?").unwrap();
        let clear_line_re = Regex::new(r"\x1b\[0K").unwrap();

        let mut lines = Vec::new();
        let mut current_section: Option<String> = None;

        for line in raw_log.lines() {
            let mut timestamp = String::new();
            let mut content = line.to_string();

            if let Some(caps) = prefix_re.captures(line) {
                timestamp = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                content = prefix_re.replace(line, "").to_string();
            }

            content = clear_line_re.replace_all(&content, "").to_string();

            let is_section_start = if let Some(caps) = section_start_re.captures(&content) {
                current_section = caps.get(1).map(|m| m.as_str().to_string());
                content = section_start_re.replace(&content, "").to_string();
                true
            } else {
                false
            };

            let is_section_end = if let Some(_caps) = section_end_re.captures(&content) {
                content = section_end_re.replace(&content, "").to_string();
                current_section = None;
                true
            } else {
                false
            };

            // If it's just a section marker line with no content, it might be empty now
            // But we keep it to track the structure
            lines.push(LogLine {
                timestamp,
                content,
                section: current_section.clone(),
                is_section_start,
                is_section_end,
            });
        }
        lines
    }

    pub fn toggle_job_section(&mut self) {
        // We toggle the section of the line currently at the top of the scroll or some selection
        // Since we don't have a "selected line" for logs yet (just scroll), 
        // let's assume we toggle the section that the user is looking at or we add selection to logs.
        // For now, let's find the first section start near the scroll position.
        let mut visible_count = 0;
        let mut target_section = None;

        for line in &self.job_log_lines {
            let is_collapsed = line.section.as_ref().map(|s| self.collapsed_sections.contains(s)).unwrap_or(false);
            
            if visible_count == self.job_log_scroll {
                if let Some(s) = &line.section {
                    target_section = Some(s.clone());
                    break;
                }
            }

            if line.is_section_start {
                if is_collapsed {
                    // Skip internal lines if collapsed, but the start itself is visible
                    visible_count += 1;
                    continue; 
                }
            }

            if !is_collapsed || line.is_section_start {
                visible_count += 1;
            }
        }

        if let Some(s) = target_section {
            if self.collapsed_sections.contains(&s) {
                self.collapsed_sections.remove(&s);
            } else {
                self.collapsed_sections.insert(s);
            }
        }
    }

    pub fn get_visible_job_log_lines_count(&self) -> u16 {
        let mut count = 0;
        for line in &self.job_log_lines {
            let is_collapsed = line.section.as_ref().map(|s| self.collapsed_sections.contains(s)).unwrap_or(false);
            if is_collapsed && !line.is_section_start {
                continue;
            }
            if line.is_section_end && line.content.trim().is_empty() {
                continue;
            }
            count += 1;
        }
        count
    }

}