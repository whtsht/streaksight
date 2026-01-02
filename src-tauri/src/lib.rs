use chrono::NaiveDate;
use deno_core::{extension, op2};
use deno_error::JsErrorBox;

mod query_builder;

fn duckdb_connect() -> Result<Connection, JsErrorBox> {
    let app_data_path = APP_DATA_PATH.get().ok_or_else(|| {
        JsErrorBox::from_err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "APP_DATA_PATH not initialized",
        ))
    })?;

    let db_path = app_data_path.join("database.duckdb");
    let conn = Connection::open(&db_path).map_err(|e| {
        JsErrorBox::from_err(std::io::Error::other(format!(
            "Failed to open DuckDB: {}",
            e
        )))
    })?;
    Ok(conn)
}

#[op2(async)]
#[string]
async fn op_read_file(#[string] path: String) -> Result<String, JsErrorBox> {
    let s = tokio::fs::read_to_string(path)
        .await
        .map_err(JsErrorBox::from_err)?;
    Ok(s)
}

#[op2(async)]
async fn op_write_file(
    #[string] path: String,
    #[string] contents: String,
) -> Result<(), JsErrorBox> {
    tokio::fs::write(path, contents)
        .await
        .map_err(JsErrorBox::from_err)?;
    Ok(())
}

#[op2(async)]
#[serde]
async fn op_run_sql(#[string] sql: String) -> Result<serde_json::Value, JsErrorBox> {
    let conn = duckdb_connect()?;

    let column_names = {
        let mut info_stmt = conn.prepare(&sql).map_err(|e| {
            JsErrorBox::from_err(std::io::Error::other(format!(
                "Failed to prepare SQL: {}",
                e
            )))
        })?;
        info_stmt.execute([]).map_err(|e| {
            JsErrorBox::from_err(std::io::Error::other(format!(
                "Failed to execute query: {}",
                e
            )))
        })?;
        info_stmt.column_names()
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| {
        JsErrorBox::from_err(std::io::Error::other(format!(
            "Failed to prepare SQL: {}",
            e
        )))
    })?;

    let mut rows = stmt.query([]).map_err(|e| {
        JsErrorBox::from_err(std::io::Error::other(format!(
            "Failed to execute query: {}",
            e
        )))
    })?;

    let mut rows_data = Vec::new();

    while let Some(row) = rows.next().map_err(|e| {
        JsErrorBox::from_err(std::io::Error::other(format!("Failed to fetch row: {}", e)))
    })? {
        let mut map = serde_json::Map::new();
        for (i, col_name) in column_names.iter().enumerate() {
            let value = match row.get_ref(i) {
                Ok(val) => duckdb_value_to_json(val),
                Err(_) => serde_json::Value::Null,
            };
            map.insert(col_name.clone(), value);
        }
        rows_data.push(serde_json::Value::Object(map));
    }

    Ok(serde_json::Value::Array(rows_data))
}

extension!(
    streaksight_ext,
    ops = [op_read_file, op_write_file, op_run_sql],
    esm_entry_point = "ext:streaksight_ext/src/runtime.js",
    esm = ["src/runtime.js"],
);

mod connector_type {
    pub const LOCAL_FILE_CSV: &str = "LocalFileCSV";
    pub const LOCAL_FILE_JSON: &str = "LocalFileJSON";
}

fn resolve_connector_path(ty: &str) -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;

    let connector_file = match ty {
        connector_type::LOCAL_FILE_CSV => "LocalFileCSVConnector.js",
        connector_type::LOCAL_FILE_JSON => "LocalFileJSONConnector.js",
        _ => return Err("Unknown connector type".to_string()),
    };

    let connector_path = if current_dir.ends_with("src-tauri") {
        current_dir.join(format!("src/{}", connector_file))
    } else {
        current_dir.join(format!("src-tauri/src/{}", connector_file))
    };

    Ok(connector_path)
}

fn duckdb_value_to_json(value: duckdb::types::ValueRef) -> serde_json::Value {
    match value {
        duckdb::types::ValueRef::Null => serde_json::Value::Null,
        duckdb::types::ValueRef::Boolean(b) => serde_json::Value::Bool(b),
        duckdb::types::ValueRef::TinyInt(i) => serde_json::Value::Number(i.into()),
        duckdb::types::ValueRef::SmallInt(i) => serde_json::Value::Number(i.into()),
        duckdb::types::ValueRef::Int(i) => serde_json::Value::Number(i.into()),
        duckdb::types::ValueRef::BigInt(i) => serde_json::Value::Number(i.into()),
        duckdb::types::ValueRef::HugeInt(i) => serde_json::Value::Number((i as i64).into()),
        duckdb::types::ValueRef::Float(f) => serde_json::Number::from_f64(f as f64)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        duckdb::types::ValueRef::Double(d) => serde_json::Number::from_f64(d)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        duckdb::types::ValueRef::Text(s) => {
            serde_json::Value::String(String::from_utf8_lossy(s).to_string())
        }
        duckdb::types::ValueRef::Date32(days) => {
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
            let date = epoch + chrono::Duration::days(days as i64);
            serde_json::Value::String(date.format("%Y-%m-%d").to_string())
        }
        _ => serde_json::Value::String(format!("{:?}", value)),
    }
}

async fn load_runtime_js(
    runtime: &mut deno_core::JsRuntime,
    current_dir: &std::path::Path,
) -> Result<(), String> {
    let runtime_js_path = if current_dir.ends_with("src-tauri") {
        current_dir.join("src/runtime.js")
    } else {
        current_dir.join("src-tauri/src/runtime.js")
    };

    let runtime_js_url = deno_core::ModuleSpecifier::from_file_path(&runtime_js_path)
        .map_err(|_| "Failed to convert runtime.js path to URL".to_string())?;

    let runtime_id = runtime
        .load_side_es_module(&runtime_js_url)
        .await
        .map_err(|e| format!("Failed to load runtime.js: {}", e))?;
    let runtime_eval = runtime.mod_evaluate(runtime_id);
    runtime
        .run_event_loop(Default::default())
        .await
        .map_err(|e| format!("Failed to run event loop for runtime.js: {}", e))?;
    runtime_eval
        .await
        .map_err(|e| format!("Failed to evaluate runtime.js: {}", e))?;

    Ok(())
}

async fn execute_deno_module(
    runtime: &mut deno_core::JsRuntime,
    module_path: &deno_core::ModuleSpecifier,
) -> Result<(), String> {
    let id = runtime
        .load_main_es_module(module_path)
        .await
        .map_err(|e| format!("Failed to load module: {}", e))?;
    let eval = runtime.mod_evaluate(id);
    runtime
        .run_event_loop(Default::default())
        .await
        .map_err(|e| format!("Failed to run event loop: {}", e))?;
    eval.await
        .map_err(|e| format!("Failed to evaluate module: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn config(ty: String) -> Result<String, String> {
    if ty != connector_type::LOCAL_FILE_CSV && ty != connector_type::LOCAL_FILE_JSON {
        return Err("Unknown connector type".to_string());
    }

    tokio::task::spawn_blocking(move || {
        use deno_core::{JsRuntime, RuntimeOptions};
        use std::rc::Rc;

        let connector_path = resolve_connector_path(&ty)?;

        let result_file_path = std::env::temp_dir().join("streaksight_config_result.json");

        let temp_js = format!(
            r#"import {{ config }} from "{}";
               const result = config();
               const resultJson = JSON.stringify(result);
               await streaksight.writeFile("{}", resultJson);"#,
            connector_path.to_str().unwrap().replace("\\", "/"),
            result_file_path.to_str().unwrap().replace("\\", "/")
        );

        let temp_js_path = std::env::temp_dir().join("streaksight_config_temp.js");
        std::fs::write(&temp_js_path, temp_js)
            .map_err(|e| format!("Failed to write temp JS file: {}", e))?;

        let module_path = deno_core::ModuleSpecifier::from_file_path(&temp_js_path)
            .map_err(|_| "Failed to convert temp path to URL".to_string())?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            let mut runtime = JsRuntime::new(RuntimeOptions {
                module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
                extensions: vec![streaksight_ext::init()],
                ..Default::default()
            });

            execute_deno_module(&mut runtime, &module_path).await?;

            let json_str = std::fs::read_to_string(&result_file_path)
                .map_err(|e| format!("Failed to read result file: {}", e))?;

            let _ = std::fs::remove_file(&temp_js_path);
            let _ = std::fs::remove_file(&result_file_path);

            Ok(json_str)
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn discovery(ty: String, config: String) -> Result<String, String> {
    if ty != connector_type::LOCAL_FILE_CSV && ty != connector_type::LOCAL_FILE_JSON {
        return Err("Unknown connector type".to_string());
    }

    tokio::task::spawn_blocking(move || {
        use deno_core::{JsRuntime, RuntimeOptions};
        use std::rc::Rc;

        let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
        let connector_path = resolve_connector_path(&ty)?;

        if !connector_path.exists() {
            return Err(format!("Connector file not found: {:?}", connector_path));
        }

        let result_file_path = std::env::temp_dir().join("streaksight_discovery_result.json");
        let temp_js_path = std::env::temp_dir().join("streaksight_discovery_temp.js");

        let temp_js = format!(
            r#"import {{ discovery }} from "{}";
               const configObj = JSON.parse(`{}`);
               const result = await discovery(configObj);
               const resultJson = JSON.stringify(result);
               await streaksight.writeFile("{}", resultJson);"#,
            connector_path.to_string_lossy().replace('\\', "/"),
            config.replace('\\', "\\\\").replace('`', "\\`"),
            result_file_path.to_string_lossy().replace('\\', "/")
        );

        std::fs::write(&temp_js_path, temp_js)
            .map_err(|e| format!("Failed to write temp JS: {}", e))?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            let mut runtime = JsRuntime::new(RuntimeOptions {
                module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
                extensions: vec![streaksight_ext::init()],
                ..Default::default()
            });

            load_runtime_js(&mut runtime, &current_dir).await?;

            let module_path = deno_core::ModuleSpecifier::from_file_path(&temp_js_path)
                .map_err(|_| "Failed to convert temp path to URL".to_string())?;

            execute_deno_module(&mut runtime, &module_path).await?;

            let json_str = std::fs::read_to_string(&result_file_path)
                .map_err(|e| format!("Failed to read result file: {}", e))?;

            let _ = std::fs::remove_file(&temp_js_path);
            let _ = std::fs::remove_file(&result_file_path);

            Ok(json_str)
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn sync(ty: String, name: String, config: String, schema: String) -> Result<String, String> {
    if ty != connector_type::LOCAL_FILE_CSV && ty != connector_type::LOCAL_FILE_JSON {
        return Err("Unknown connector type".to_string());
    }

    tokio::task::spawn_blocking(move || {
        use deno_core::{JsRuntime, RuntimeOptions};
        use std::rc::Rc;

        let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
        let connector_path = resolve_connector_path(&ty)?;

        if !connector_path.exists() {
            return Err(format!("Connector file not found: {:?}", connector_path));
        }

        let temp_js_path = std::env::temp_dir().join("streaksight_sync_temp.js");

        let temp_js = format!(
            r#"import {{ sync }} from "{}";
               const configObj = JSON.parse(`{}`);
               const schemaObj = JSON.parse(`{}`);
               await sync("{}", configObj, schemaObj);"#,
            connector_path.to_string_lossy().replace('\\', "/"),
            config.replace('\\', "\\\\").replace('`', "\\`"),
            schema.replace('\\', "\\\\").replace('`', "\\`"),
            name.replace('"', "\\\"")
        );

        std::fs::write(&temp_js_path, temp_js)
            .map_err(|e| format!("Failed to write temp JS: {}", e))?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            let mut runtime = JsRuntime::new(RuntimeOptions {
                module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
                extensions: vec![streaksight_ext::init()],
                ..Default::default()
            });

            load_runtime_js(&mut runtime, &current_dir).await?;

            let module_path = deno_core::ModuleSpecifier::from_file_path(&temp_js_path)
                .map_err(|_| "Failed to convert temp path to URL".to_string())?;

            execute_deno_module(&mut runtime, &module_path).await?;

            let _ = std::fs::remove_file(&temp_js_path);

            Ok("Sync completed successfully".to_string())
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn tables() -> Result<String, String> {
    let conn = duckdb_connect().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let tables: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            Ok(serde_json::json!({
                "name": name,
                "row_count": 0
            }))
        })
        .map_err(|e| format!("Failed to query tables: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

    let result = serde_json::json!({
        "tables": tables
    });

    Ok(result.to_string())
}

#[tauri::command]
async fn table_schema(table_name: String) -> Result<String, String> {
    let conn = duckdb_connect().map_err(|e| e.to_string())?;

    let query = format!("DESCRIBE {}", table_name);
    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let columns: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let column_type: String = row.get(1)?;

            let mapped_type = match column_type.to_uppercase().as_str() {
                t if t.contains("INT")
                    || t.contains("DOUBLE")
                    || t.contains("FLOAT")
                    || t.contains("DECIMAL") =>
                {
                    "number"
                }
                t if t.contains("BOOL") => "boolean",
                t if t.contains("DATE") || t.contains("TIME") => "date",
                _ => "string",
            };

            Ok(serde_json::json!({
                "name": name,
                "type": mapped_type
            }))
        })
        .map_err(|e| format!("Failed to query schema: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

    let result = serde_json::json!({
        "table_name": table_name,
        "columns": columns
    });

    Ok(result.to_string())
}

#[tauri::command]
async fn drop_table(table_name: String) -> Result<String, String> {
    let conn = duckdb_connect().map_err(|e| e.to_string())?;

    if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Invalid table name".to_string());
    }

    let query = format!("DROP TABLE IF EXISTS {}", table_name);
    conn.execute(&query, [])
        .map_err(|e| format!("Failed to drop table: {}", e))?;

    Ok(format!("Table {} dropped successfully", table_name))
}

#[tauri::command]
async fn run_query(
    node_graph: String,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<String, String> {
    let graph: query_builder::NodeGraph = serde_json::from_str(&node_graph)
        .map_err(|e| format!("Failed to parse node graph: {}", e))?;

    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(100);
    let limit = page_size as i64;
    let offset = ((page - 1) * page_size) as i64;

    let sql = query_builder::generate_sql(&graph, Some((limit, offset)))?;

    let conn = duckdb_connect().map_err(|e| e.to_string())?;

    let column_names = {
        let mut info_stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare SQL: {}", e))?;
        info_stmt
            .execute([])
            .map_err(|e| format!("Failed to execute query: {}", e))?;
        info_stmt.column_names()
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare SQL: {}", e))?;

    let mut rows_data = Vec::new();
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to fetch row: {}", e))?
    {
        let mut row_obj = serde_json::Map::new();
        for (i, col_name) in column_names.iter().enumerate() {
            let value = match row.get_ref(i) {
                Ok(val) => duckdb_value_to_json(val),
                Err(_) => serde_json::Value::Null,
            };
            row_obj.insert(col_name.clone(), value);
        }
        rows_data.push(serde_json::Value::Object(row_obj));
    }

    let columns_info: Vec<serde_json::Value> = column_names
        .iter()
        .map(|name| {
            serde_json::json!({
                "name": name
            })
        })
        .collect();

    let result = serde_json::json!({
        "columns": columns_info,
        "rows": rows_data,
        "row_count": rows_data.len()
    });

    Ok(result.to_string())
}

#[tauri::command]
async fn get_query_row_count(node_graph: String) -> Result<i64, String> {
    let graph: query_builder::NodeGraph = serde_json::from_str(&node_graph)
        .map_err(|e| format!("Failed to parse node graph: {}", e))?;

    let sql = query_builder::generate_sql(&graph, None)?;

    let count_sql = format!("SELECT COUNT(*) FROM ({}) AS subquery", sql);

    let conn = duckdb_connect().map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row(&count_sql, [], |row| row.get(0))
        .map_err(|e| format!("Failed to get row count: {}", e))?;

    Ok(count)
}

use duckdb::Connection;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::{path::BaseDirectory, Manager};

static APP_DATA_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn set_app_data_path(path: PathBuf) {
    APP_DATA_PATH.set(path).ok();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_path = app.path().resolve("data", BaseDirectory::AppData)?;
            std::fs::create_dir_all(&app_data_path)?;
            set_app_data_path(app_data_path);
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            config,
            discovery,
            sync,
            tables,
            table_schema,
            run_query,
            get_query_row_count,
            drop_table
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use deno_core::{error::AnyError, JsRuntime, RuntimeOptions};
    use std::rc::Rc;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_test_env() {
        INIT.call_once(|| {
            let temp_dir = std::env::temp_dir().join("streaksight_test");
            let _ = std::fs::remove_file(temp_dir.join("database.duckdb"));
            let _ = std::fs::remove_file(temp_dir.join("database.duckdb.wal"));
            std::fs::create_dir_all(&temp_dir).unwrap();
            set_app_data_path(temp_dir);
        });
    }

    async fn run_plugin_test(test_csv_path: &str) -> Result<(), AnyError> {
        let current_dir = std::env::current_dir()?;
        let connector_path = current_dir.join("src/LocalFileCSVConnector.js");

        let test_js = format!(
            r#"import {{ discovery, sync }} from "{}";
               export async function main() {{
                   const config = {{
                       filePath: "{}",
                       delimiter: ",",
                       hasHeader: true
                   }};
                   const schema = await discovery(config);
                   console.log("Schema:", JSON.stringify(schema));
                   await sync("test_csv_table", config, schema);
                   console.log("Sync completed");
               }}"#,
            connector_path.to_str().unwrap().replace("\\", "/"),
            test_csv_path
        );

        let test_js_path = std::env::temp_dir()
            .join("streaksight_test")
            .join("test_plugin.js");
        std::fs::write(&test_js_path, test_js)?;

        let plugin_module =
            deno_core::resolve_path(test_js_path.to_str().unwrap(), &std::env::current_dir()?)?;

        let mut rt = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            extensions: vec![streaksight_ext::init()],
            ..Default::default()
        });

        let id = rt.load_main_es_module(&plugin_module).await?;
        let eval = rt.mod_evaluate(id);
        rt.run_event_loop(Default::default()).await?;
        eval.await?;

        let script = format!(
            r#"import("{}").then(m => m.main())"#,
            plugin_module.as_str()
        );
        rt.execute_script("<call_main>", script)?;
        rt.run_event_loop(Default::default()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_csv_connector_sync() {
        setup_test_env();

        // Create test CSV file
        let test_csv = r#"id,name,active,score
1,Alice,true,95.5
2,Bob,false,87.3
3,Charlie,true,92.1"#;

        let temp_dir = std::env::temp_dir().join("streaksight_test");
        let csv_path = temp_dir.join("test_sync.csv");
        std::fs::write(&csv_path, test_csv).unwrap();

        run_plugin_test(csv_path.to_str().unwrap()).await.unwrap();

        let conn = duckdb_connect().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, active, score FROM test_csv_table ORDER BY id")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();

        let row = rows.next().unwrap().unwrap();
        let id: f64 = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let active: bool = row.get(2).unwrap();
        let score: f64 = row.get(3).unwrap();
        assert_eq!(id, 1.0);
        assert_eq!(name, "Alice");
        assert_eq!(active, true);
        assert_eq!(score, 95.5);

        let row = rows.next().unwrap().unwrap();
        let id: f64 = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let active: bool = row.get(2).unwrap();
        let score: f64 = row.get(3).unwrap();
        assert_eq!(id, 2.0);
        assert_eq!(name, "Bob");
        assert_eq!(active, false);
        assert_eq!(score, 87.3);

        let row = rows.next().unwrap().unwrap();
        let id: f64 = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let active: bool = row.get(2).unwrap();
        let score: f64 = row.get(3).unwrap();
        assert_eq!(id, 3.0);
        assert_eq!(name, "Charlie");
        assert_eq!(active, true);
        assert_eq!(score, 92.1);

        assert!(rows.next().unwrap().is_none());
    }

    #[test]
    fn test_resolve_connector_path_csv() {
        let result = resolve_connector_path(connector_type::LOCAL_FILE_CSV);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().contains("LocalFileCSVConnector.js"));
    }

    #[test]
    fn test_resolve_connector_path_json() {
        let result = resolve_connector_path(connector_type::LOCAL_FILE_JSON);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().contains("LocalFileJSONConnector.js"));
    }

    #[test]
    fn test_resolve_connector_path_unknown() {
        let result = resolve_connector_path("UnknownType");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown connector type");
    }

    #[test]
    fn test_duckdb_value_to_json_null() {
        let value = duckdb_value_to_json(duckdb::types::ValueRef::Null);
        assert_eq!(value, serde_json::Value::Null);
    }

    #[test]
    fn test_duckdb_value_to_json_boolean() {
        let value = duckdb_value_to_json(duckdb::types::ValueRef::Boolean(true));
        assert_eq!(value, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_duckdb_value_to_json_int() {
        let value = duckdb_value_to_json(duckdb::types::ValueRef::Int(42));
        assert_eq!(value, serde_json::json!(42));
    }

    #[test]
    fn test_duckdb_value_to_json_text() {
        let text = b"hello";
        let value = duckdb_value_to_json(duckdb::types::ValueRef::Text(text));
        assert_eq!(value, serde_json::Value::String("hello".to_string()));
    }

    #[test]
    fn test_duckdb_value_to_json_date32() {
        let value = duckdb_value_to_json(duckdb::types::ValueRef::Date32(0));
        assert_eq!(value, serde_json::Value::String("1970-01-01".to_string()));
    }
}
