//! Crew status HUD widget for the AGNOS desktop environment.
//!
//! Surfaces active Agnostic crews with real-time status by polling the
//! `agnostic_list_crews` MCP tool via the daimon MCP call API at
//! `http://localhost:8090/v1/mcp/tools/call`.

use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use uuid::Uuid;

/// Status of a single Agnostic crew.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CrewRunStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
    #[default]
    Unknown,
}

impl From<&str> for CrewRunStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "idle" => Self::Idle,
            "running" | "active" => Self::Running,
            "paused" => Self::Paused,
            "completed" | "done" | "finished" => Self::Completed,
            "failed" | "error" => Self::Failed,
            _ => Self::Unknown,
        }
    }
}

/// Snapshot of a single crew as displayed in the HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewEntry {
    /// Crew identifier (name or UUID string from Agnostic).
    pub crew_id: String,
    /// Human-readable crew name.
    pub name: String,
    /// Current run status.
    pub status: CrewRunStatus,
    /// Number of agents assigned to this crew.
    pub agent_count: usize,
    /// Current task description, if any.
    pub current_task: Option<String>,
    /// Progress 0.0–1.0, if reported.
    pub progress: Option<f32>,
    /// Wall-clock time when this snapshot was recorded.
    pub last_updated: DateTime<Utc>,
}

impl Default for CrewEntry {
    fn default() -> Self {
        Self {
            crew_id: Uuid::new_v4().to_string(),
            name: String::new(),
            status: CrewRunStatus::Unknown,
            agent_count: 0,
            current_task: None,
            progress: None,
            last_updated: Utc::now(),
        }
    }
}

/// Render output produced by [`CrewStatusWidget::render`].
///
/// The compositor consumes this descriptor to draw the widget; no actual pixel
/// operations happen here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewStatusRenderData {
    /// Ordered list of crew snapshots to display.
    pub crews: Vec<CrewEntry>,
    /// Number of crews currently in a running state.
    pub active_count: usize,
    /// Whether the last `update()` call succeeded.
    pub last_fetch_ok: bool,
    /// Timestamp of the most recent successful fetch.
    pub last_fetch_at: Option<DateTime<Utc>>,
}

/// HUD widget that tracks active Agnostic crew status.
///
/// # Example
/// ```rust,no_run
/// use aethersafha::hud::crew_status::CrewStatusWidget;
/// use std::time::Duration;
///
/// let widget = CrewStatusWidget::new();
/// let handle = widget.start_polling(Duration::from_secs(5));
/// let render = widget.render();
/// println!("Active crews: {}", render.active_count);
/// ```
#[derive(Debug, Clone)]
pub struct CrewStatusWidget {
    crews: Arc<RwLock<Vec<CrewEntry>>>,
    last_fetch_ok: Arc<RwLock<bool>>,
    last_fetch_at: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl Default for CrewStatusWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl CrewStatusWidget {
    /// Create a new widget with empty state.
    pub fn new() -> Self {
        Self {
            crews: Arc::new(RwLock::new(Vec::new())),
            last_fetch_ok: Arc::new(RwLock::new(false)),
            last_fetch_at: Arc::new(RwLock::new(None)),
        }
    }

    /// Return display-ready data for the compositor to render.
    pub fn render(&self) -> CrewStatusRenderData {
        let crews = self.crews.read().unwrap_or_else(|e| e.into_inner()).clone();
        let active_count = crews
            .iter()
            .filter(|c| c.status == CrewRunStatus::Running)
            .count();
        let last_fetch_ok = *self.last_fetch_ok.read().unwrap_or_else(|e| e.into_inner());
        let last_fetch_at = *self.last_fetch_at.read().unwrap_or_else(|e| e.into_inner());

        CrewStatusRenderData {
            crews,
            active_count,
            last_fetch_ok,
            last_fetch_at,
        }
    }

    /// Fetch fresh crew data from the daimon MCP endpoint and update state.
    ///
    /// Calls `POST /v1/mcp/tools/call` with tool `agnostic_list_crews`.
    pub async fn update(&self) {
        match Self::fetch_crews().await {
            Ok(entries) => {
                let mut crews = self.crews.write().unwrap_or_else(|e| e.into_inner());
                *crews = entries;
                *self
                    .last_fetch_ok
                    .write()
                    .unwrap_or_else(|e| e.into_inner()) = true;
                *self
                    .last_fetch_at
                    .write()
                    .unwrap_or_else(|e| e.into_inner()) = Some(Utc::now());
                debug!("Crew status HUD: {} crew(s) loaded", crews.len());
            }
            Err(e) => {
                warn!("Crew status HUD update failed: {}", e);
                *self
                    .last_fetch_ok
                    .write()
                    .unwrap_or_else(|e| e.into_inner()) = false;
            }
        }
    }

    /// Spawn a background task that calls [`update`](Self::update) on `interval`.
    pub fn start_polling(&self, interval: std::time::Duration) -> tokio::task::JoinHandle<()> {
        let widget = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                widget.update().await;
            }
        })
    }

    /// Directly overwrite the crew list (useful for tests or manual injection).
    pub fn set_crews(&self, entries: Vec<CrewEntry>) {
        let mut crews = self.crews.write().unwrap_or_else(|e| e.into_inner());
        *crews = entries;
        *self
            .last_fetch_ok
            .write()
            .unwrap_or_else(|e| e.into_inner()) = true;
        *self
            .last_fetch_at
            .write()
            .unwrap_or_else(|e| e.into_inner()) = Some(Utc::now());
    }

    // --- private helpers ---

    async fn fetch_crews() -> Result<Vec<CrewEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let body = serde_json::json!({
            "name": "agnostic_list_crews",
            "arguments": {}
        });

        let resp = client
            .post("http://localhost:8090/v1/mcp/tools/call")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("MCP call returned {}", resp.status()).into());
        }

        let json: serde_json::Value = resp.json().await?;
        Self::parse_crews(&json)
    }

    fn parse_crews(
        json: &serde_json::Value,
    ) -> Result<Vec<CrewEntry>, Box<dyn std::error::Error + Send + Sync>> {
        // The MCP result embeds the tool output in `result.content[0].text` (JSON string)
        // or directly as an array under `result.crews` / `crews`.
        let inner = json
            .pointer("/result/content/0/text")
            .and_then(|v| v.as_str())
            .map(|s| {
                serde_json::from_str::<serde_json::Value>(s).unwrap_or(serde_json::Value::Null)
            })
            .unwrap_or_else(|| json.clone());

        let arr = inner
            .get("crews")
            .or_else(|| inner.as_array().map(|_| &inner))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let entries = arr
            .iter()
            .map(|c| CrewEntry {
                crew_id: c["id"]
                    .as_str()
                    .or_else(|| c["crew_id"].as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                name: c["name"].as_str().unwrap_or("unnamed crew").to_string(),
                status: c["status"]
                    .as_str()
                    .map(CrewRunStatus::from)
                    .unwrap_or_default(),
                agent_count: c["agent_count"].as_u64().unwrap_or(0) as usize,
                current_task: c["current_task"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
                progress: c["progress"].as_f64().map(|f| f as f32),
                last_updated: Utc::now(),
            })
            .collect();

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> serde_json::Value {
        serde_json::json!({
            "crews": [
                {
                    "id": "crew-alpha",
                    "name": "Alpha",
                    "status": "running",
                    "agent_count": 3,
                    "current_task": "Analysing logs",
                    "progress": 0.42
                },
                {
                    "id": "crew-beta",
                    "name": "Beta",
                    "status": "idle",
                    "agent_count": 2,
                    "current_task": "",
                    "progress": null
                }
            ]
        })
    }

    #[test]
    fn test_parse_crews() {
        let json = sample_json();
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].crew_id, "crew-alpha");
        assert_eq!(entries[0].status, CrewRunStatus::Running);
        assert_eq!(entries[0].agent_count, 3);
        assert_eq!(entries[0].progress, Some(0.42));
        assert_eq!(entries[1].status, CrewRunStatus::Idle);
        assert!(entries[1].current_task.is_none());
        assert!(entries[1].progress.is_none());
    }

    #[test]
    fn test_render_empty() {
        let widget = CrewStatusWidget::new();
        let data = widget.render();
        assert!(data.crews.is_empty());
        assert_eq!(data.active_count, 0);
        assert!(!data.last_fetch_ok);
        assert!(data.last_fetch_at.is_none());
    }

    #[test]
    fn test_render_with_crews() {
        let widget = CrewStatusWidget::new();
        let entries = vec![
            CrewEntry {
                crew_id: "c1".to_string(),
                name: "QA Crew".to_string(),
                status: CrewRunStatus::Running,
                agent_count: 4,
                current_task: Some("running tests".to_string()),
                progress: Some(0.7),
                last_updated: Utc::now(),
            },
            CrewEntry {
                crew_id: "c2".to_string(),
                name: "Deploy Crew".to_string(),
                status: CrewRunStatus::Completed,
                agent_count: 2,
                current_task: None,
                progress: Some(1.0),
                last_updated: Utc::now(),
            },
        ];
        widget.set_crews(entries);

        let data = widget.render();
        assert_eq!(data.crews.len(), 2);
        assert_eq!(data.active_count, 1);
        assert!(data.last_fetch_ok);
        assert!(data.last_fetch_at.is_some());
    }

    #[test]
    fn test_crew_run_status_from_str() {
        assert_eq!(CrewRunStatus::from("running"), CrewRunStatus::Running);
        assert_eq!(CrewRunStatus::from("active"), CrewRunStatus::Running);
        assert_eq!(CrewRunStatus::from("idle"), CrewRunStatus::Idle);
        assert_eq!(CrewRunStatus::from("completed"), CrewRunStatus::Completed);
        assert_eq!(CrewRunStatus::from("done"), CrewRunStatus::Completed);
        assert_eq!(CrewRunStatus::from("failed"), CrewRunStatus::Failed);
        assert_eq!(CrewRunStatus::from("error"), CrewRunStatus::Failed);
        assert_eq!(CrewRunStatus::from("paused"), CrewRunStatus::Paused);
        assert_eq!(CrewRunStatus::from("???"), CrewRunStatus::Unknown);
    }

    #[test]
    fn test_crew_entry_default() {
        let entry = CrewEntry::default();
        assert_eq!(entry.status, CrewRunStatus::Unknown);
        assert_eq!(entry.agent_count, 0);
        assert!(entry.current_task.is_none());
    }

    #[test]
    fn test_parse_mcp_wrapped_response() {
        // Simulate the MCP envelope where content is nested in result.content[0].text
        let inner = serde_json::json!({
            "crews": [
                { "id": "x", "name": "X", "status": "running", "agent_count": 1 }
            ]
        });
        let wrapped = serde_json::json!({
            "result": {
                "content": [
                    { "text": inner.to_string() }
                ]
            }
        });
        let entries = CrewStatusWidget::parse_crews(&wrapped).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].status, CrewRunStatus::Running);
    }

    #[test]
    fn test_crew_run_status_case_insensitive() {
        assert_eq!(CrewRunStatus::from("RUNNING"), CrewRunStatus::Running);
        assert_eq!(CrewRunStatus::from("Active"), CrewRunStatus::Running);
        assert_eq!(CrewRunStatus::from("IDLE"), CrewRunStatus::Idle);
        assert_eq!(CrewRunStatus::from("Paused"), CrewRunStatus::Paused);
        assert_eq!(CrewRunStatus::from("COMPLETED"), CrewRunStatus::Completed);
        assert_eq!(CrewRunStatus::from("Finished"), CrewRunStatus::Completed);
        assert_eq!(CrewRunStatus::from("FAILED"), CrewRunStatus::Failed);
        assert_eq!(CrewRunStatus::from("Error"), CrewRunStatus::Failed);
        assert_eq!(CrewRunStatus::from("Done"), CrewRunStatus::Completed);
    }

    #[test]
    fn test_crew_run_status_default() {
        let status = CrewRunStatus::default();
        assert_eq!(status, CrewRunStatus::Unknown);
    }

    #[test]
    fn test_parse_crews_bare_array() {
        // parse_crews also accepts a bare JSON array (not wrapped in "crews" key)
        let json = serde_json::json!([
            { "id": "c1", "name": "Crew1", "status": "idle", "agent_count": 1 },
            { "id": "c2", "name": "Crew2", "status": "running", "agent_count": 2 }
        ]);
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].crew_id, "c1");
        assert_eq!(entries[1].crew_id, "c2");
    }

    #[test]
    fn test_parse_crews_crew_id_field_fallback() {
        // When "id" is absent, falls back to "crew_id"
        let json = serde_json::json!({
            "crews": [
                { "crew_id": "fallback-id", "name": "Fallback", "status": "idle", "agent_count": 0 }
            ]
        });
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert_eq!(entries[0].crew_id, "fallback-id");
    }

    #[test]
    fn test_parse_crews_missing_id_defaults_to_unknown() {
        let json = serde_json::json!({
            "crews": [
                { "name": "NoId", "status": "idle", "agent_count": 0 }
            ]
        });
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert_eq!(entries[0].crew_id, "unknown");
    }

    #[test]
    fn test_parse_crews_missing_name_defaults_to_unnamed() {
        let json = serde_json::json!({
            "crews": [
                { "id": "x", "status": "idle", "agent_count": 0 }
            ]
        });
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert_eq!(entries[0].name, "unnamed crew");
    }

    #[test]
    fn test_parse_crews_missing_status_defaults_to_unknown() {
        let json = serde_json::json!({
            "crews": [
                { "id": "x", "name": "X", "agent_count": 1 }
            ]
        });
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert_eq!(entries[0].status, CrewRunStatus::Unknown);
    }

    #[test]
    fn test_parse_crews_empty_object() {
        let json = serde_json::json!({});
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_crews_null_value() {
        let json = serde_json::Value::Null;
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_crews_mcp_wrapped_invalid_inner_json() {
        // When inner text is invalid JSON, should fall back to the outer json
        let wrapped = serde_json::json!({
            "result": {
                "content": [
                    { "text": "not valid json!!!" }
                ]
            }
        });
        let entries = CrewStatusWidget::parse_crews(&wrapped).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_crew_entry_default_progress_and_task() {
        let entry = CrewEntry::default();
        assert!(entry.progress.is_none());
        assert!(entry.current_task.is_none());
        assert!(entry.name.is_empty());
        // crew_id should be a valid UUID string
        assert!(uuid::Uuid::parse_str(&entry.crew_id).is_ok());
    }

    #[test]
    fn test_widget_default_trait() {
        let widget = CrewStatusWidget::default();
        let data = widget.render();
        assert!(data.crews.is_empty());
        assert!(!data.last_fetch_ok);
    }

    #[test]
    fn test_set_crews_updates_fetch_state() {
        let widget = CrewStatusWidget::new();
        // Before set_crews, last_fetch_ok is false
        assert!(!widget.render().last_fetch_ok);
        assert!(widget.render().last_fetch_at.is_none());

        widget.set_crews(vec![]);
        // After set_crews, last_fetch_ok becomes true even with empty list
        assert!(widget.render().last_fetch_ok);
        assert!(widget.render().last_fetch_at.is_some());
    }

    #[test]
    fn test_render_active_count_multiple_running() {
        let widget = CrewStatusWidget::new();
        let entries = vec![
            CrewEntry {
                crew_id: "c1".to_string(),
                name: "A".to_string(),
                status: CrewRunStatus::Running,
                agent_count: 1,
                current_task: None,
                progress: None,
                last_updated: Utc::now(),
            },
            CrewEntry {
                crew_id: "c2".to_string(),
                name: "B".to_string(),
                status: CrewRunStatus::Running,
                agent_count: 2,
                current_task: None,
                progress: None,
                last_updated: Utc::now(),
            },
            CrewEntry {
                crew_id: "c3".to_string(),
                name: "C".to_string(),
                status: CrewRunStatus::Idle,
                agent_count: 1,
                current_task: None,
                progress: None,
                last_updated: Utc::now(),
            },
        ];
        widget.set_crews(entries);
        let data = widget.render();
        assert_eq!(data.active_count, 2);
    }

    #[test]
    fn test_crew_run_status_serde_roundtrip() {
        let statuses = vec![
            CrewRunStatus::Idle,
            CrewRunStatus::Running,
            CrewRunStatus::Paused,
            CrewRunStatus::Completed,
            CrewRunStatus::Failed,
            CrewRunStatus::Unknown,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let back: CrewRunStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_parse_crews_progress_present_and_absent() {
        let json = serde_json::json!({
            "crews": [
                { "id": "a", "name": "A", "status": "running", "agent_count": 1, "progress": 0.5 },
                { "id": "b", "name": "B", "status": "idle", "agent_count": 0 }
            ]
        });
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert_eq!(entries[0].progress, Some(0.5));
        assert!(entries[1].progress.is_none());
    }

    #[test]
    fn test_parse_crews_empty_current_task_becomes_none() {
        let json = serde_json::json!({
            "crews": [
                { "id": "a", "name": "A", "status": "idle", "agent_count": 0, "current_task": "" }
            ]
        });
        let entries = CrewStatusWidget::parse_crews(&json).unwrap();
        assert!(entries[0].current_task.is_none());
    }
}
