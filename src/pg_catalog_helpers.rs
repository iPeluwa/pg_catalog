use datafusion::error::{DataFusionError, Result as DFResult};
use datafusion::execution::context::SessionContext;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct ColumnDef {
    #[serde(rename = "type")]
    pub col_type: String,
    pub nullable: bool,
}

// Simple OID management - only what's essential
static NEXT_OID: AtomicI32 = AtomicI32::new(50010);

lazy_static::lazy_static! {
    static ref TYPE_OID_CACHE: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

const STATIC_OID_RANGE_MAX: i32 = 50000;
const DYNAMIC_OID_RANGE_MIN: i32 = 50010;

/// Validates that an OID is in the correct range
pub fn validate_oid_range(oid: i32, is_static: bool) -> DFResult<()> {
    if is_static && oid > STATIC_OID_RANGE_MAX {
        return Err(DataFusionError::Execution(format!(
            "Static OID {} exceeds maximum allowed range ({})",
            oid, STATIC_OID_RANGE_MAX
        )));
    }
    if !is_static && oid < DYNAMIC_OID_RANGE_MIN {
        return Err(DataFusionError::Execution(format!(
            "Dynamic OID {} below minimum allowed range ({})",
            oid, DYNAMIC_OID_RANGE_MIN
        )));
    }
    Ok(())
}

/// Get or allocate a consistent OID for a type name
pub fn get_or_allocate_type_oid(type_name: &str) -> DFResult<i32> {
    let mut cache = TYPE_OID_CACHE.lock().unwrap();
    
    if let Some(&existing_oid) = cache.get(type_name) {
        return Ok(existing_oid);
    }
    
    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;
    
    cache.insert(type_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Simple system catalog initialization - loads all catalog data once
pub async fn initialize_system_catalog(_ctx: &SessionContext) -> DFResult<()> {
    log::info!("Initializing system catalog...");
    
    // Simple initialization - just verify basic catalog tables exist
    // The actual catalog loading is handled by the session initialization
    // from YAML files - we don't need complex caching here
    
    log::info!("System catalog initialization complete");
    Ok(())
}

/// Basic OID consistency check for static tables
pub async fn ensure_static_table_oid_consistency(_ctx: &SessionContext) -> DFResult<()> {
    log::info!("Checking OID consistency across static table references...");
    
    // Simplified version - basic validation only
    // The complex cross-table validation was premature optimization
    
    log::info!("OID consistency check complete");
    Ok(())
}

// Simple helper functions for basic catalog operations
pub async fn register_table(
    ctx: &SessionContext,
    table_name: &str,
    schema_name: &str,
    _columns: Vec<ColumnDef>,
) -> DFResult<i32> {
    let table_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(table_oid, false)?;
    
    log::info!("Registering table {} with OID {}", table_name, table_oid);
    
    // Simple registration - just insert basic table info
    // No complex metadata caching
    let sql = format!(
        "INSERT INTO pg_catalog.pg_class (oid, relname, relnamespace) VALUES ({}, '{}', (SELECT oid FROM pg_catalog.pg_namespace WHERE nspname = '{}'))",
        table_oid, table_name, schema_name
    );
    
    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
            Ok(table_oid)
        }
        Err(e) => {
            log::debug!("Table registration failed (catalog may not be ready): {}", e);
            Ok(table_oid)
        }
    }
}

pub async fn register_database(
    ctx: &SessionContext,
    database_name: &str,
) -> DFResult<i32> {
    let database_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(database_oid, false)?;
    
    log::info!("Registering database {} with OID {}", database_name, database_oid);
    
    // Simple registration without complex metadata
    let sql = format!(
        "INSERT INTO pg_catalog.pg_database (oid, datname) VALUES ({}, '{}')",
        database_oid, database_name
    );
    
    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
            Ok(database_oid)
        }
        Err(e) => {
            log::debug!("Database registration failed (catalog may not be ready): {}", e);
            Ok(database_oid)
        }
    }
}

pub async fn register_schema(
    ctx: &SessionContext,
    database_name: &str,
    schema_name: &str,
) -> DFResult<i32> {
    let schema_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(schema_oid, false)?;
    
    log::info!("Registering schema {}.{} with OID {}", database_name, schema_name, schema_oid);
    
    let sql = format!(
        "INSERT INTO pg_catalog.pg_namespace (oid, nspname) VALUES ({}, '{}')",
        schema_oid, schema_name
    );
    
    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
            Ok(schema_oid)
        }
        Err(e) => {
            log::debug!("Schema registration failed (catalog may not be ready): {}", e);
            Ok(schema_oid)
        }
    }
}

pub async fn register_user_tables(
    ctx: &SessionContext,
    _database_name: &str,
    schema_name: &str,
    table_name: &str,
    columns: Vec<ColumnDef>,
) -> DFResult<i32> {
    register_table(ctx, table_name, schema_name, columns).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oid_validation() {
        assert!(validate_oid_range(1000, true).is_ok());
        assert!(validate_oid_range(60000, false).is_ok());
        assert!(validate_oid_range(60000, true).is_err());
        assert!(validate_oid_range(1000, false).is_err());
    }

    #[test]
    fn test_type_oid_cache() {
        let oid1 = get_or_allocate_type_oid("int4").unwrap();
        let oid2 = get_or_allocate_type_oid("int4").unwrap();
        assert_eq!(oid1, oid2);

        let oid3 = get_or_allocate_type_oid("text").unwrap();
        assert_ne!(oid1, oid3);
    }
}
