use crate::{Error, Result, Spreadsheet, SpreadsheetProvider, Sheet, CellValue};
use crate::microsoft::MicrosoftClient;
use async_trait::async_trait;
use serde::Deserialize;

impl MicrosoftClient {
    pub async fn get_workbook(&self, item_id: &str) -> Result<Workbook> {
        let response = self.client()
            .get(format!("{}/me/drive/items/{}/workbook", self.base_url(), item_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let workbook: Workbook = response.json().await?;
        Ok(workbook)
    }

    pub async fn list_worksheets(&self, item_id: &str) -> Result<WorksheetsResponse> {
        let response = self.client()
            .get(format!("{}/me/drive/items/{}/workbook/worksheets", self.base_url(), item_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let worksheets: WorksheetsResponse = response.json().await?;
        Ok(worksheets)
    }

    pub async fn get_worksheet_range(&self, item_id: &str, worksheet_name: &str, range: &str) -> Result<WorkbookRange> {
        let response = self.client()
            .get(format!(
                "{}/me/drive/items/{}/workbook/worksheets/{}/range(address='{}')",
                self.base_url(),
                item_id,
                urlencoding::encode(worksheet_name),
                urlencoding::encode(range)
            ))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let range_data: WorkbookRange = response.json().await?;
        Ok(range_data)
    }

    pub async fn update_worksheet_range(&self, item_id: &str, worksheet_name: &str, range: &str, values: Vec<Vec<serde_json::Value>>) -> Result<WorkbookRange> {
        let body = serde_json::json!({
            "values": values
        });

        let response = self.client()
            .patch(format!(
                "{}/me/drive/items/{}/workbook/worksheets/{}/range(address='{}')",
                self.base_url(),
                item_id,
                urlencoding::encode(worksheet_name),
                urlencoding::encode(range)
            ))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let range_data: WorkbookRange = response.json().await?;
        Ok(range_data)
    }

    pub async fn get_used_range(&self, item_id: &str, worksheet_name: &str) -> Result<WorkbookRange> {
        let response = self.client()
            .get(format!(
                "{}/me/drive/items/{}/workbook/worksheets/{}/usedRange",
                self.base_url(),
                item_id,
                urlencoding::encode(worksheet_name)
            ))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let range_data: WorkbookRange = response.json().await?;
        Ok(range_data)
    }

    pub async fn add_worksheet(&self, item_id: &str, name: &str) -> Result<Worksheet> {
        let body = serde_json::json!({
            "name": name
        });

        let response = self.client()
            .post(format!("{}/me/drive/items/{}/workbook/worksheets/add", self.base_url(), item_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let worksheet: Worksheet = response.json().await?;
        Ok(worksheet)
    }

    pub async fn delete_worksheet(&self, item_id: &str, worksheet_name: &str) -> Result<()> {
        let response = self.client()
            .delete(format!(
                "{}/me/drive/items/{}/workbook/worksheets/{}",
                self.base_url(),
                item_id,
                urlencoding::encode(worksheet_name)
            ))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        Ok(())
    }

    pub async fn create_table(&self, item_id: &str, worksheet_name: &str, range: &str, has_headers: bool) -> Result<WorkbookTable> {
        let body = serde_json::json!({
            "address": range,
            "hasHeaders": has_headers
        });

        let response = self.client()
            .post(format!(
                "{}/me/drive/items/{}/workbook/worksheets/{}/tables/add",
                self.base_url(),
                item_id,
                urlencoding::encode(worksheet_name)
            ))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let table: WorkbookTable = response.json().await?;
        Ok(table)
    }

    pub async fn add_table_row(&self, item_id: &str, table_name: &str, values: Vec<serde_json::Value>) -> Result<TableRow> {
        let body = serde_json::json!({
            "values": [values]
        });

        let response = self.client()
            .post(format!(
                "{}/me/drive/items/{}/workbook/tables/{}/rows/add",
                self.base_url(),
                item_id,
                urlencoding::encode(table_name)
            ))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let row: TableRow = response.json().await?;
        Ok(row)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Workbook {
    pub id: Option<String>,
    pub application: Option<WorkbookApplication>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkbookApplication {
    #[serde(rename = "calculationMode")]
    pub calculation_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorksheetsResponse {
    pub value: Vec<Worksheet>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Worksheet {
    pub id: String,
    pub name: String,
    pub position: i32,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkbookRange {
    pub address: Option<String>,
    #[serde(rename = "addressLocal")]
    pub address_local: Option<String>,
    #[serde(rename = "cellCount")]
    pub cell_count: Option<i32>,
    #[serde(rename = "columnCount")]
    pub column_count: Option<i32>,
    #[serde(rename = "columnIndex")]
    pub column_index: Option<i32>,
    #[serde(rename = "rowCount")]
    pub row_count: Option<i32>,
    #[serde(rename = "rowIndex")]
    pub row_index: Option<i32>,
    pub values: Option<Vec<Vec<serde_json::Value>>>,
    pub formulas: Option<Vec<Vec<serde_json::Value>>>,
    pub text: Option<Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkbookTable {
    pub id: String,
    pub name: String,
    #[serde(rename = "showHeaders")]
    pub show_headers: Option<bool>,
    #[serde(rename = "showTotals")]
    pub show_totals: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableRow {
    pub index: i32,
    pub values: Option<Vec<Vec<serde_json::Value>>>,
}

fn json_to_cell_value(value: &serde_json::Value) -> CellValue {
    match value {
        serde_json::Value::String(s) => {
            if s.starts_with('=') {
                CellValue::Formula(s.clone())
            } else {
                CellValue::Text(s.clone())
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                CellValue::Number(f)
            } else {
                CellValue::Text(n.to_string())
            }
        }
        serde_json::Value::Bool(b) => CellValue::Boolean(*b),
        serde_json::Value::Null => CellValue::Empty,
        _ => CellValue::Text(value.to_string()),
    }
}

fn cell_value_to_json(value: &CellValue) -> serde_json::Value {
    match value {
        CellValue::Text(s) => serde_json::Value::String(s.clone()),
        CellValue::Number(n) => serde_json::json!(n),
        CellValue::Boolean(b) => serde_json::Value::Bool(*b),
        CellValue::Formula(f) => serde_json::Value::String(f.clone()),
        CellValue::Empty => serde_json::Value::Null,
    }
}

pub struct ExcelProvider {
    client: MicrosoftClient,
    item_id: String,
}

impl ExcelProvider {
    pub fn new(access_token: &str, item_id: &str) -> Self {
        Self {
            client: MicrosoftClient::new(access_token),
            item_id: item_id.to_string(),
        }
    }
}

#[async_trait]
impl SpreadsheetProvider for ExcelProvider {
    async fn get_spreadsheet(&self, _id: &str) -> Result<Spreadsheet> {
        let worksheets = self.client.list_worksheets(&self.item_id).await?;

        let sheets = worksheets.value.into_iter()
            .map(|ws| Sheet {
                id: ws.id,
                name: ws.name,
                index: ws.position as u32,
            })
            .collect();

        Ok(Spreadsheet {
            id: self.item_id.clone(),
            title: "Excel Workbook".to_string(),
            url: None,
            sheets,
        })
    }

    async fn create_spreadsheet(&self, _title: &str) -> Result<Spreadsheet> {
        Err(Error::InvalidRequest("Cannot create Excel workbooks via API. Upload a file to OneDrive instead.".to_string()))
    }

    async fn get_values(&self, _spreadsheet_id: &str, range: &str) -> Result<Vec<Vec<CellValue>>> {
        let parts: Vec<&str> = range.splitn(2, '!').collect();
        let (worksheet_name, cell_range) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("Sheet1", range)
        };

        let range_data = self.client.get_worksheet_range(&self.item_id, worksheet_name, cell_range).await?;

        let values = range_data.values.unwrap_or_default().into_iter()
            .map(|row| row.iter().map(json_to_cell_value).collect())
            .collect();

        Ok(values)
    }

    async fn update_values(&self, _spreadsheet_id: &str, range: &str, values: Vec<Vec<CellValue>>) -> Result<()> {
        let parts: Vec<&str> = range.splitn(2, '!').collect();
        let (worksheet_name, cell_range) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("Sheet1", range)
        };

        let json_values: Vec<Vec<serde_json::Value>> = values.iter()
            .map(|row| row.iter().map(cell_value_to_json).collect())
            .collect();

        self.client.update_worksheet_range(&self.item_id, worksheet_name, cell_range, json_values).await?;
        Ok(())
    }

    async fn append_values(&self, _spreadsheet_id: &str, _range: &str, _values: Vec<Vec<CellValue>>) -> Result<()> {
        Err(Error::InvalidRequest("Append not directly supported in Excel. Use update_values with specific range.".to_string()))
    }

    async fn clear_values(&self, _spreadsheet_id: &str, range: &str) -> Result<()> {
        let parts: Vec<&str> = range.splitn(2, '!').collect();
        let (worksheet_name, cell_range) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("Sheet1", range)
        };

        let range_data = self.client.get_worksheet_range(&self.item_id, worksheet_name, cell_range).await?;

        if let Some(values) = &range_data.values {
            let empty_values: Vec<Vec<serde_json::Value>> = values.iter()
                .map(|row| row.iter().map(|_| serde_json::Value::Null).collect())
                .collect();
            self.client.update_worksheet_range(&self.item_id, worksheet_name, cell_range, empty_values).await?;
        }

        Ok(())
    }
}
