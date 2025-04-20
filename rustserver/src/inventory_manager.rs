use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
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

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct InventoryItemCount {
    pub name: String,
    pub count: i64,
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
    Storage(Vec<InventoryItemCount>),
}

type ComputerIds = Vec<i64>;
type InventoryReports = VecDeque<(Instant, InventoryReport)>;

pub struct InventoryManager {
    inventory_reports: RwLock<(ComputerIds,InventoryReports)>,
    // used so that we can clone the sender
    sender: UnboundedSender<InventoryReport>,
}


impl InventoryManager {
    pub fn new(sender: UnboundedSender<InventoryReport>) -> Self {
        Self {
            inventory_reports: RwLock::new((Vec::new(), VecDeque::new())),
            sender,
        }
    }

    pub fn get_sender(&self) -> UnboundedSender<InventoryReport> {
        self.sender.clone()
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


    pub async fn get_report(&self, computer_id: i64, over_past: Duration) -> Option<InventoryManagerReport> {
        let guard = self.inventory_reports.read().await;
        let mut inventory_rate_map: HashMap<String, f64> = HashMap::new();
        let mut inventory_type = None;
        let mut number_of_reports = 0;
        for (_, report) in guard.1.iter().filter(|(time_reported,report)| {
            Instant::now().duration_since(*time_reported) <= over_past && report.computer_id == computer_id
        }) {
            number_of_reports += 1;
            match inventory_type {
                None => {
                    inventory_type = Some(report.inventory_type.clone());
                }
                Some(ref mut inventory_type) => {
                    if *inventory_type != report.inventory_type {
                        // monitor registered with different inventory type at some point, so discard old reports
                        break;
                    }
                }
            }
            for item in &report.inventory {
                let entry = inventory_rate_map.entry(item.name.clone()).or_insert(0.0);
                *entry += item.count as f64;
            }
            if let InventoryType::Storage = report.inventory_type {
                break; // storage we only care about the most recent report
            }
        }

        match inventory_type {
            Some(InventoryType::Input { .. }) => {
                let mut inventory_rate = Vec::new();
                for (name, count) in inventory_rate_map {
                    let rate = count / number_of_reports as f64 / SECONDS_PER_REPORT as f64;
                    inventory_rate.push(InventoryRate {
                        name,
                        rate_per_second: rate,
                    });
                }
                Some(InventoryManagerReport::Input(inventory_rate))
            }
            Some(InventoryType::Output { .. }) => {
                let mut inventory_rate = Vec::new();
                for (name, count) in inventory_rate_map {
                    let rate = count / number_of_reports as f64 / SECONDS_PER_REPORT as f64;
                    inventory_rate.push(InventoryRate {
                        name,
                        rate_per_second: rate,
                    });
                }
                Some(InventoryManagerReport::Output(inventory_rate))
            }
            Some(InventoryType::Storage) => {
                let mut inventory_count = Vec::new();
                for (name, count) in inventory_rate_map {
                    inventory_count.push(InventoryItemCount {
                        name,
                        count: count as i64,
                    });
                }
                Some(InventoryManagerReport::Storage(inventory_count))
            }
            None => {
                // no inventory type found, return None
                None
            }
        }

    }
}

impl PartialOrd for InventoryRate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.rate_per_second == other.rate_per_second {
            return Some(self.name.cmp(&other.name));
        }
        if self.rate_per_second > other.rate_per_second {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Less)
    }
}

impl PartialOrd for InventoryItemCount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.count == other.count {
            return Some(self.name.cmp(&other.name));
        }
        if self.count > other.count {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Less)
    }
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

        let inventory_Register = CCTweakedMonitorInputEvent::InventoryRegister {
            size: Size { width: 10, height: 20 },
            computer_id: 0,
            common_name: "123".to_string(),
        };
        let serialized = serde_json::to_string(&inventory_Register).unwrap();
        assert_eq!(
            serialized,
            r#"{"inventory_register":{"size":{"width":10,"height":20},"computer_id":0,"common_name":"123"}}"#
        );
    }
}