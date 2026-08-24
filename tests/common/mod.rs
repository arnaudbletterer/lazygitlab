#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TestGround {
    pub url: String,
    pub project: GroundProject,
    pub counts: GroundCounts,
    pub anchors: GroundAnchors,
    #[serde(default)]
    pub extras: GroundExtras,
    #[serde(default)]
    pub massive_mr: Option<MassiveMr>,
    #[serde(default)]
    pub last_seeded_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroundProject {
    pub id: u64,
    pub path_with_namespace: String,
    pub web_url: Option<String>,
    pub default_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroundCounts {
    pub labels: usize,
    pub branches: usize,
    pub issues: usize,
    pub merge_requests: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroundAnchors {
    pub first_label: String,
    pub first_branch: String,
    pub first_issue_iid: Option<u64>,
    pub first_mr_iid: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MassiveMr {
    pub iid: Option<u64>,
    pub branch: String,
    pub files_count: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GroundExtras {
    #[serde(default)]
    pub gitlab_ci_present: bool,
    #[serde(default)]
    pub pipeline_id: Option<u64>,
    #[serde(default)]
    pub board_id: Option<u64>,
    #[serde(default)]
    pub board_list_present: bool,
    #[serde(default)]
    pub deep_tree_root: Option<String>,
}

impl TestGround {
    pub fn load() -> &'static TestGround {
        static CELL: OnceLock<TestGround> = OnceLock::new();
        CELL.get_or_init(|| {
            let path = ground_file_path();
            let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "Failed to read test ground file at {}: {}. \
                     Run `python scripts/populate_test_ground.py` first.",
                    path.display(),
                    e
                )
            });
            serde_json::from_str(&raw).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse test ground file at {}: {}",
                    path.display(),
                    e
                )
            })
        })
    }
}

fn ground_file_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("LAZYGITLAB_TEST_GROUND") {
        return PathBuf::from(override_path);
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("data").join("test_ground.json")
}

pub fn test_ground() -> &'static TestGround {
    TestGround::load()
}

pub fn requires_massive_mr() -> &'static MassiveMr {
    let g = test_ground();
    g.massive_mr
        .as_ref()
        .filter(|m| m.iid.is_some())
        .unwrap_or_else(|| {
            panic!(
                "Test requires a massive MR, but data/test_ground.json shows massive_mr.iid=null. \
                 Re-run `python scripts/populate_test_ground.py --with-massive-mr`."
            )
        })
}

pub fn live_env_present() -> bool {
    let _ = dotenvy::dotenv();
    let token_ok = std::env::var("GITLAB_TOKEN")
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);
    let ground_ok = ground_file_path().exists();
    token_ok && ground_ok
}

pub fn skip_if_live_unavailable(test_name: &str) -> bool {
    if live_env_present() {
        return false;
    }
    eprintln!(
        "[{}] skipped: GITLAB_TOKEN unset or data/test_ground.json missing. \
         See scripts/populate_test_ground.py.",
        test_name
    );
    true
}
