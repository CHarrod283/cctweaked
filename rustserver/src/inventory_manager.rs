use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

pub const SECONDS_PER_REPORT: u64 = 5;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
pub struct InventoryReport {
    pub common_name: String,
    pub computer_id: i64,
    pub inventory: Vec<InventoryItem>,
    pub peripheral_name: String,
    pub inventory_type: InventoryType,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
pub struct InventoryItem {
    pub slot: i64,
    pub name: String,
    pub count: i64,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct InventoryRate {
    pub name: String,
    pub rate_per_second: f64,
}


#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
pub enum InventoryType {
    #[serde(rename = "input")]
    Input{
        destination: String,
    },
    #[serde(rename = "output")]
    Output{
        source: String,
    },
    #[serde(rename = "storage")]
    Storage
}

pub enum InventoryManagerReport{
    Input(Vec<InventoryRate>),
    Output(Vec<InventoryRate>),
    Storage(Vec<InventoryItem>),
}

type ComputerIds = Vec<i64>;
type InventoryReports = VecDeque<(Instant, InventoryReport)>;

pub struct InventoryManager {
    inventory_reports: RwLock<(ComputerIds,InventoryReports)>,
}

impl Default for InventoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryManager {
    pub fn new() -> Self {
        Self {
            inventory_reports: RwLock::new((Vec::new(), VecDeque::new())),
        }
    }
    
    pub async fn run(&self, mut event_receiver: tokio::sync::mpsc::UnboundedReceiver<InventoryReport>) {
        loop {
            let report = event_receiver.recv().await;
            let now = Instant::now();
            let mut guard = self.inventory_reports.write().await;
            loop {
                // drop the oldest reports if they are older than 30 minutes
                let Some(back) = guard.1.back() else {
                    break;
                };
                if back.0 + std::time::Duration::from_secs(60 * 30) > now {
                    break
                } 
                guard.1.pop_back();
            }
            if let Some(report) = report {
                if !guard.0.contains(&report.computer_id) {
                    // new computer id, reserve enough report capacity
                    guard.1.reserve(30 * (60 / SECONDS_PER_REPORT) as usize);
                }
                guard.1.push_front((now, report));
            }
        }
    }
    
    pub async fn get_storage_report(&self, computer_id: i64) -> Option<InventoryManagerReport> {
        let guard = self.inventory_reports.read().await;
        let report = guard.1.iter().find(|(_, report)| report.computer_id == computer_id)?;
        let (_, report) = report;
        if let InventoryType::Storage = report.inventory_type {
            Some(InventoryManagerReport::Storage(report.inventory.clone()))
        } else {
            None
        }
    }
    
    pub async fn get_rate_report(&self, computer_id: i64, over_past: Duration) -> Option<InventoryManagerReport> {
        let guard = self.inventory_reports.read().await;
        let mut inventory_rate_map: HashMap<String, f64> = HashMap::new();
        let mut inventory_type = None;
        for (_, report) in guard.1.iter().filter(|(time_reported,report)| {
            Instant::now().duration_since(*time_reported) <= over_past && report.computer_id == computer_id
        }) {
            if inventory_type.is_none() {
                match &report.inventory_type {
                    InventoryType::Storage { .. } => return None, // storage is not supported
                    itype => inventory_type = Some(itype),
                }
            }
            for item in &report.inventory {
                let entry = inventory_rate_map.entry(item.name.clone()).or_insert(0.0);
                *entry += item.count as f64;
            }
        }
        let mut inventory_rate = Vec::new();
        for (name, count) in inventory_rate_map {
            let rate = count / over_past.as_secs_f64();
            inventory_rate.push(InventoryRate {
                name,
                rate_per_second: rate,
            });
        }
        match inventory_type {
            Some(InventoryType::Input { .. }) => {
                Some(InventoryManagerReport::Input(inventory_rate))
            }
            Some(InventoryType::Output { .. }) => {
                Some(InventoryManagerReport::Output(inventory_rate))
            }
            _ => None,
        }
            
    }
}

fn time_within(time: &Instant, duration: Duration) -> bool {
    let now = Instant::now();
    let elapsed = now.duration_since(*time);
    elapsed < duration
}


#[cfg(test)]
mod tests {
    use ratatui::layout::Size;
    use crate::cctweaked::CCTweakedMonitorInputEvent;
    use super::*;

    #[test]
    fn test_serialize() {
        let report = InventoryReport {
            common_name: "Test Computer".to_string(),
            computer_id: 12345,
            inventory: vec![
                InventoryItem {
                    slot: 1,
                    name: "Test Item".to_string(),
                    count: 10,
                },
            ],
            peripheral_name: "Test Peripheral".to_string(),
            inventory_type: InventoryType::Input {
                destination: "Test Destination".to_string(),
            },
        };

        let serialized = serde_json::to_string(&report).unwrap();
        assert_eq!(
            serialized,
            r#"{"common_name":"Test Computer","computer_id":12345,"inventory":[{"slot":1,"name":"Test Item","count":10}],"peripheral_name":"Test Peripheral","inventory_type":{"input":{"destination":"Test Destination"}}}"#
        );

        let report = InventoryReport {
            common_name: "Test Computer".to_string(),
            computer_id: 12345,
            inventory: vec![
                InventoryItem {
                    slot: 1,
                    name: "Test Item".to_string(),
                    count: 10,
                },
            ],
            peripheral_name: "Test Peripheral".to_string(),
            inventory_type: InventoryType::Storage
        };

        let serialized = serde_json::to_string(&report).unwrap();
        assert_eq!(
            serialized,
            r#"{"common_name":"Test Computer","computer_id":12345,"inventory":[{"slot":1,"name":"Test Item","count":10}],"peripheral_name":"Test Peripheral","inventory_type":"storage"}"#
        );

        let monitor_resize = CCTweakedMonitorInputEvent::MonitorResize(Size { width: 10, height: 20 });
        let serialized = serde_json::to_string(&monitor_resize).unwrap();
        assert_eq!(
            serialized,
            r#"{"monitor_resize":{"width":10,"height":20}}"#
        );
    }
}