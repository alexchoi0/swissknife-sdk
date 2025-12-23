use crate::{Error, Result, Spreadsheet, SpreadsheetProvider, CellValue};
use crate::google::GoogleClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const SHEETS_URL: &str = "https://sheets.googleapis.com/v4";

impl GoogleClient {
    pub async fn get_spreadsheet(&self, spreadsheet_id: &str) -> Result<GoogleSpreadsheet> {
        let response = self.client()
            .get(format!("{}/spreadsheets/{}", SHEETS_URL, spreadsheet_id))
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

        let spreadsheet: GoogleSpreadsheet = response.json().await?;
        Ok(spreadsheet)
    }

    pub async fn create_spreadsheet(&self, title: &str) -> Result<GoogleSpreadsheet> {
        let body = serde_json::json!({
            "properties": {
                "title": title
            }
        });

        let response = self.client()
            .post(format!("{}/spreadsheets", SHEETS_URL))
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

        let spreadsheet: GoogleSpreadsheet = response.json().await?;
        Ok(spreadsheet)
    }

    pub async fn get_sheet_values(&self, spreadsheet_id: &str, range: &str) -> Result<ValueRange> {
        let response = self.client()
            .get(format!("{}/spreadsheets/{}/values/{}", SHEETS_URL, spreadsheet_id, urlencoding::encode(range)))
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

        let values: ValueRange = response.json().await?;
        Ok(values)
    }

    pub async fn update_sheet_values(&self, spreadsheet_id: &str, range: &str, values: Vec<Vec<serde_json::Value>>) -> Result<UpdateValuesResponse> {
        let body = serde_json::json!({
            "range": range,
            "majorDimension": "ROWS",
            "values": values
        });

        let response = self.client()
            .put(format!("{}/spreadsheets/{}/values/{}", SHEETS_URL, spreadsheet_id, urlencoding::encode(range)))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("valueInputOption", "USER_ENTERED")])
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

        let result: UpdateValuesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn append_sheet_values(&self, spreadsheet_id: &str, range: &str, values: Vec<Vec<serde_json::Value>>) -> Result<AppendValuesResponse> {
        let body = serde_json::json!({
            "range": range,
            "majorDimension": "ROWS",
            "values": values
        });

        let response = self.client()
            .post(format!("{}/spreadsheets/{}/values/{}:append", SHEETS_URL, spreadsheet_id, urlencoding::encode(range)))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[
                ("valueInputOption", "USER_ENTERED"),
                ("insertDataOption", "INSERT_ROWS"),
            ])
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

        let result: AppendValuesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn clear_sheet_values(&self, spreadsheet_id: &str, range: &str) -> Result<ClearValuesResponse> {
        let response = self.client()
            .post(format!("{}/spreadsheets/{}/values/{}:clear", SHEETS_URL, spreadsheet_id, urlencoding::encode(range)))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&serde_json::json!({}))
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

        let result: ClearValuesResponse = response.json().await?;
        Ok(result)
    }

    pub async fn batch_get_values(&self, spreadsheet_id: &str, ranges: &[&str]) -> Result<BatchGetValuesResponse> {
        let ranges_param: Vec<(&str, &str)> = ranges.iter().map(|r| ("ranges", *r)).collect();

        let response = self.client()
            .get(format!("{}/spreadsheets/{}/values:batchGet", SHEETS_URL, spreadsheet_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&ranges_param)
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

        let result: BatchGetValuesResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleSpreadsheet {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    pub properties: SpreadsheetProperties,
    pub sheets: Option<Vec<Sheet>>,
    #[serde(rename = "spreadsheetUrl")]
    pub spreadsheet_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpreadsheetProperties {
    pub title: String,
    pub locale: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sheet {
    pub properties: SheetProperties,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SheetProperties {
    #[serde(rename = "sheetId")]
    pub sheet_id: i64,
    pub title: String,
    pub index: i32,
    #[serde(rename = "gridProperties")]
    pub grid_properties: Option<GridProperties>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GridProperties {
    #[serde(rename = "rowCount")]
    pub row_count: Option<i32>,
    #[serde(rename = "columnCount")]
    pub column_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValueRange {
    pub range: Option<String>,
    #[serde(rename = "majorDimension")]
    pub major_dimension: Option<String>,
    pub values: Option<Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateValuesResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "updatedRange")]
    pub updated_range: Option<String>,
    #[serde(rename = "updatedRows")]
    pub updated_rows: Option<i32>,
    #[serde(rename = "updatedColumns")]
    pub updated_columns: Option<i32>,
    #[serde(rename = "updatedCells")]
    pub updated_cells: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppendValuesResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "tableRange")]
    pub table_range: Option<String>,
    pub updates: Option<UpdateValuesResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClearValuesResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "clearedRange")]
    pub cleared_range: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchGetValuesResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "valueRanges")]
    pub value_ranges: Option<Vec<ValueRange>>,
}

fn json_to_cell_value(value: &serde_json::Value) -> CellValue {
    match value {
        serde_json::Value::String(s) => CellValue::Text(s.clone()),
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
        CellValue::Empty => serde_json::Value::String(String::new()),
    }
}

pub struct GoogleSheetsProvider {
    client: GoogleClient,
}

impl GoogleSheetsProvider {
    pub fn new(access_token: &str) -> Self {
        Self {
            client: GoogleClient::new(access_token),
        }
    }
}

#[async_trait]
impl SpreadsheetProvider for GoogleSheetsProvider {
    async fn get_spreadsheet(&self, id: &str) -> Result<Spreadsheet> {
        let spreadsheet = self.client.get_spreadsheet(id).await?;

        let sheets = spreadsheet.sheets.unwrap_or_default().into_iter()
            .map(|s| crate::Sheet {
                id: s.properties.sheet_id.to_string(),
                name: s.properties.title,
                index: s.properties.index as u32,
            })
            .collect();

        Ok(Spreadsheet {
            id: spreadsheet.spreadsheet_id,
            title: spreadsheet.properties.title,
            url: spreadsheet.spreadsheet_url,
            sheets,
        })
    }

    async fn create_spreadsheet(&self, title: &str) -> Result<Spreadsheet> {
        let spreadsheet = self.client.create_spreadsheet(title).await?;

        let sheets = spreadsheet.sheets.unwrap_or_default().into_iter()
            .map(|s| crate::Sheet {
                id: s.properties.sheet_id.to_string(),
                name: s.properties.title,
                index: s.properties.index as u32,
            })
            .collect();

        Ok(Spreadsheet {
            id: spreadsheet.spreadsheet_id,
            title: spreadsheet.properties.title,
            url: spreadsheet.spreadsheet_url,
            sheets,
        })
    }

    async fn get_values(&self, spreadsheet_id: &str, range: &str) -> Result<Vec<Vec<CellValue>>> {
        let value_range = self.client.get_sheet_values(spreadsheet_id, range).await?;

        let values = value_range.values.unwrap_or_default().into_iter()
            .map(|row| row.iter().map(json_to_cell_value).collect())
            .collect();

        Ok(values)
    }

    async fn update_values(&self, spreadsheet_id: &str, range: &str, values: Vec<Vec<CellValue>>) -> Result<()> {
        let json_values: Vec<Vec<serde_json::Value>> = values.iter()
            .map(|row| row.iter().map(cell_value_to_json).collect())
            .collect();

        self.client.update_sheet_values(spreadsheet_id, range, json_values).await?;
        Ok(())
    }

    async fn append_values(&self, spreadsheet_id: &str, range: &str, values: Vec<Vec<CellValue>>) -> Result<()> {
        let json_values: Vec<Vec<serde_json::Value>> = values.iter()
            .map(|row| row.iter().map(cell_value_to_json).collect())
            .collect();

        self.client.append_sheet_values(spreadsheet_id, range, json_values).await?;
        Ok(())
    }

    async fn clear_values(&self, spreadsheet_id: &str, range: &str) -> Result<()> {
        self.client.clear_sheet_values(spreadsheet_id, range).await?;
        Ok(())
    }
}
