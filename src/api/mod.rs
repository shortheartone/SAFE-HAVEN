use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

pub const MAX_BULK_ITEMS: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkItem {
    pub id: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkItemError {
    pub index: usize,
    pub id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkResult {
    pub operation_id: String,
    pub total: usize,
    pub success: usize,
    pub failure: usize,
    pub errors: Vec<BulkItemError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkProgress {
    pub operation_id: String,
    pub total: usize,
    pub processed: usize,
    pub success: usize,
    pub failure: usize,
    pub done: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkOperation {
    Create,
    Update,
    Delete,
}

impl BulkOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            BulkOperation::Create => "create",
            BulkOperation::Update => "update",
            BulkOperation::Delete => "delete",
        }
    }
}

pub struct BulkProgressTracker {
    operations: Mutex<HashMap<String, BulkProgress>>,
}

impl BulkProgressTracker {
    pub fn new() -> Self {
        Self {
            operations: Mutex::new(HashMap::new()),
        }
    }

    pub fn start(&self, operation_id: &str, total: usize) {
        let mut ops = self.operations.lock().unwrap();
        ops.insert(
            operation_id.to_string(),
            BulkProgress {
                operation_id: operation_id.to_string(),
                total,
                processed: 0,
                success: 0,
                failure: 0,
                done: false,
            },
        );
    }

    pub fn record(&self, operation_id: &str, success: bool) {
        let mut ops = self.operations.lock().unwrap();
        if let Some(progress) = ops.get_mut(operation_id) {
            progress.processed += 1;
            if success {
                progress.success += 1;
            } else {
                progress.failure += 1;
            }
            if progress.processed >= progress.total {
                progress.done = true;
            }
        }
    }

    pub fn get(&self, operation_id: &str) -> Option<BulkProgress> {
        let ops = self.operations.lock().unwrap();
        ops.get(operation_id).cloned()
    }
}

impl Default for BulkProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_item(op: BulkOperation, index: usize, item: &BulkItem) -> Result<(), BulkItemError> {
    match op {
        BulkOperation::Create => {
            if item.data.is_null() {
                return Err(BulkItemError {
                    index,
                    id: item.id.clone(),
                    message: "item data must not be null".to_string(),
                });
            }
        }
        BulkOperation::Update | BulkOperation::Delete => {
            if item.id.as_deref().map(str::is_empty).unwrap_or(true) {
                return Err(BulkItemError {
                    index,
                    id: item.id.clone(),
                    message: "item id is required".to_string(),
                });
            }
        }
    }
    Ok(())
}

pub fn validate_bulk_request(items: &[BulkItem]) -> Result<(), BulkItemError> {
    if items.is_empty() {
        return Err(BulkItemError {
            index: 0,
            id: None,
            message: "bulk request must contain at least one item".to_string(),
        });
    }
    if items.len() > MAX_BULK_ITEMS {
        return Err(BulkItemError {
            index: MAX_BULK_ITEMS,
            id: None,
            message: format!("bulk request exceeds maximum of {} items", MAX_BULK_ITEMS),
        });
    }
    Ok(())
}

pub fn process_bulk<F>(
    op: BulkOperation,
    operation_id: &str,
    items: &[BulkItem],
    tracker: &BulkProgressTracker,
    mut apply: F,
) -> Result<BulkResult, BulkItemError>
where
    F: FnMut(BulkOperation, &BulkItem) -> Result<(), String>,
{
    validate_bulk_request(items)?;
    tracker.start(operation_id, items.len());

    let mut errors = Vec::new();
    let mut success = 0usize;

    for (index, item) in items.iter().enumerate() {
        let outcome = match validate_item(op, index, item) {
            Ok(()) => apply(op, item),
            Err(err) => Err(err.message),
        };

        match outcome {
            Ok(()) => {
                success += 1;
                tracker.record(operation_id, true);
            }
            Err(message) => {
                errors.push(BulkItemError {
                    index,
                    id: item.id.clone(),
                    message,
                });
                tracker.record(operation_id, false);
            }
        }
    }

    Ok(BulkResult {
        operation_id: operation_id.to_string(),
        total: items.len(),
        success,
        failure: errors.len(),
        errors,
    })
}
