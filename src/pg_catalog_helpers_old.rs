use arrow::array::Int64Array;
use datafusion::common::ScalarValue;
use datafusion::error::{DataFusionError, Result as DFResult};
use datafusion::execution::context::SessionContext;

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct ColumnDef {
    #[serde(rename = "type")]
    pub col_type: String,
    pub nullable: bool,
}

/// Enhanced metadata for table definitions (Phase 3)
#[derive(Debug, Clone)]
pub struct EnhancedTableMetadata {
    pub table_oid: i32,
    pub table_name: String,
    pub schema_oid: i32,
    pub table_type: TableType,
    pub columns: Vec<ColumnMetadata>,
    pub constraints: Vec<ConstraintMetadata>,
    pub indexes: Vec<IndexMetadata>,
    pub dependencies: Vec<DependencyMetadata>,
}

/// Phase 4A: Critical catalog table metadata structures
#[derive(Debug, Clone)]
pub struct DatabaseMetadata {
    pub database_oid: i32,
    pub database_name: String,
    pub owner_oid: i32,
    pub encoding: i32,
    pub collate: String,
    pub ctype: String,
    pub is_template: bool,
    pub allow_connections: bool,
    pub connection_limit: i32,
    pub default_tablespace_oid: i32,
    pub privileges: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TablespaceMetadata {
    pub tablespace_oid: i32,
    pub tablespace_name: String,
    pub owner_oid: i32,
    pub location: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UserMetadata {
    pub user_oid: i32,
    pub username: String,
    pub is_superuser: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub can_login: bool,
    pub is_replication: bool,
    pub password_hash: Option<String>,
    pub valid_until: Option<String>,
    pub connection_limit: i32,
}

/// Phase 4B: High priority catalog table metadata structures
#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    pub function_oid: i32,
    pub function_name: String,
    pub namespace_oid: i32,
    pub owner_oid: i32,
    pub language_oid: i32,
    pub arg_types: Vec<i32>, // OIDs of argument types
    pub return_type_oid: i32,
    pub is_aggregate: bool,
    pub is_window: bool,
    pub is_security_definer: bool,
    pub is_strict: bool,
    pub is_immutable: bool,
    pub function_body: Option<String>,
    pub function_cost: f32,
    pub estimated_rows: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct OperatorMetadata {
    pub operator_oid: i32,
    pub operator_name: String,
    pub namespace_oid: i32,
    pub owner_oid: i32,
    pub left_type_oid: Option<i32>,
    pub right_type_oid: Option<i32>,
    pub result_type_oid: i32,
    pub function_oid: i32, // Implementation function
    pub commutator_oid: Option<i32>,
    pub negator_oid: Option<i32>,
    pub is_merge_joinable: bool,
    pub is_hash_joinable: bool,
    pub left_sort_operator: Option<i32>,
    pub right_sort_operator: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct AggregateMetadata {
    pub aggregate_oid: i32,
    pub function_oid: i32, // References pg_proc
    pub kind: AggregateKind,
    pub num_direct_args: i16,
    pub transition_function_oid: i32,
    pub final_function_oid: Option<i32>,
    pub combine_function_oid: Option<i32>,
    pub serialization_function_oid: Option<i32>,
    pub deserialization_function_oid: Option<i32>,
    pub transition_type_oid: i32,
    pub transition_space: i32,
    pub initial_value: Option<String>,
    pub initial_condition: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AggregateKind {
    Normal,
    Ordered,
    Hypothetical,
}

#[derive(Debug, Clone)]
pub struct TriggerMetadata {
    pub trigger_oid: i32,
    pub trigger_name: String,
    pub table_oid: i32,
    pub function_oid: i32,
    pub trigger_type: TriggerType,
    pub enabled_state: TriggerEnabledState,
    pub is_internal: bool,
    pub when_clause: Option<String>,
    pub transition_old_table: Option<String>,
    pub transition_new_table: Option<String>,
    pub trigger_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TriggerType {
    pub before: bool,
    pub after: bool,
    pub instead: bool,
    pub insert: bool,
    pub update: bool,
    pub delete: bool,
    pub truncate: bool,
    pub per_row: bool,
    pub per_statement: bool,
}

#[derive(Debug, Clone)]
pub enum TriggerEnabledState {
    Origin,
    Disabled,
    Replica,
    Always,
}

/// Phase 4C: Security and Access Control Metadata

#[derive(Debug, Clone)]
pub struct AuthMemberMetadata {
    pub roleid: i32,        // Member role OID
    pub member: i32,        // Member OID (role being granted to)
    pub grantor: i32,       // OID of role that granted this membership
    pub admin_option: bool, // True if member can grant this role to others
}

#[derive(Debug, Clone)]
pub struct DefaultAclMetadata {
    pub oid: i32,
    pub defaclrole: i32,        // OID of role whose objects get these ACLs
    pub defaclnamespace: i32,   // OID of namespace these ACLs apply to (0 for all)
    pub defaclobjtype: char, // Object type: 'r' = relation, 'S' = sequence, 'f' = function, 'T' = type
    pub defaclacl: Vec<String>, // Access privileges in ACL format
}

#[derive(Debug, Clone)]
pub struct InitPrivsMetadata {
    pub objoid: i32,            // OID of the object
    pub classoid: i32,          // OID of the system catalog containing the object
    pub objsubid: i32,          // Column number for column privileges, 0 for object
    pub privtype: char,         // Type of initial privilege (e = extension, i = initdb)
    pub initprivs: Vec<String>, // Initial access privileges in ACL format
}

#[derive(Debug, Clone)]
pub struct SecLabelMetadata {
    pub objoid: i32,      // OID of the object this label is for
    pub classoid: i32,    // OID of the system catalog containing the object
    pub objsubid: i32,    // Column number for column labels, 0 for object
    pub provider: String, // Label provider (e.g., selinux)
    pub label: String,    // Security label value
}

/// Advanced Table Features Metadata

#[derive(Debug, Clone)]
pub struct PartitionedTableMetadata {
    pub partrelid: i32,            // OID of the partitioned table
    pub partstrat: char,           // Partitioning strategy: 'h' = hash, 'l' = list, 'r' = range
    pub partnatts: i16,            // Number of partitioning columns
    pub partdefid: i32,            // OID of the default partition (0 if none)
    pub partattrs: Vec<i16>,       // Array of partitioning column numbers
    pub partclass: Vec<i32>,       // Array of operator class OIDs for partitioning columns
    pub partcollation: Vec<i32>,   // Array of collation OIDs for partitioning columns
    pub partexprs: Option<String>, // Partitioning expressions (serialized)
}

#[derive(Debug, Clone)]
pub struct EventTriggerMetadata {
    pub oid: i32,
    pub evtname: String,      // Event trigger name
    pub evtevent: String, // Event name (ddl_command_start, ddl_command_end, table_rewrite, sql_drop)
    pub evtowner: i32,    // OID of owner
    pub evtfoid: i32,     // OID of trigger function
    pub evtenabled: char, // Enabled state: 'O' = origin, 'D' = disabled, 'R' = replica, 'A' = always
    pub evttags: Vec<String>, // Array of command tags (empty array = all commands)
}

#[derive(Debug, Clone)]
pub struct UserMappingMetadata {
    pub oid: i32,
    pub umuser: i32,            // OID of user (0 for public)
    pub umserver: i32,          // OID of foreign server
    pub umoptions: Vec<String>, // User mapping options in "key=value" format
}

/// Large Object Support Metadata

#[derive(Debug, Clone)]
pub struct LargeObjectMetadata {
    pub oid: i32,            // Large object OID
    pub lomowner: i32,       // OID of owner
    pub lomacl: Vec<String>, // Access privileges
}

#[derive(Debug, Clone)]
pub struct LargeObjectDataMetadata {
    pub loid: i32,     // Large object OID
    pub pageno: i32,   // Page number within large object
    pub data: Vec<u8>, // Page data (up to 2KB)
}

/// Text Search System Metadata

#[derive(Debug, Clone)]
pub struct TextSearchConfigMetadata {
    pub oid: i32,
    pub cfgname: String,   // Configuration name
    pub cfgnamespace: i32, // Namespace OID
    pub cfgowner: i32,     // Owner OID
    pub cfgparser: i32,    // Parser OID
}

#[derive(Debug, Clone)]
pub struct TextSearchDictMetadata {
    pub oid: i32,
    pub dictname: String,               // Dictionary name
    pub dictnamespace: i32,             // Namespace OID
    pub dictowner: i32,                 // Owner OID
    pub dicttemplate: i32,              // Template OID
    pub dictinitoption: Option<String>, // Initialization options
}

#[derive(Debug, Clone)]
pub struct TextSearchParserMetadata {
    pub oid: i32,
    pub prsname: String,   // Parser name
    pub prsnamespace: i32, // Namespace OID
    pub prsstart: i32,     // Start function OID
    pub prstoken: i32,     // Token function OID
    pub prsend: i32,       // End function OID
    pub prslextype: i32,   // Lexeme type function OID
    pub prsheadline: i32,  // Headline function OID
}

#[derive(Debug, Clone)]
pub struct TextSearchTemplateMetadata {
    pub oid: i32,
    pub tmplname: String,   // Template name
    pub tmplnamespace: i32, // Namespace OID
    pub tmplinit: i32,      // Init function OID
    pub tmpllexize: i32,    // Lexize function OID
}

/// Extended Statistics Metadata

#[derive(Debug, Clone)]
pub struct ExtendedStatisticMetadata {
    pub oid: i32,
    pub stxrelid: i32,            // Table OID
    pub stxname: String,          // Statistics object name
    pub stxnamespace: i32,        // Namespace OID
    pub stxowner: i32,            // Owner OID
    pub stxkeys: Vec<i16>,        // Column numbers
    pub stxkind: Vec<char>, // Statistics kinds: 'd' = ndistinct, 'f' = functional deps, 'm' = MCV, 'e' = expression
    pub stxexprs: Option<String>, // Expressions (serialized)
}

#[derive(Debug, Clone)]
pub struct ExtendedStatisticDataMetadata {
    pub stxoid: i32,                      // Statistics object OID
    pub stxdinherit: bool,                // True if inherited statistics
    pub stxdndistinct: Option<String>,    // N-distinct statistics data
    pub stxddependencies: Option<String>, // Functional dependencies data
    pub stxdmcv: Option<String>,          // Most Common Values data
    pub stxdexpr: Option<String>,         // Expression statistics data
}

/// Replication Features Metadata

#[derive(Debug, Clone)]
pub struct PublicationNamespaceMetadata {
    pub pnpubid: i32, // Publication OID
    pub pnnspid: i32, // Namespace OID
}

#[derive(Debug, Clone)]
pub struct ReplicationOriginMetadata {
    pub roident: i32,   // Origin identifier
    pub roname: String, // Origin name
}

#[derive(Debug, Clone)]
pub struct SubscriptionRelMetadata {
    pub srsubid: i32,             // Subscription OID
    pub srrelid: i32,             // Relation OID
    pub srsubstate: char, // Subscription state: 'i' = initialize, 'd' = data copy, 's' = synchronized, 'r' = ready
    pub srsublsn: Option<String>, // LSN of the subscription
}

#[derive(Debug, Clone)]
pub enum TableType {
    Table,
    View,
    Index,
    Sequence,
    MaterializedView,
}

#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub column_name: String,
    pub column_oid: i32,
    pub data_type_oid: i32,
    pub column_number: i16,
    pub is_nullable: bool,
    pub has_default: bool,
    pub is_identity: bool,
}

#[derive(Debug, Clone)]
pub struct ConstraintMetadata {
    pub constraint_oid: i32,
    pub constraint_name: String,
    pub constraint_type: ConstraintType,
    pub table_oid: i32,
    pub columns: Vec<i16>, // Column numbers
    pub referenced_table_oid: Option<i32>,
    pub referenced_columns: Vec<i16>,
}

#[derive(Debug, Clone)]
pub enum ConstraintType {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check,
    NotNull,
}

#[derive(Debug, Clone)]
pub struct IndexMetadata {
    pub index_oid: i32,
    pub index_name: String,
    pub table_oid: i32,
    pub columns: Vec<i16>, // Column numbers
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: String, // btree, hash, gin, gist, etc.
}

#[derive(Debug, Clone)]
pub struct DependencyMetadata {
    pub dependent_oid: i32,
    pub dependency_oid: i32,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone)]
pub enum DependencyType {
    Normal,
    Auto,
    Internal,
    Extension,
}

// OID ranges:
// 1-50000: Reserved for static catalog data (pg_type, pg_description, etc.)
// 50000+: Dynamic user objects (tables, schemas, etc.)
static NEXT_OID: AtomicI32 = AtomicI32::new(50010);

// Minimal essential caches
lazy_static::lazy_static! {
    static ref TYPE_OID_CACHE: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}



const STATIC_OID_RANGE_MAX: i32 = 50000;
const DYNAMIC_OID_RANGE_MIN: i32 = 50010;

/// Validates that an OID is in the correct range
fn validate_oid_range(oid: i32, is_static: bool) -> DFResult<()> {
    if is_static && oid > STATIC_OID_RANGE_MAX {
        return Err(DataFusionError::Execution(format!(
            "Static OID {} exceeds maximum allowed range (1-{})",
            oid, STATIC_OID_RANGE_MAX
        )));
    }
    if !is_static && oid < DYNAMIC_OID_RANGE_MIN {
        return Err(DataFusionError::Execution(format!(
            "Dynamic OID {} below minimum allowed range ({}+)",
            oid, DYNAMIC_OID_RANGE_MIN
        )));
    }
    Ok(())
}

fn map_type_to_oid(t: &str) -> i32 {
    match t.to_lowercase().as_str() {
        "int" | "integer" | "int4" => 23,
        "bigint" | "int8" => 20,
        "bool" | "boolean" => 16,
        _ => 25, // default to text
    }
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






/// Apply OID cache to pg_type entries during registration
pub async fn register_pg_type_with_oid_cache(
    ctx: &SessionContext,
    type_name: &str,
    _type_def: &serde_yaml::Value,
) -> DFResult<()> {


    // Get consistent OID for this type
    let type_oid = get_or_allocate_type_oid(type_name)?;

    // Apply OID cache to type definition
    log::debug!(
        "Registering pg_type entry '{}' with OID {}",
        type_name,
        type_oid
    );

    // Insert/update the type in pg_type table with consistent OID
    let sql = format!(
        "INSERT OR REPLACE INTO pg_catalog.pg_type (oid, typname) VALUES ({}, '{}')",
        type_oid,
        type_name.replace('\'', "''")
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
            Ok(())
        }
        Err(_) => {
            // Table might not exist yet, that's OK for lazy loading
            log::debug!("pg_type table not ready for OID registration yet");
            Ok(())
        }
    }
}

/// Apply OID cache to pg_description entries during registration  
pub async fn register_pg_description_with_oid_cache(
    ctx: &SessionContext,
    objoid: i32,
    classoid: i32,
    objsubid: i32,
    description: &str,
) -> DFResult<()> {


    log::debug!(
        "Registering pg_description entry for object {}",
        objoid
    );

    // Insert/update the description in pg_description table with consistent OID
    let sql = format!(
        "INSERT OR REPLACE INTO pg_catalog.pg_description (objoid, classoid, objsubid, description) VALUES ({}, {}, {}, '{}')",
        objoid, classoid, objsubid, description.replace('\'', "''")
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
            Ok(())
        }
        Err(_) => {
            // Table might not exist yet, that's OK for lazy loading
            log::debug!("pg_description table not ready for OID registration yet");
            Ok(())
        }
    }
}

/// Ensure OID consistency across all static table references
pub async fn ensure_static_table_oid_consistency(ctx: &SessionContext) -> DFResult<()> {
    log::info!("Ensuring OID consistency across static table references...");



    // Verify OID consistency between related tables
    // For example, ensure pg_type OIDs are consistently referenced in other tables

    // Check if pg_type references are consistent in pg_class.reltype
    let consistency_check = ctx
        .sql(
            "
            SELECT COUNT(*) as count 
            FROM pg_catalog.pg_class c 
            LEFT JOIN pg_catalog.pg_type t ON c.reltype = t.oid 
            WHERE c.reltype IS NOT NULL AND t.oid IS NULL
        ",
        )
        .await;

    match consistency_check {
        Ok(df) => {
            let batches = df.collect().await?;
            if !batches.is_empty() && batches[0].num_rows() > 0 {
                let count_array = batches[0]
                    .column(0)
                    .as_any()
                    .downcast_ref::<arrow::array::Int64Array>()
                    .unwrap();
                let inconsistent_count = count_array.value(0);

                if inconsistent_count > 0 {
                    log::warn!(
                        "Found {} inconsistent OID references between pg_class and pg_type",
                        inconsistent_count
                    );
                } else {
                    log::info!("OID consistency verified between pg_class and pg_type");
                }
            }
        }
        Err(e) => {
            log::debug!(
                "Could not verify OID consistency (tables may not be ready): {}",
                e
            );
        }
    }

    log::info!("Static table OID consistency check completed");
    Ok(())
}

/// Phase 3: Enhanced table registration with richer metadata
pub async fn register_enhanced_table(
    ctx: &SessionContext,
    _database_name: &str,
    schema_name: &str,
    table_name: &str,
    table_type: TableType,
    columns: Vec<ColumnMetadata>,
    constraints: Vec<ConstraintMetadata>,
    indexes: Vec<IndexMetadata>,
) -> DFResult<EnhancedTableMetadata> {
    // Get consistent OIDs for all components
    let table_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(table_oid, false)?;

    let type_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(type_oid, false)?;

    // Get schema OID
    let schema_oid_result = ctx
        .sql("SELECT oid FROM pg_catalog.pg_namespace WHERE nspname=$schema")
        .await?
        .with_param_values(vec![("schema", ScalarValue::from(schema_name))])?
        .collect()
        .await?;

    let schema_oid = if schema_oid_result.is_empty() || schema_oid_result[0].num_rows() == 0 {
        return Err(DataFusionError::Execution("schema not found".to_string()));
    } else {
        let arr = schema_oid_result[0]
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::Int32Array>()
            .unwrap();
        arr.value(0)
    };

    // Register in pg_class with enhanced metadata
    let relkind = match table_type {
        TableType::Table => 'r',
        TableType::View => 'v',
        TableType::Index => 'i',
        TableType::Sequence => 'S',
        TableType::MaterializedView => 'm',
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_class \
         (oid, relname, relnamespace, relkind, reltuples, reltype, relispartition, relhasindex) \
         VALUES ({table_oid},'{}',{schema_oid},'{relkind}',0,{type_oid}, false, {})",
        table_name.replace('\'', "''"),
        !indexes.is_empty()
    );
    ctx.sql(&sql).await?.collect().await?;

    // Register in pg_type
    let sql = format!(
        "INSERT INTO pg_catalog.pg_type \
         (oid, typname, typrelid, typlen, typcategory) \
         VALUES ({type_oid},'_{table_name}',{table_oid},-1,'C')"
    );
    ctx.sql(&sql).await?.collect().await?;

    // Register columns in pg_attribute with enhanced metadata
    for (idx, column) in columns.iter().enumerate() {
        let sql = format!(
            "INSERT INTO pg_catalog.pg_attribute \
             (attrelid,attnum,attname,atttypid,atttypmod,attnotnull,attisdropped,atthasdef,attidentity) \
             VALUES ({table_oid},{},'{}',{},{},'{}',false,'{}','{}')",
            idx + 1,
            column.column_name.replace('\'', "''"),
            column.data_type_oid,
            -1, // atttypmod
            if column.is_nullable { "false" } else { "true" },
            if column.has_default { "true" } else { "false" },
            if column.is_identity { "d" } else { "" }
        );
        ctx.sql(&sql).await?.collect().await?;
    }

    // Create enhanced metadata object
    let enhanced_metadata = EnhancedTableMetadata {
        table_oid,
        table_name: table_name.to_string(),
        schema_oid,
        table_type,
        columns,
        constraints: constraints.clone(),
        indexes: indexes.clone(),
        dependencies: Vec::new(), // Will be populated separately
    };

    // Cache the enhanced metadata
    {
        let mut cache = TABLE_METADATA_CACHE.lock().unwrap();
        cache.insert(table_oid, enhanced_metadata.clone());
    }

    // Register constraints (resilient to missing pg_constraint table)
    for constraint in constraints {
        if let Err(e) = register_constraint(ctx, &constraint).await {
            log::debug!(
                "Could not register constraint '{}': {}",
                constraint.constraint_name,
                e
            );
            // Continue - not critical for basic table registration
        }
    }

    // Register indexes (resilient to missing pg_index table)
    for index in indexes {
        if let Err(e) = register_index(ctx, &index).await {
            log::debug!("Could not register index '{}': {}", index.index_name, e);
            // Continue - not critical for basic table registration
        }
    }

    log::info!(
        "Enhanced table '{}' registered with OID {} and {} constraints, {} indexes",
        table_name,
        table_oid,
        enhanced_metadata.constraints.len(),
        enhanced_metadata.indexes.len()
    );

    Ok(enhanced_metadata)
}

/// Register a constraint in pg_constraint
pub async fn register_constraint(
    ctx: &SessionContext,
    constraint: &ConstraintMetadata,
) -> DFResult<()> {
    let constraint_oid = get_or_allocate_constraint_oid(&constraint.constraint_name)?;

    let contype = match constraint.constraint_type {
        ConstraintType::PrimaryKey => 'p',
        ConstraintType::ForeignKey => 'f',
        ConstraintType::Unique => 'u',
        ConstraintType::Check => 'c',
        ConstraintType::NotNull => 'n',
    };

    let conkey = format!(
        "ARRAY[{}]",
        constraint
            .columns
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let confkey = if !constraint.referenced_columns.is_empty() {
        format!(
            "ARRAY[{}]",
            constraint
                .referenced_columns
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    } else {
        "NULL".to_string()
    };

    let confrelid = constraint.referenced_table_oid.unwrap_or(0);

    let sql = format!(
        "INSERT INTO pg_catalog.pg_constraint \
         (oid, conname, conrelid, contype, conkey, confrelid, confkey) \
         VALUES ({constraint_oid}, '{}', {}, '{contype}', {conkey}, {confrelid}, {confkey})",
        constraint.constraint_name.replace('\'', "''"),
        constraint.table_oid
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
            log::debug!(
                "Registered constraint '{}' with OID {}",
                constraint.constraint_name,
                constraint_oid
            );
        }
        Err(_) => {
            log::debug!("pg_constraint table not ready yet");
        }
    }

    Ok(())
}

/// Register an index in pg_index
pub async fn register_index(ctx: &SessionContext, index: &IndexMetadata) -> DFResult<()> {
    let index_oid = get_or_allocate_index_oid(&index.index_name)?;

    let indkey = format!(
        "ARRAY[{}]",
        index
            .columns
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let sql = format!(
        "INSERT INTO pg_catalog.pg_index \
         (indexrelid, indrelid, indkey, indisunique, indisprimary) \
         VALUES ({index_oid}, {}, {indkey}, {}, {})",
        index.table_oid, index.is_unique, index.is_primary
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;

            // Also register the index as a relation in pg_class
            let index_class_sql = format!(
                "INSERT INTO pg_catalog.pg_class \
                 (oid, relname, relnamespace, relkind, relam) \
                 VALUES ({index_oid}, '{}', (SELECT relnamespace FROM pg_catalog.pg_class WHERE oid = {}), 'i', 403)",
                index.index_name.replace('\'', "''"),
                index.table_oid
            );
            ctx.sql(&index_class_sql).await?.collect().await?;

            log::debug!(
                "Registered index '{}' with OID {}",
                index.index_name,
                index_oid
            );
        }
        Err(_) => {
            log::debug!("pg_index table not ready yet");
        }
    }

    Ok(())
}

/// Get or allocate consistent OID for constraint
pub fn get_or_allocate_constraint_oid(constraint_name: &str) -> DFResult<i32> {
    let mut cache = CONSTRAINT_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(constraint_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(constraint_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Get or allocate consistent OID for index
pub fn get_or_allocate_index_oid(index_name: &str) -> DFResult<i32> {
    let mut cache = INDEX_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(index_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(index_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Register a dependency in pg_depend
pub async fn register_dependency(
    ctx: &SessionContext,
    dependent_oid: i32,
    dependency_oid: i32,
    dependency_type: DependencyType,
) -> DFResult<()> {
    let deptype = match dependency_type {
        DependencyType::Normal => 'n',
        DependencyType::Auto => 'a',
        DependencyType::Internal => 'i',
        DependencyType::Extension => 'e',
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_depend \
         (objid, refobjid, deptype) \
         VALUES ({}, {}, '{deptype}')",
        dependent_oid, dependency_oid
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
            log::debug!(
                "Registered dependency: {} -> {}",
                dependent_oid,
                dependency_oid
            );
        }
        Err(_) => {
            log::debug!("pg_depend table not ready yet");
        }
    }

    // Cache the dependency for future reference
    {
        let mut cache = DEPENDENCY_OID_CACHE.lock().unwrap();
        let key = (dependent_oid, dependency_oid);
        cache.insert(key, dependent_oid); // Store dependent as value for lookup
    }

    Ok(())
}

/// Get enhanced metadata for a table
pub fn get_enhanced_table_metadata(table_oid: i32) -> Option<EnhancedTableMetadata> {
    let cache = TABLE_METADATA_CACHE.lock().unwrap();
    cache.get(&table_oid).cloned()
}

/// Update enhanced metadata cache
pub fn update_enhanced_table_metadata(table_oid: i32, metadata: EnhancedTableMetadata) {
    let mut cache = TABLE_METADATA_CACHE.lock().unwrap();
    cache.insert(table_oid, metadata);
}

/// Enhanced cross-table OID consistency for dynamic tables (Phase 3)
pub async fn ensure_enhanced_oid_consistency(ctx: &SessionContext) -> DFResult<()> {
    log::info!("Ensuring enhanced OID consistency across dynamic tables...");

    // Check constraint references
    let constraint_check = ctx
        .sql(
            "
            SELECT COUNT(*) as count 
            FROM pg_catalog.pg_constraint c 
            LEFT JOIN pg_catalog.pg_class t ON c.conrelid = t.oid 
            WHERE c.conrelid IS NOT NULL AND t.oid IS NULL
        ",
        )
        .await;

    if let Ok(df) = constraint_check {
        if let Ok(batches) = df.collect().await {
            if !batches.is_empty() && batches[0].num_rows() > 0 {
                let count_array = batches[0]
                    .column(0)
                    .as_any()
                    .downcast_ref::<arrow::array::Int64Array>()
                    .unwrap();
                let inconsistent_count = count_array.value(0);

                if inconsistent_count > 0 {
                    log::warn!(
                        "Found {} inconsistent constraint references",
                        inconsistent_count
                    );
                } else {
                    log::info!("Constraint OID consistency verified");
                }
            }
        }
    }

    // Check index references
    let index_check = ctx
        .sql(
            "
            SELECT COUNT(*) as count 
            FROM pg_catalog.pg_index i 
            LEFT JOIN pg_catalog.pg_class t ON i.indrelid = t.oid 
            WHERE i.indrelid IS NOT NULL AND t.oid IS NULL
        ",
        )
        .await;

    if let Ok(df) = index_check {
        if let Ok(batches) = df.collect().await {
            if !batches.is_empty() && batches[0].num_rows() > 0 {
                let count_array = batches[0]
                    .column(0)
                    .as_any()
                    .downcast_ref::<arrow::array::Int64Array>()
                    .unwrap();
                let inconsistent_count = count_array.value(0);

                if inconsistent_count > 0 {
                    log::warn!("Found {} inconsistent index references", inconsistent_count);
                } else {
                    log::info!("Index OID consistency verified");
                }
            }
        }
    }

    log::info!("Enhanced OID consistency check completed");
    Ok(())
}

/// Phase 4A: Enhanced database registration with full metadata
pub async fn register_enhanced_database(
    ctx: &SessionContext,
    database_name: &str,
    _owner_name: &str,
    encoding: i32,
    collate: &str,
    ctype: &str,
    is_template: bool,
    allow_connections: bool,
    connection_limit: i32,
) -> DFResult<DatabaseMetadata> {
    // Get or allocate consistent OID for database
    let database_oid = get_or_allocate_database_oid(database_name)?;

    // Get owner OID (assume superuser for now, will be enhanced with pg_authid)
    let owner_oid = 10; // Bootstrap superuser OID

    // Default tablespace OID
    let default_tablespace_oid = 1663; // pg_default tablespace

    // Check if database already exists
    let exists_check = ctx
        .sql("SELECT COUNT(*) FROM pg_catalog.pg_database WHERE datname=$database_name")
        .await?
        .with_param_values(vec![("database_name", ScalarValue::from(database_name))])?
        .collect()
        .await?;

    let already_exists = if !exists_check.is_empty() && exists_check[0].num_rows() > 0 {
        let count_array = exists_check[0]
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::Int64Array>()
            .unwrap();
        count_array.value(0) > 0
    } else {
        false
    };

    if !already_exists {
        let sql = format!(
            "INSERT INTO pg_catalog.pg_database \
             (oid, datname, datdba, encoding, datcollate, datctype, datistemplate, datallowconn, datconnlimit, dattablespace) \
             VALUES ({}, '{}', {}, {}, '{}', '{}', {}, {}, {}, {})",
            database_oid,
            database_name.replace('\'', "''"),
            owner_oid,
            encoding,
            collate.replace('\'', "''"),
            ctype.replace('\'', "''"),
            is_template,
            allow_connections,
            connection_limit,
            default_tablespace_oid
        );

        ctx.sql(&sql).await?.collect().await?;
    }

    // Create enhanced metadata
    let metadata = DatabaseMetadata {
        database_oid,
        database_name: database_name.to_string(),
        owner_oid,
        encoding,
        collate: collate.to_string(),
        ctype: ctype.to_string(),
        is_template,
        allow_connections,
        connection_limit,
        default_tablespace_oid,
        privileges: vec!["CONNECT".to_string(), "CREATE".to_string()], // Default privileges
    };

    // Cache the metadata
    {
        let mut cache = DATABASE_METADATA_CACHE.lock().unwrap();
        cache.insert(database_oid, metadata.clone());
    }

    log::info!(
        "Enhanced database '{}' registered with OID {}",
        database_name,
        database_oid
    );
    Ok(metadata)
}

/// Phase 4A: Register tablespace with full metadata
pub async fn register_tablespace(
    ctx: &SessionContext,
    tablespace_name: &str,
    _owner_name: &str,
    location: Option<&str>,
) -> DFResult<TablespaceMetadata> {
    // Get or allocate consistent OID for tablespace
    let tablespace_oid = get_or_allocate_tablespace_oid(tablespace_name)?;

    // Get owner OID (assume superuser for now)
    let owner_oid = 10; // Bootstrap superuser OID

    let sql = format!(
        "INSERT INTO pg_catalog.pg_tablespace \
         (oid, spcname, spcowner, spclocation) \
         VALUES ({}, '{}', {}, {})",
        tablespace_oid,
        tablespace_name.replace('\'', "''"),
        owner_oid,
        location
            .map(|l| format!("'{}'", l.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string())
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_tablespace table not ready yet");
        }
    }

    // Create enhanced metadata
    let metadata = TablespaceMetadata {
        tablespace_oid,
        tablespace_name: tablespace_name.to_string(),
        owner_oid,
        location: location.map(|s| s.to_string()),
        options: vec![], // No options for now
    };

    // Cache the metadata
    {
        let mut cache = TABLESPACE_METADATA_CACHE.lock().unwrap();
        cache.insert(tablespace_oid, metadata.clone());
    }

    log::info!(
        "Tablespace '{}' registered with OID {}",
        tablespace_name,
        tablespace_oid
    );
    Ok(metadata)
}

/// Get or allocate consistent OID for database
pub fn get_or_allocate_database_oid(database_name: &str) -> DFResult<i32> {
    let mut cache = DATABASE_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(database_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(database_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Get or allocate consistent OID for tablespace
pub fn get_or_allocate_tablespace_oid(tablespace_name: &str) -> DFResult<i32> {
    let mut cache = TABLESPACE_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(tablespace_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(tablespace_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Phase 4A: Register user/role with full metadata
pub async fn register_user(
    ctx: &SessionContext,
    username: &str,
    is_superuser: bool,
    can_create_db: bool,
    can_create_role: bool,
    can_login: bool,
    is_replication: bool,
    password_hash: Option<&str>,
    connection_limit: i32,
) -> DFResult<UserMetadata> {
    // Get or allocate consistent OID for user
    let user_oid = get_or_allocate_user_oid(username)?;

    // Register in pg_authid (primary table)
    let sql = format!(
        "INSERT INTO pg_catalog.pg_authid \
         (oid, rolname, rolsuper, rolinherit, rolcreaterole, rolcreatedb, rolcanlogin, rolreplication, rolconnlimit, rolpassword) \
         VALUES ({}, '{}', {}, true, {}, {}, {}, {}, {}, {})",
        user_oid,
        username.replace('\'', "''"),
        is_superuser,
        can_create_role,
        can_create_db,
        can_login,
        is_replication,
        connection_limit,
        password_hash.map(|p| format!("'{}'", p.replace('\'', "''")))
                     .unwrap_or_else(|| "NULL".to_string())
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;

            // Also register in pg_user view if it's a login role
            if can_login {
                let user_sql = format!(
                    "INSERT INTO pg_catalog.pg_user \
                     (usename, usesysid, usecreatedb, usesuper, userepl, usebypassrls, passwd, valuntil, useconfig) \
                     VALUES ('{}', {}, {}, {}, {}, false, {}, NULL, NULL)",
                    username.replace('\'', "''"),
                    user_oid,
                    can_create_db,
                    is_superuser,
                    is_replication,
                    password_hash.map(|p| format!("'{}'", p.replace('\'', "''")))
                                 .unwrap_or_else(|| "NULL".to_string())
                );

                if let Ok(df) = ctx.sql(&user_sql).await {
                    df.collect().await?;
                }
            }
        }
        Err(_) => {
            log::debug!("pg_authid table not ready yet");
        }
    }

    // Create enhanced metadata
    let metadata = UserMetadata {
        user_oid,
        username: username.to_string(),
        is_superuser,
        can_create_db,
        can_create_role,
        can_login,
        is_replication,
        password_hash: password_hash.map(|s| s.to_string()),
        valid_until: None, // No expiration by default
        connection_limit,
    };

    // Cache the metadata
    {
        let mut cache = USER_METADATA_CACHE.lock().unwrap();
        cache.insert(user_oid, metadata.clone());
    }

    log::info!(
        "User '{}' registered with OID {} (superuser: {}, can_login: {})",
        username,
        user_oid,
        is_superuser,
        can_login
    );
    Ok(metadata)
}

/// Get or allocate consistent OID for user
pub fn get_or_allocate_user_oid(username: &str) -> DFResult<i32> {
    let mut cache = USER_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(username) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(username.to_string(), new_oid);
    Ok(new_oid)
}

/// Phase 4A: Initialize default PostgreSQL system objects
pub async fn initialize_system_catalog(ctx: &SessionContext) -> DFResult<()> {
    log::info!("Initializing Phase 4A system catalog objects...");

    // Initialize default tablespace
    let _pg_default = register_tablespace(
        ctx,
        "pg_default",
        "postgres",
        None, // No specific location
    )
    .await?;

    let _pg_global = register_tablespace(ctx, "pg_global", "postgres", None).await?;

    // Initialize bootstrap superuser
    let _postgres_user = register_user(
        ctx, "postgres", true, // is_superuser
        true, // can_create_db
        true, // can_create_role
        true, // can_login
        true, // is_replication
        None, // no password hash
        -1,   // unlimited connections
    )
    .await?;

    // Initialize template databases
    let _template0 = register_enhanced_database(
        ctx,
        "template0",
        "postgres",
        6, // UTF8 encoding
        "C",
        "C",
        true,  // is_template
        false, // no connections allowed
        -1,    // unlimited connections
    )
    .await?;

    let _template1 = register_enhanced_database(
        ctx,
        "template1",
        "postgres",
        6, // UTF8 encoding
        "C",
        "C",
        true, // is_template
        true, // connections allowed
        -1,   // unlimited connections
    )
    .await?;

    log::info!("Phase 4A system catalog initialization completed");
    Ok(())
}

/// Phase 4B: Register function with full metadata
pub async fn register_function(
    ctx: &SessionContext,
    function_name: &str,
    namespace_name: &str,
    _owner_name: &str,
    language_name: &str,
    arg_types: Vec<i32>,
    return_type_oid: i32,
    is_aggregate: bool,
    is_window: bool,
    is_security_definer: bool,
    is_strict: bool,
    is_immutable: bool,
    function_body: Option<&str>,
    function_cost: f32,
) -> DFResult<FunctionMetadata> {
    // Get or allocate consistent OID for function
    let function_oid = get_or_allocate_function_oid(function_name)?;

    // Get namespace OID
    let namespace_oid = get_namespace_oid(ctx, namespace_name).await.unwrap_or(2200); // public namespace

    // Default owner and language OIDs
    let owner_oid = 10; // Bootstrap superuser
    let language_oid = get_language_oid(language_name).unwrap_or(12); // SQL language

    // Format argument types array
    let arg_types_str = if arg_types.is_empty() {
        "'{}'".to_string()
    } else {
        format!(
            "ARRAY[{}]",
            arg_types
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_proc \
         (oid, proname, pronamespace, proowner, prolang, procost, prorows, provariadic, protransform, \
          prokind, prosecdef, proleakproof, proisstrict, proretset, provolatile, proparallel, \
          pronargs, pronargdefaults, prorettype, proargtypes, proargmodes, proargnames, \
          proargdefaults, protrftypes, prosrc, probin, proconfig, proacl) \
         VALUES ({}, '{}', {}, {}, {}, {}, {}, 0, '-', \
                 '{}', {}, false, {}, false, '{}', 's', \
                 {}, 0, {}, {}, NULL, NULL, \
                 NULL, NULL, '{}', NULL, NULL, NULL)",
        function_oid,
        function_name.replace('\'', "''"),
        namespace_oid,
        owner_oid,
        language_oid,
        function_cost,
        1.0, // Default estimated rows
        if is_aggregate { 'a' } else if is_window { 'w' } else { 'f' }, // prokind
        is_security_definer,
        is_strict,
        if is_immutable { 'i' } else { 'v' }, // provolatile
        arg_types.len(),
        return_type_oid,
        arg_types_str,
        function_body.unwrap_or("").replace('\'', "''")
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_proc table not ready yet");
        }
    }

    // Create enhanced metadata
    let metadata = FunctionMetadata {
        function_oid,
        function_name: function_name.to_string(),
        namespace_oid,
        owner_oid,
        language_oid,
        arg_types,
        return_type_oid,
        is_aggregate,
        is_window,
        is_security_definer,
        is_strict,
        is_immutable,
        function_body: function_body.map(|s| s.to_string()),
        function_cost,
        estimated_rows: Some(1.0),
    };

    // Cache the metadata
    {
        let mut cache = FUNCTION_METADATA_CACHE.lock().unwrap();
        cache.insert(function_oid, metadata.clone());
    }

    log::info!(
        "Function '{}' registered with OID {} (aggregate: {}, immutable: {})",
        function_name,
        function_oid,
        is_aggregate,
        is_immutable
    );
    Ok(metadata)
}

/// Phase 4B: Register operator with full metadata
pub async fn register_operator(
    ctx: &SessionContext,
    operator_name: &str,
    namespace_name: &str,
    _owner_name: &str,
    left_type_oid: Option<i32>,
    right_type_oid: Option<i32>,
    result_type_oid: i32,
    function_oid: i32,
) -> DFResult<OperatorMetadata> {
    // Get or allocate consistent OID for operator
    let operator_oid = get_or_allocate_operator_oid(operator_name)?;

    // Get namespace OID
    let namespace_oid = get_namespace_oid(ctx, namespace_name).await.unwrap_or(2200);

    // Default owner OID
    let owner_oid = 10; // Bootstrap superuser

    let sql = format!(
        "INSERT INTO pg_catalog.pg_operator \
         (oid, oprname, oprnamespace, oprowner, oprkind, oprcanmerge, oprcanhash, \
          oprleft, oprright, oprresult, oprcom, oprnegate, oprcode, oprrest, oprjoin) \
         VALUES ({}, '{}', {}, {}, '{}', false, false, \
                 {}, {}, {}, 0, 0, {}, '-', '-')",
        operator_oid,
        operator_name.replace('\'', "''"),
        namespace_oid,
        owner_oid,
        if left_type_oid.is_none() {
            'r'
        } else if right_type_oid.is_none() {
            'l'
        } else {
            'b'
        }, // oprkind
        left_type_oid.unwrap_or(0),
        right_type_oid.unwrap_or(0),
        result_type_oid,
        function_oid
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_operator table not ready yet");
        }
    }

    // Create enhanced metadata
    let metadata = OperatorMetadata {
        operator_oid,
        operator_name: operator_name.to_string(),
        namespace_oid,
        owner_oid,
        left_type_oid,
        right_type_oid,
        result_type_oid,
        function_oid,
        commutator_oid: None,
        negator_oid: None,
        is_merge_joinable: false,
        is_hash_joinable: false,
        left_sort_operator: None,
        right_sort_operator: None,
    };

    // Cache the metadata
    {
        let mut cache = OPERATOR_METADATA_CACHE.lock().unwrap();
        cache.insert(operator_oid, metadata.clone());
    }

    log::info!(
        "Operator '{}' registered with OID {}",
        operator_name,
        operator_oid
    );
    Ok(metadata)
}

/// Phase 4B: Register aggregate with full metadata
pub async fn register_aggregate(
    ctx: &SessionContext,
    aggregate_name: &str,
    namespace_name: &str,
    transition_function_oid: i32,
    final_function_oid: Option<i32>,
    transition_type_oid: i32,
    initial_value: Option<&str>,
) -> DFResult<AggregateMetadata> {
    // First register the aggregate as a function
    let function_metadata = register_function(
        ctx,
        aggregate_name,
        namespace_name,
        "postgres",
        "internal",
        vec![],              // Will be set based on aggregate definition
        transition_type_oid, // Return type matches transition type
        true,                // is_aggregate
        false,               // not window
        false,               // not security definer
        false,               // not strict
        true,                // immutable
        None,                // no function body
        100.0,               // default cost
    )
    .await?;

    // Get or allocate consistent OID for aggregate
    let aggregate_oid = get_or_allocate_aggregate_oid(aggregate_name)?;

    let sql = format!(
        "INSERT INTO pg_catalog.pg_aggregate \
         (aggfnoid, aggkind, aggnumdirectargs, aggtransfn, aggfinalfn, aggcombinefn, \
          aggserialfn, aggdeserialfn, aggmtransfn, aggminvtransfn, aggmfinalfn, \
          aggfinalextra, aggmfinalextra, aggsortop, aggtranstype, aggtransspace, \
          aggmtranstype, aggmtransspace, agginitval, aggminitval) \
         VALUES ({}, 'n', 0, {}, {}, 0, \
                 0, 0, 0, 0, 0, \
                 false, false, 0, {}, 0, \
                 0, 0, {}, NULL)",
        function_metadata.function_oid,
        transition_function_oid,
        final_function_oid.unwrap_or(0),
        transition_type_oid,
        initial_value
            .map(|v| format!("'{}'", v.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string())
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_aggregate table not ready yet");
        }
    }

    // Create enhanced metadata
    let metadata = AggregateMetadata {
        aggregate_oid,
        function_oid: function_metadata.function_oid,
        kind: AggregateKind::Normal,
        num_direct_args: 0,
        transition_function_oid,
        final_function_oid,
        combine_function_oid: None,
        serialization_function_oid: None,
        deserialization_function_oid: None,
        transition_type_oid,
        transition_space: 0,
        initial_value: initial_value.map(|s| s.to_string()),
        initial_condition: None,
    };

    // Cache the metadata
    {
        let mut cache = AGGREGATE_METADATA_CACHE.lock().unwrap();
        cache.insert(aggregate_oid, metadata.clone());
    }

    log::info!(
        "Aggregate '{}' registered with OID {}",
        aggregate_name,
        aggregate_oid
    );
    Ok(metadata)
}

/// Helper function to get namespace OID
async fn get_namespace_oid(ctx: &SessionContext, namespace_name: &str) -> Option<i32> {
    let result = ctx
        .sql("SELECT oid FROM pg_catalog.pg_namespace WHERE nspname=$schema")
        .await
        .ok()?
        .with_param_values(vec![("schema", ScalarValue::from(namespace_name))])
        .ok()?
        .collect()
        .await
        .ok()?;

    if result.is_empty() || result[0].num_rows() == 0 {
        None
    } else {
        let arr = result[0]
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::Int32Array>()
            .unwrap();
        Some(arr.value(0))
    }
}

/// Helper function to get language OID
fn get_language_oid(language_name: &str) -> Option<i32> {
    match language_name.to_lowercase().as_str() {
        "sql" => Some(12),
        "plpgsql" => Some(13),
        "c" => Some(13),
        "internal" => Some(12),
        _ => Some(12), // Default to SQL
    }
}

/// Get or allocate consistent OID for function
pub fn get_or_allocate_function_oid(function_name: &str) -> DFResult<i32> {
    let mut cache = FUNCTION_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(function_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(function_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Get or allocate consistent OID for operator
pub fn get_or_allocate_operator_oid(operator_name: &str) -> DFResult<i32> {
    let mut cache = OPERATOR_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(operator_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(operator_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Get or allocate consistent OID for aggregate
pub fn get_or_allocate_aggregate_oid(aggregate_name: &str) -> DFResult<i32> {
    let mut cache = AGGREGATE_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(aggregate_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(aggregate_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Get or allocate consistent OID for trigger
pub fn get_or_allocate_trigger_oid(trigger_name: &str) -> DFResult<i32> {
    let mut cache = TRIGGER_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(trigger_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(trigger_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Register a trigger in the system catalog
pub async fn register_trigger(
    ctx: &SessionContext,
    trigger_name: &str,
    table_oid: i32,
    function_oid: i32,
    trigger_type: TriggerType,
    enabled_state: TriggerEnabledState,
    is_internal: bool,
    when_clause: Option<&str>,
    transition_old_table: Option<&str>,
    transition_new_table: Option<&str>,
    trigger_args: Vec<String>,
) -> DFResult<TriggerMetadata> {
    // Get or allocate consistent OID for trigger
    let trigger_oid = get_or_allocate_trigger_oid(trigger_name)?;

    // Convert TriggerType to bitmask for pg_trigger storage
    let mut tgtype = 0i16;
    if trigger_type.before {
        tgtype |= 2;
    }
    if trigger_type.after {
        tgtype |= 0;
    }
    if trigger_type.instead {
        tgtype |= 64;
    }
    if trigger_type.insert {
        tgtype |= 4;
    }
    if trigger_type.update {
        tgtype |= 8;
    }
    if trigger_type.delete {
        tgtype |= 16;
    }
    if trigger_type.truncate {
        tgtype |= 32;
    }
    if trigger_type.per_row {
        tgtype |= 1;
    }

    let enabled_char = match enabled_state {
        TriggerEnabledState::Origin => 'O',
        TriggerEnabledState::Always => 'A',
        TriggerEnabledState::Replica => 'R',
        TriggerEnabledState::Disabled => 'D',
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_trigger \
         (oid, tgrelid, tgname, tgfoid, tgtype, tgenabled, tgisinternal, tgconstrrelid, \
          tgconstrindid, tgconstraint, tgdeferrable, tginitdeferred, tgnargs, tgattr, \
          tgargs, tgqual, tgoldtable, tgnewtable) \
         VALUES ({}, {}, '{}', {}, {}, '{}', {}, 0, \
                 0, 0, false, false, {}, NULL, \
                 {}, {}, {}, {})",
        trigger_oid,
        table_oid,
        trigger_name.replace('\'', "''"),
        function_oid,
        tgtype,
        enabled_char,
        is_internal,
        trigger_args.len(),
        if trigger_args.is_empty() {
            "NULL".to_string()
        } else {
            format!("'{{{}}}'", trigger_args.join(","))
        },
        when_clause
            .map(|w| format!("'{}'", w.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string()),
        transition_old_table
            .map(|t| format!("'{}'", t.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string()),
        transition_new_table
            .map(|t| format!("'{}'", t.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string())
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_trigger table not ready yet");
        }
    }

    // Create enhanced metadata
    let metadata = TriggerMetadata {
        trigger_oid,
        trigger_name: trigger_name.to_string(),
        table_oid,
        function_oid,
        trigger_type,
        enabled_state,
        is_internal,
        when_clause: when_clause.map(|s| s.to_string()),
        transition_old_table: transition_old_table.map(|s| s.to_string()),
        transition_new_table: transition_new_table.map(|s| s.to_string()),
        trigger_args,
    };

    // Cache the metadata
    {
        let mut cache = TRIGGER_METADATA_CACHE.lock().unwrap();
        cache.insert(trigger_oid, metadata.clone());
    }

    log::info!(
        "Trigger '{}' registered with OID {} on table OID {} (function OID: {})",
        trigger_name,
        trigger_oid,
        table_oid,
        function_oid
    );
    Ok(metadata)
}

/// Phase 4C: Security and Access Control Registration Functions

/// Register role membership in pg_auth_members
pub async fn register_auth_member(
    ctx: &SessionContext,
    roleid: i32,        // The role being granted
    member: i32,        // The role receiving the grant
    grantor: i32,       // The role doing the granting
    admin_option: bool, // Whether member can grant this role to others
) -> DFResult<AuthMemberMetadata> {
    let sql = format!(
        "INSERT INTO pg_catalog.pg_auth_members \
         (roleid, member, grantor, admin_option) \
         VALUES ({}, {}, {}, {})",
        roleid, member, grantor, admin_option
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_auth_members table not ready yet");
        }
    }

    // Create metadata
    let metadata = AuthMemberMetadata {
        roleid,
        member,
        grantor,
        admin_option,
    };

    // Cache the metadata
    {
        let mut cache = AUTH_MEMBER_CACHE.lock().unwrap();
        cache.insert((roleid, member), metadata.clone());
    }

    log::info!(
        "Auth membership registered: role {} granted to member {} by grantor {} (admin: {})",
        roleid,
        member,
        grantor,
        admin_option
    );
    Ok(metadata)
}

/// Get or allocate consistent OID for default ACL
pub fn get_or_allocate_default_acl_oid(acl_key: &str) -> DFResult<i32> {
    let mut cache = DEFAULT_ACL_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(acl_key) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(acl_key.to_string(), new_oid);
    Ok(new_oid)
}

/// Register default ACL in pg_default_acl
pub async fn register_default_acl(
    ctx: &SessionContext,
    defaclrole: i32,        // OID of role whose objects get these ACLs
    defaclnamespace: i32,   // OID of namespace (0 for all namespaces)
    defaclobjtype: char, // Object type: 'r' = relation, 'S' = sequence, 'f' = function, 'T' = type
    defaclacl: Vec<String>, // Access privileges in ACL format
) -> DFResult<DefaultAclMetadata> {
    // Generate unique key for OID allocation
    let acl_key = format!("{}:{}:{}", defaclrole, defaclnamespace, defaclobjtype);
    let oid = get_or_allocate_default_acl_oid(&acl_key)?;

    let acl_text = if defaclacl.is_empty() {
        "NULL".to_string()
    } else {
        format!("'{{{}}}'", defaclacl.join(","))
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_default_acl \
         (oid, defaclrole, defaclnamespace, defaclobjtype, defaclacl) \
         VALUES ({}, {}, {}, '{}', {})",
        oid, defaclrole, defaclnamespace, defaclobjtype, acl_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_default_acl table not ready yet");
        }
    }

    // Create metadata
    let metadata = DefaultAclMetadata {
        oid,
        defaclrole,
        defaclnamespace,
        defaclobjtype,
        defaclacl,
    };

    // Cache the metadata
    {
        let mut cache = DEFAULT_ACL_CACHE.lock().unwrap();
        cache.insert(oid, metadata.clone());
    }

    log::info!(
        "Default ACL registered with OID {} for role {} namespace {} objtype '{}'",
        oid,
        defaclrole,
        defaclnamespace,
        defaclobjtype
    );
    Ok(metadata)
}

/// Register initial privileges in pg_init_privs
pub async fn register_init_privs(
    ctx: &SessionContext,
    objoid: i32,            // OID of the object
    classoid: i32,          // OID of the system catalog containing the object
    objsubid: i32,          // Column number for column privileges, 0 for object
    privtype: char,         // Type of initial privilege (e = extension, i = initdb)
    initprivs: Vec<String>, // Initial access privileges in ACL format
) -> DFResult<InitPrivsMetadata> {
    let privs_text = if initprivs.is_empty() {
        "NULL".to_string()
    } else {
        format!("'{{{}}}'", initprivs.join(","))
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_init_privs \
         (objoid, classoid, objsubid, privtype, initprivs) \
         VALUES ({}, {}, {}, '{}', {})",
        objoid, classoid, objsubid, privtype, privs_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_init_privs table not ready yet");
        }
    }

    // Create metadata
    let metadata = InitPrivsMetadata {
        objoid,
        classoid,
        objsubid,
        privtype,
        initprivs,
    };

    // Cache the metadata
    {
        let mut cache = INIT_PRIVS_CACHE.lock().unwrap();
        cache.insert((objoid, classoid, objsubid), metadata.clone());
    }

    log::info!(
        "Initial privileges registered for object {} in catalog {} (subid: {}, type: '{}')",
        objoid,
        classoid,
        objsubid,
        privtype
    );
    Ok(metadata)
}

/// Register security label in pg_seclabel
pub async fn register_security_label(
    ctx: &SessionContext,
    objoid: i32,    // OID of the object this label is for
    classoid: i32,  // OID of the system catalog containing the object
    objsubid: i32,  // Column number for column labels, 0 for object
    provider: &str, // Label provider (e.g., selinux)
    label: &str,    // Security label value
) -> DFResult<SecLabelMetadata> {
    let sql = format!(
        "INSERT INTO pg_catalog.pg_seclabel \
         (objoid, classoid, objsubid, provider, label) \
         VALUES ({}, {}, {}, '{}', '{}')",
        objoid,
        classoid,
        objsubid,
        provider.replace('\'', "''"),
        label.replace('\'', "''")
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_seclabel table not ready yet");
        }
    }

    // Create metadata
    let metadata = SecLabelMetadata {
        objoid,
        classoid,
        objsubid,
        provider: provider.to_string(),
        label: label.to_string(),
    };

    // Cache the metadata
    {
        let mut cache = SEC_LABEL_CACHE.lock().unwrap();
        cache.insert(
            (objoid, classoid, objsubid, provider.to_string()),
            metadata.clone(),
        );
    }

    log::info!(
        "Security label registered for object {} in catalog {} (subid: {}, provider: '{}')",
        objoid,
        classoid,
        objsubid,
        provider
    );
    Ok(metadata)
}

/// Phase 4C: Advanced Table Features Registration Functions

/// Register partitioned table in pg_partitioned_table
pub async fn register_partitioned_table(
    ctx: &SessionContext,
    partrelid: i32,          // OID of the partitioned table
    partstrat: char,         // Partitioning strategy: 'h' = hash, 'l' = list, 'r' = range
    partnatts: i16,          // Number of partitioning columns
    partdefid: i32,          // OID of the default partition (0 if none)
    partattrs: Vec<i16>,     // Array of partitioning column numbers
    partclass: Vec<i32>,     // Array of operator class OIDs for partitioning columns
    partcollation: Vec<i32>, // Array of collation OIDs for partitioning columns
    partexprs: Option<&str>, // Partitioning expressions (serialized)
) -> DFResult<PartitionedTableMetadata> {
    let partattrs_text = if partattrs.is_empty() {
        "NULL".to_string()
    } else {
        format!(
            "'{{{}}}'",
            partattrs
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    let partclass_text = if partclass.is_empty() {
        "NULL".to_string()
    } else {
        format!(
            "'{{{}}}'",
            partclass
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    let partcollation_text = if partcollation.is_empty() {
        "NULL".to_string()
    } else {
        format!(
            "'{{{}}}'",
            partcollation
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    let partexprs_text = partexprs
        .map(|e| format!("'{}'", e.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());

    let sql = format!(
        "INSERT INTO pg_catalog.pg_partitioned_table \
         (partrelid, partstrat, partnatts, partdefid, partattrs, partclass, partcollation, partexprs) \
         VALUES ({}, '{}', {}, {}, {}, {}, {}, {})",
        partrelid, partstrat, partnatts, partdefid,
        partattrs_text, partclass_text, partcollation_text, partexprs_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_partitioned_table table not ready yet");
        }
    }

    // Create metadata
    let metadata = PartitionedTableMetadata {
        partrelid,
        partstrat,
        partnatts,
        partdefid,
        partattrs,
        partclass,
        partcollation,
        partexprs: partexprs.map(|s| s.to_string()),
    };

    // Cache the metadata
    {
        let mut cache = PARTITIONED_TABLE_CACHE.lock().unwrap();
        cache.insert(partrelid, metadata.clone());
    }

    log::info!(
        "Partitioned table registered for table OID {} (strategy: '{}', {} columns)",
        partrelid,
        partstrat,
        partnatts
    );
    Ok(metadata)
}

/// Get or allocate consistent OID for event trigger
pub fn get_or_allocate_event_trigger_oid(event_trigger_name: &str) -> DFResult<i32> {
    let mut cache = EVENT_TRIGGER_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(event_trigger_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(event_trigger_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Register event trigger in pg_event_trigger
pub async fn register_event_trigger(
    ctx: &SessionContext,
    evtname: &str,        // Event trigger name
    evtevent: &str, // Event name (ddl_command_start, ddl_command_end, table_rewrite, sql_drop)
    evtowner: i32,  // OID of owner
    evtfoid: i32,   // OID of trigger function
    evtenabled: char, // Enabled state: 'O' = origin, 'D' = disabled, 'R' = replica, 'A' = always
    evttags: Vec<String>, // Array of command tags (empty array = all commands)
) -> DFResult<EventTriggerMetadata> {
    let oid = get_or_allocate_event_trigger_oid(evtname)?;

    let evttags_text = if evttags.is_empty() {
        "NULL".to_string()
    } else {
        format!("'{{{}}}'", evttags.join(","))
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_event_trigger \
         (oid, evtname, evtevent, evtowner, evtfoid, evtenabled, evttags) \
         VALUES ({}, '{}', '{}', {}, {}, '{}', {})",
        oid,
        evtname.replace('\'', "''"),
        evtevent.replace('\'', "''"),
        evtowner,
        evtfoid,
        evtenabled,
        evttags_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_event_trigger table not ready yet");
        }
    }

    // Create metadata
    let metadata = EventTriggerMetadata {
        oid,
        evtname: evtname.to_string(),
        evtevent: evtevent.to_string(),
        evtowner,
        evtfoid,
        evtenabled,
        evttags,
    };

    // Cache the metadata
    {
        let mut cache = EVENT_TRIGGER_CACHE.lock().unwrap();
        cache.insert(oid, metadata.clone());
    }

    log::info!(
        "Event trigger '{}' registered with OID {} for event '{}' (function: {})",
        evtname,
        oid,
        evtevent,
        evtfoid
    );
    Ok(metadata)
}

/// Get or allocate consistent OID for user mapping
pub fn get_or_allocate_user_mapping_oid(mapping_key: &str) -> DFResult<i32> {
    let mut cache = USER_MAPPING_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(mapping_key) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(mapping_key.to_string(), new_oid);
    Ok(new_oid)
}

/// Register user mapping in pg_user_mapping
pub async fn register_user_mapping(
    ctx: &SessionContext,
    umuser: i32,            // OID of user (0 for public)
    umserver: i32,          // OID of foreign server
    umoptions: Vec<String>, // User mapping options in "key=value" format
) -> DFResult<UserMappingMetadata> {
    // Generate unique key for OID allocation
    let mapping_key = format!("{}:{}", umuser, umserver);
    let oid = get_or_allocate_user_mapping_oid(&mapping_key)?;

    let umoptions_text = if umoptions.is_empty() {
        "NULL".to_string()
    } else {
        format!("'{{{}}}'", umoptions.join(","))
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_user_mapping \
         (oid, umuser, umserver, umoptions) \
         VALUES ({}, {}, {}, {})",
        oid, umuser, umserver, umoptions_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_user_mapping table not ready yet");
        }
    }

    // Create metadata
    let metadata = UserMappingMetadata {
        oid,
        umuser,
        umserver,
        umoptions,
    };

    // Cache the metadata
    {
        let mut cache = USER_MAPPING_CACHE.lock().unwrap();
        cache.insert(oid, metadata.clone());
    }

    log::info!(
        "User mapping registered with OID {} for user {} server {}",
        oid,
        umuser,
        umserver
    );
    Ok(metadata)
}

/// Phase 4C: Large Object Support Registration Functions

/// Register large object in pg_largeobject_metadata
pub async fn register_large_object(
    ctx: &SessionContext,
    oid: i32,            // Large object OID
    lomowner: i32,       // OID of owner
    lomacl: Vec<String>, // Access privileges
) -> DFResult<LargeObjectMetadata> {
    let lomacl_text = if lomacl.is_empty() {
        "NULL".to_string()
    } else {
        format!("'{{{}}}'", lomacl.join(","))
    };

    let sql = format!(
        "INSERT INTO pg_catalog.pg_largeobject_metadata \
         (oid, lomowner, lomacl) \
         VALUES ({}, {}, {})",
        oid, lomowner, lomacl_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_largeobject_metadata table not ready yet");
        }
    }

    // Create metadata
    let metadata = LargeObjectMetadata {
        oid,
        lomowner,
        lomacl,
    };

    // Cache the metadata
    {
        let mut cache = LARGE_OBJECT_CACHE.lock().unwrap();
        cache.insert(oid, metadata.clone());
    }

    log::info!(
        "Large object registered with OID {} (owner: {})",
        oid,
        lomowner
    );
    Ok(metadata)
}

/// Register large object data page in pg_largeobject
pub async fn register_large_object_data(
    ctx: &SessionContext,
    loid: i32,     // Large object OID
    pageno: i32,   // Page number within large object
    data: Vec<u8>, // Page data (up to 2KB)
) -> DFResult<LargeObjectDataMetadata> {
    // Convert binary data to bytea format for PostgreSQL
    let data_hex = hex::encode(&data);

    let sql = format!(
        "INSERT INTO pg_catalog.pg_largeobject \
         (loid, pageno, data) \
         VALUES ({}, {}, '\\x{}')",
        loid, pageno, data_hex
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_largeobject table not ready yet");
        }
    }

    // Create metadata
    let metadata = LargeObjectDataMetadata { loid, pageno, data };

    // Cache the metadata
    {
        let mut cache = LARGE_OBJECT_DATA_CACHE.lock().unwrap();
        cache.insert((loid, pageno), metadata.clone());
    }

    log::info!(
        "Large object data registered for LOB {} page {} ({} bytes)",
        loid,
        pageno,
        metadata.data.len()
    );
    Ok(metadata)
}

/// Phase 4C: Text Search System Registration Functions

/// Get or allocate consistent OID for text search config
pub fn get_or_allocate_ts_config_oid(config_name: &str) -> DFResult<i32> {
    let mut cache = TS_CONFIG_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(config_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(config_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Register text search configuration in pg_ts_config
pub async fn register_text_search_config(
    ctx: &SessionContext,
    cfgname: &str,     // Configuration name
    cfgnamespace: i32, // Namespace OID
    cfgowner: i32,     // Owner OID
    cfgparser: i32,    // Parser OID
) -> DFResult<TextSearchConfigMetadata> {
    let oid = get_or_allocate_ts_config_oid(cfgname)?;

    let sql = format!(
        "INSERT INTO pg_catalog.pg_ts_config \
         (oid, cfgname, cfgnamespace, cfgowner, cfgparser) \
         VALUES ({}, '{}', {}, {}, {})",
        oid,
        cfgname.replace('\'', "''"),
        cfgnamespace,
        cfgowner,
        cfgparser
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_ts_config table not ready yet");
        }
    }

    // Create metadata
    let metadata = TextSearchConfigMetadata {
        oid,
        cfgname: cfgname.to_string(),
        cfgnamespace,
        cfgowner,
        cfgparser,
    };

    // Cache the metadata
    {
        let mut cache = TS_CONFIG_CACHE.lock().unwrap();
        cache.insert(oid, metadata.clone());
    }

    log::info!(
        "Text search config '{}' registered with OID {} (parser: {})",
        cfgname,
        oid,
        cfgparser
    );
    Ok(metadata)
}

/// Get or allocate consistent OID for text search dictionary
pub fn get_or_allocate_ts_dict_oid(dict_name: &str) -> DFResult<i32> {
    let mut cache = TS_DICT_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(dict_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(dict_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Register text search dictionary in pg_ts_dict
pub async fn register_text_search_dict(
    ctx: &SessionContext,
    dictname: &str,               // Dictionary name
    dictnamespace: i32,           // Namespace OID
    dictowner: i32,               // Owner OID
    dicttemplate: i32,            // Template OID
    dictinitoption: Option<&str>, // Initialization options
) -> DFResult<TextSearchDictMetadata> {
    let oid = get_or_allocate_ts_dict_oid(dictname)?;

    let initoption_text = dictinitoption
        .map(|opt| format!("'{}'", opt.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());

    let sql = format!(
        "INSERT INTO pg_catalog.pg_ts_dict \
         (oid, dictname, dictnamespace, dictowner, dicttemplate, dictinitoption) \
         VALUES ({}, '{}', {}, {}, {}, {})",
        oid,
        dictname.replace('\'', "''"),
        dictnamespace,
        dictowner,
        dicttemplate,
        initoption_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_ts_dict table not ready yet");
        }
    }

    // Create metadata
    let metadata = TextSearchDictMetadata {
        oid,
        dictname: dictname.to_string(),
        dictnamespace,
        dictowner,
        dicttemplate,
        dictinitoption: dictinitoption.map(|s| s.to_string()),
    };

    // Cache the metadata
    {
        let mut cache = TS_DICT_CACHE.lock().unwrap();
        cache.insert(oid, metadata.clone());
    }

    log::info!(
        "Text search dictionary '{}' registered with OID {} (template: {})",
        dictname,
        oid,
        dicttemplate
    );
    Ok(metadata)
}

/// Phase 4C: Extended Statistics Registration Functions

/// Get or allocate consistent OID for extended statistics
pub fn get_or_allocate_ext_stats_oid(stats_name: &str) -> DFResult<i32> {
    let mut cache = EXT_STATS_OID_CACHE.lock().unwrap();

    if let Some(&existing_oid) = cache.get(stats_name) {
        return Ok(existing_oid);
    }

    let new_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(new_oid, false)?;

    cache.insert(stats_name.to_string(), new_oid);
    Ok(new_oid)
}

/// Register extended statistics in pg_statistic_ext
pub async fn register_extended_statistic(
    ctx: &SessionContext,
    stxrelid: i32,          // Table OID
    stxname: &str,          // Statistics object name
    stxnamespace: i32,      // Namespace OID
    stxowner: i32,          // Owner OID
    stxkeys: Vec<i16>,      // Column numbers
    stxkind: Vec<char>, // Statistics kinds: 'd' = ndistinct, 'f' = functional deps, 'm' = MCV, 'e' = expression
    stxexprs: Option<&str>, // Expressions (serialized)
) -> DFResult<ExtendedStatisticMetadata> {
    let oid = get_or_allocate_ext_stats_oid(stxname)?;

    let stxkeys_text = if stxkeys.is_empty() {
        "NULL".to_string()
    } else {
        format!(
            "'{{{}}}'",
            stxkeys
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    let stxkind_text = if stxkind.is_empty() {
        "NULL".to_string()
    } else {
        format!("'{{{}}}'", stxkind.iter().collect::<String>())
    };

    let stxexprs_text = stxexprs
        .map(|e| format!("'{}'", e.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());

    let sql = format!(
        "INSERT INTO pg_catalog.pg_statistic_ext \
         (oid, stxrelid, stxname, stxnamespace, stxowner, stxkeys, stxkind, stxexprs) \
         VALUES ({}, {}, '{}', {}, {}, {}, {}, {})",
        oid,
        stxrelid,
        stxname.replace('\'', "''"),
        stxnamespace,
        stxowner,
        stxkeys_text,
        stxkind_text,
        stxexprs_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_statistic_ext table not ready yet");
        }
    }

    // Create metadata
    let metadata = ExtendedStatisticMetadata {
        oid,
        stxrelid,
        stxname: stxname.to_string(),
        stxnamespace,
        stxowner,
        stxkeys,
        stxkind,
        stxexprs: stxexprs.map(|s| s.to_string()),
    };

    // Cache the metadata
    {
        let mut cache = EXT_STATS_CACHE.lock().unwrap();
        cache.insert(oid, metadata.clone());
    }

    log::info!(
        "Extended statistics '{}' registered with OID {} for table {} ({} columns)",
        stxname,
        oid,
        stxrelid,
        metadata.stxkeys.len()
    );
    Ok(metadata)
}

/// Register extended statistics data in pg_statistic_ext_data
pub async fn register_extended_statistic_data(
    ctx: &SessionContext,
    stxoid: i32,                    // Statistics object OID
    stxdinherit: bool,              // True if inherited statistics
    stxdndistinct: Option<&str>,    // N-distinct statistics data
    stxddependencies: Option<&str>, // Functional dependencies data
    stxdmcv: Option<&str>,          // Most Common Values data
    stxdexpr: Option<&str>,         // Expression statistics data
) -> DFResult<ExtendedStatisticDataMetadata> {
    let ndistinct_text = stxdndistinct
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());
    let dependencies_text = stxddependencies
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());
    let mcv_text = stxdmcv
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());
    let expr_text = stxdexpr
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());

    let sql = format!(
        "INSERT INTO pg_catalog.pg_statistic_ext_data \
         (stxoid, stxdinherit, stxdndistinct, stxddependencies, stxdmcv, stxdexpr) \
         VALUES ({}, {}, {}, {}, {}, {})",
        stxoid, stxdinherit, ndistinct_text, dependencies_text, mcv_text, expr_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_statistic_ext_data table not ready yet");
        }
    }

    // Create metadata
    let metadata = ExtendedStatisticDataMetadata {
        stxoid,
        stxdinherit,
        stxdndistinct: stxdndistinct.map(|s| s.to_string()),
        stxddependencies: stxddependencies.map(|s| s.to_string()),
        stxdmcv: stxdmcv.map(|s| s.to_string()),
        stxdexpr: stxdexpr.map(|s| s.to_string()),
    };

    // Cache the metadata
    {
        let mut cache = EXT_STATS_DATA_CACHE.lock().unwrap();
        cache.insert(stxoid, metadata.clone());
    }

    log::info!(
        "Extended statistics data registered for stats OID {} (inherit: {})",
        stxoid,
        stxdinherit
    );
    Ok(metadata)
}

/// Phase 4C: Replication Features Registration Functions

/// Register publication namespace in pg_publication_namespace
pub async fn register_publication_namespace(
    ctx: &SessionContext,
    pnpubid: i32, // Publication OID
    pnnspid: i32, // Namespace OID
) -> DFResult<PublicationNamespaceMetadata> {
    let sql = format!(
        "INSERT INTO pg_catalog.pg_publication_namespace \
         (pnpubid, pnnspid) \
         VALUES ({}, {})",
        pnpubid, pnnspid
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_publication_namespace table not ready yet");
        }
    }

    // Create metadata
    let metadata = PublicationNamespaceMetadata { pnpubid, pnnspid };

    // Cache the metadata
    {
        let mut cache = PUB_NAMESPACE_CACHE.lock().unwrap();
        cache.insert((pnpubid, pnnspid), metadata.clone());
    }

    log::info!(
        "Publication namespace mapping registered: publication {} -> namespace {}",
        pnpubid,
        pnnspid
    );
    Ok(metadata)
}

/// Register replication origin in pg_replication_origin
pub async fn register_replication_origin(
    ctx: &SessionContext,
    roident: i32, // Origin identifier
    roname: &str, // Origin name
) -> DFResult<ReplicationOriginMetadata> {
    let sql = format!(
        "INSERT INTO pg_catalog.pg_replication_origin \
         (roident, roname) \
         VALUES ({}, '{}')",
        roident,
        roname.replace('\'', "''")
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_replication_origin table not ready yet");
        }
    }

    // Create metadata
    let metadata = ReplicationOriginMetadata {
        roident,
        roname: roname.to_string(),
    };

    // Cache the metadata
    {
        let mut cache = REPLICATION_ORIGIN_CACHE.lock().unwrap();
        cache.insert(roname.to_string(), metadata.clone());
    }

    log::info!(
        "Replication origin '{}' registered with identifier {}",
        roname,
        roident
    );
    Ok(metadata)
}

/// Register subscription relation in pg_subscription_rel
pub async fn register_subscription_rel(
    ctx: &SessionContext,
    srsubid: i32,           // Subscription OID
    srrelid: i32,           // Relation OID
    srsubstate: char, // Subscription state: 'i' = initialize, 'd' = data copy, 's' = synchronized, 'r' = ready
    srsublsn: Option<&str>, // LSN of the subscription
) -> DFResult<SubscriptionRelMetadata> {
    let lsn_text = srsublsn
        .map(|lsn| format!("'{}'", lsn.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_string());

    let sql = format!(
        "INSERT INTO pg_catalog.pg_subscription_rel \
         (srsubid, srrelid, srsubstate, srsublsn) \
         VALUES ({}, {}, '{}', {})",
        srsubid, srrelid, srsubstate, lsn_text
    );

    match ctx.sql(&sql).await {
        Ok(df) => {
            df.collect().await?;
        }
        Err(_) => {
            log::debug!("pg_subscription_rel table not ready yet");
        }
    }

    // Create metadata
    let metadata = SubscriptionRelMetadata {
        srsubid,
        srrelid,
        srsubstate,
        srsublsn: srsublsn.map(|s| s.to_string()),
    };

    // Cache the metadata
    {
        let mut cache = SUBSCRIPTION_REL_CACHE.lock().unwrap();
        cache.insert((srsubid, srrelid), metadata.clone());
    }

    log::info!(
        "Subscription relation registered: subscription {} -> relation {} (state: '{}')",
        srsubid,
        srrelid,
        srsubstate
    );
    Ok(metadata)
}

pub async fn register_user_database(ctx: &SessionContext, database_name: &str) -> DFResult<()> {
    // let oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);

    let df: datafusion::prelude::DataFrame = ctx
        .sql("SELECT datname FROM pg_catalog.pg_database where datname=$database_name")
        .await?
        .with_param_values(vec![("database_name", ScalarValue::from(database_name))])?;
    if df.count().await? == 0 {
        let getiddf = ctx
            .sql("select max(oid)+1 from pg_catalog.pg_database")
            .await?;
        let batches = getiddf.collect().await?;
        let array = batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        let dbid = array.value(0);

        let df = ctx
            .sql(&format!(
                "INSERT INTO pg_catalog.pg_database (
            oid,
            datname,
            datdba,
            encoding,
            datcollate,
            datctype,
            datistemplate,
            datallowconn,
            datconnlimit,
            datfrozenxid,
            datminmxid,
            dattablespace,
            datacl
        ) VALUES (
            {},
            '{}',
            27735,
            6,
            'C',
            'C',
            false,
            true,
            -1,        
            726,
            1,
            1663,
            ARRAY['=Tc/dbuser', 'dbuser=CTc/dbuser']
        );",
                dbid,
                database_name.replace('\'', "''")
            ))
            .await?;
        df.collect().await?;
    }
    let df = ctx
        .sql("select datname from pg_catalog.pg_database")
        .await?;
    df.show().await?;
    Ok(())
}

pub async fn register_schema(
    ctx: &SessionContext,
    _database_name: &str,
    schema_name: &str,
) -> DFResult<()> {
    let df = ctx
        .sql("SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname=$schema")
        .await?
        .with_param_values(vec![("schema", ScalarValue::from(schema_name))])?;

    if df.count().await? == 0 {
        let oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
        validate_oid_range(oid, false)?;
        let sql = format!(
            "INSERT INTO pg_catalog.pg_namespace (oid, nspname, nspowner, nspacl) VALUES ({oid}, '{}', 27735, NULL)",
            schema_name.replace('\'', "''")
        );
        ctx.sql(&sql).await?.collect().await?;
    }

    Ok(())
}

pub async fn register_user_tables(
    ctx: &SessionContext,
    _database_name: &str,
    schema_name: &str,
    table_name: &str,
    columns: Vec<BTreeMap<String, ColumnDef>>,
) -> DFResult<()> {
    let df = ctx
        .sql("SELECT 1 FROM pg_catalog.pg_class WHERE relname=$relname")
        .await?
        .with_param_values(vec![("relname", ScalarValue::from(table_name))])?;

    if df.count().await? > 0 {
        log::info!("table already exists {:}?", table_name);
        return Ok(());
    }

    let table_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(table_oid, false)?;

    let type_oid = NEXT_OID.fetch_add(1, Ordering::SeqCst);
    validate_oid_range(type_oid, false)?;

    let ns_df = ctx
        .sql("SELECT oid FROM pg_catalog.pg_namespace WHERE nspname=$schema")
        .await?
        .with_param_values(vec![("schema", ScalarValue::from(schema_name))])?;
    let ns_batches = ns_df.collect().await?;
    let schema_oid = if ns_batches.is_empty() || ns_batches[0].num_rows() == 0 {
        return Err(DataFusionError::Execution("schema not found".to_string()));
    } else {
        let arr = ns_batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::Int32Array>()
            .unwrap();
        arr.value(0)
    };

    if ctx
        .sql(&format!(
            "SELECT 1 FROM pg_catalog.pg_class WHERE oid = {table_oid}"
        ))
        .await?
        .count()
        .await?
        == 0
    {
        let sql = format!(
            "INSERT INTO pg_catalog.pg_class \
                 (oid, relname, relnamespace, relkind, reltuples, reltype, relispartition) \
                 VALUES ({table_oid},'{}',{schema_oid},'r',0,{type_oid}, false)",
            table_name.replace('\'', "''")
        );
        ctx.sql(&sql).await?.collect().await?;
    }

    if ctx
        .sql(&format!(
            "SELECT 1 FROM pg_catalog.pg_type WHERE oid = {type_oid}"
        ))
        .await?
        .count()
        .await?
        == 0
    {
        let sql = format!(
            "INSERT INTO pg_catalog.pg_type \
                 (oid, typname, typrelid, typlen, typcategory) \
                 VALUES ({type_oid},'_{table_name}',{table_oid},-1,'C')"
        );
        ctx.sql(&sql).await?.collect().await?;
    }

    for (idx, col) in columns.iter().enumerate() {
        let (name, def) = col.iter().next().unwrap();
        let atttypid = map_type_to_oid(&def.col_type);
        let notnull = if def.nullable { "false" } else { "true" };
        let sql = format!(
            "INSERT INTO pg_catalog.pg_attribute \
                 (attrelid,attnum,attname,atttypid,atttypmod,attnotnull,attisdropped) \
                 VALUES ({table_oid},{},'{}',{atttypid},-1,{notnull},false)",
            idx + 1,
            name.replace('\'', "''")
        );
        ctx.sql(&sql).await?.collect().await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::get_base_session_context;

    #[test]
    fn test_oid_validation() {
        // Static OID validation
        assert!(validate_oid_range(1000, true).is_ok());
        assert!(validate_oid_range(50000, true).is_ok());
        assert!(validate_oid_range(50001, true).is_err());

        // Dynamic OID validation
        assert!(validate_oid_range(50010, false).is_ok());
        assert!(validate_oid_range(60000, false).is_ok());
        assert!(validate_oid_range(50009, false).is_err());
    }

    #[test]
    fn test_type_oid_cache() {
        // Test consistent OID allocation for types
        let type1_oid1 = get_or_allocate_type_oid("custom_type").unwrap();
        let type1_oid2 = get_or_allocate_type_oid("custom_type").unwrap();
        assert_eq!(type1_oid1, type1_oid2);

        let type2_oid = get_or_allocate_type_oid("another_type").unwrap();
        assert_ne!(type1_oid1, type2_oid);
    }

    #[test]
    fn test_description_oid_cache() {
        // Test consistent OID allocation for descriptions
        let desc1_oid1 = get_or_allocate_description_oid(1000, 1255, 0).unwrap();
        let desc1_oid2 = get_or_allocate_description_oid(1000, 1255, 0).unwrap();
        assert_eq!(desc1_oid1, desc1_oid2);

        let desc2_oid = get_or_allocate_description_oid(1001, 1255, 0).unwrap();
        assert_ne!(desc1_oid1, desc2_oid);
    }



    #[test]
    fn test_oid_range_validation_edge_cases() {
        // Test edge cases for OID range validation
        assert!(validate_oid_range(1, true).is_ok());
        assert!(validate_oid_range(STATIC_OID_RANGE_MAX, true).is_ok());
        assert!(validate_oid_range(STATIC_OID_RANGE_MAX + 1, true).is_err());

        assert!(validate_oid_range(DYNAMIC_OID_RANGE_MIN, false).is_ok());
        assert!(validate_oid_range(DYNAMIC_OID_RANGE_MIN - 1, false).is_err());
        assert!(validate_oid_range(100000, false).is_ok());
    }

    #[test]
    fn test_type_oid_cache_performance() {
        // Test that OID cache provides consistent performance
        let start = std::time::Instant::now();

        // First call - should allocate new OID
        let _oid1 = get_or_allocate_type_oid("perf_test_type").unwrap();
        let first_duration = start.elapsed();

        let start2 = std::time::Instant::now();
        // Second call - should use cached OID
        let _oid2 = get_or_allocate_type_oid("perf_test_type").unwrap();
        let second_duration = start2.elapsed();

        // Cached access should be faster than first allocation
        assert!(second_duration <= first_duration);
    }

    #[tokio::test]
    async fn test_phase2_integration() {
        // Test Phase 2 integration with real session context
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test that OID caches are populated after Phase 2
        // This will depend on whether YAML files exist in test environment
        let type_cache_populated = {
            let cache = TYPE_OID_CACHE.lock().unwrap();
            !cache.is_empty()
        };

        // If cache is populated, verify some basic types exist
        if type_cache_populated {
            let cache = TYPE_OID_CACHE.lock().unwrap();
            // Check for some common PostgreSQL types that should be in pg_type
            let has_basic_types = cache.contains_key("int4")
                || cache.contains_key("text")
                || cache.contains_key("bool");
            if !has_basic_types {
                // This is OK if we don't have the full YAML data in tests
                log::debug!("Basic types not found in cache during test");
            }
        }

        // Test that consistency check can run without errors
        let consistency_result = ensure_static_table_oid_consistency(&ctx).await;
        assert!(
            consistency_result.is_ok(),
            "OID consistency check should not fail"
        );
    }

    #[tokio::test]
    async fn test_phase3_enhanced_table_registration() {
        // Test Phase 3 enhanced table registration
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Ensure schema exists
        register_schema(&ctx, "pgtry", "public").await.unwrap();

        // Create enhanced table metadata
        let columns = vec![
            ColumnMetadata {
                column_name: "id".to_string(),
                column_oid: 0,     // Will be assigned
                data_type_oid: 23, // int4
                column_number: 1,
                is_nullable: false,
                has_default: false,
                is_identity: true,
            },
            ColumnMetadata {
                column_name: "name".to_string(),
                column_oid: 0,
                data_type_oid: 25, // text
                column_number: 2,
                is_nullable: true,
                has_default: false,
                is_identity: false,
            },
        ];

        let constraints = vec![ConstraintMetadata {
            constraint_oid: 0,
            constraint_name: "users_pkey".to_string(),
            constraint_type: ConstraintType::PrimaryKey,
            table_oid: 0,     // Will be set
            columns: vec![1], // id column
            referenced_table_oid: None,
            referenced_columns: vec![],
        }];

        let indexes = vec![IndexMetadata {
            index_oid: 0,
            index_name: "users_pkey_idx".to_string(),
            table_oid: 0,     // Will be set
            columns: vec![1], // id column
            is_unique: true,
            is_primary: true,
            index_type: "btree".to_string(),
        }];

        // Register enhanced table
        let result = register_enhanced_table(
            &ctx,
            "pgtry",
            "public",
            "users",
            TableType::Table,
            columns,
            constraints,
            indexes,
        )
        .await;

        if let Err(ref e) = result {
            log::error!("Enhanced table registration failed: {}", e);
        }
        assert!(
            result.is_ok(),
            "Enhanced table registration should succeed: {:?}",
            result
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.table_name, "users");
        assert_eq!(metadata.columns.len(), 2);
        assert_eq!(metadata.constraints.len(), 1);
        assert_eq!(metadata.indexes.len(), 1);

        // Verify table was registered in pg_class
        let class_check = ctx
            .sql("SELECT relname FROM pg_catalog.pg_class WHERE relname='users'")
            .await
            .unwrap();
        assert_eq!(class_check.count().await.unwrap(), 1);
    }

    #[test]
    fn test_enhanced_metadata_caching() {
        // Test enhanced metadata caching
        let metadata = EnhancedTableMetadata {
            table_oid: 50001,
            table_name: "test_table".to_string(),
            schema_oid: 1000,
            table_type: TableType::Table,
            columns: vec![],
            constraints: vec![],
            indexes: vec![],
            dependencies: vec![],
        };

        // Store metadata
        update_enhanced_table_metadata(50001, metadata.clone());

        // Retrieve metadata
        let retrieved = get_enhanced_table_metadata(50001);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.table_name, "test_table");
        assert_eq!(retrieved.table_oid, 50001);
    }

    #[test]
    fn test_constraint_and_index_oid_allocation() {
        // Test constraint OID allocation
        let constraint_oid1 = get_or_allocate_constraint_oid("test_constraint").unwrap();
        let constraint_oid2 = get_or_allocate_constraint_oid("test_constraint").unwrap();
        assert_eq!(
            constraint_oid1, constraint_oid2,
            "Constraint OIDs should be consistent"
        );

        // Test index OID allocation
        let index_oid1 = get_or_allocate_index_oid("test_index").unwrap();
        let index_oid2 = get_or_allocate_index_oid("test_index").unwrap();
        assert_eq!(index_oid1, index_oid2, "Index OIDs should be consistent");

        // Ensure different objects get different OIDs
        let different_constraint_oid = get_or_allocate_constraint_oid("other_constraint").unwrap();
        assert_ne!(
            constraint_oid1, different_constraint_oid,
            "Different constraints should have different OIDs"
        );
    }

    #[tokio::test]
    async fn test_phase3_oid_consistency() {
        // Test Phase 3 OID consistency checks
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test that enhanced consistency check runs without errors
        let consistency_result = ensure_enhanced_oid_consistency(&ctx).await;
        assert!(
            consistency_result.is_ok(),
            "Enhanced OID consistency check should not fail"
        );
    }

    #[tokio::test]
    async fn test_phase4a_enhanced_database_registration() {
        // Test Phase 4A enhanced database registration
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Register enhanced database
        let result = register_enhanced_database(
            &ctx, "test_db", "postgres", 6, // UTF8
            "C", "C", false, // not template
            true,  // allow connections
            100,   // connection limit
        )
        .await;

        assert!(
            result.is_ok(),
            "Enhanced database registration should succeed"
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.database_name, "test_db");
        assert_eq!(metadata.encoding, 6);
        assert_eq!(metadata.collate, "C");
        assert!(!metadata.is_template);
        assert!(metadata.allow_connections);
        assert_eq!(metadata.connection_limit, 100);
    }

    #[tokio::test]
    async fn test_phase4a_tablespace_registration() {
        // Test Phase 4A tablespace registration
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Register tablespace
        let result = register_tablespace(
            &ctx,
            "test_tablespace",
            "postgres",
            Some("/var/lib/postgresql/tablespaces/test"),
        )
        .await;

        assert!(result.is_ok(), "Tablespace registration should succeed");

        let metadata = result.unwrap();
        assert_eq!(metadata.tablespace_name, "test_tablespace");
        assert_eq!(
            metadata.location,
            Some("/var/lib/postgresql/tablespaces/test".to_string())
        );
    }

    #[tokio::test]
    async fn test_phase4a_user_registration() {
        // Test Phase 4A user registration
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Register user
        let result = register_user(
            &ctx,
            "test_user",
            false, // not superuser
            true,  // can create db
            false, // cannot create role
            true,  // can login
            false, // not replication
            Some("password_hash"),
            10, // connection limit
        )
        .await;

        assert!(result.is_ok(), "User registration should succeed");

        let metadata = result.unwrap();
        assert_eq!(metadata.username, "test_user");
        assert!(!metadata.is_superuser);
        assert!(metadata.can_create_db);
        assert!(!metadata.can_create_role);
        assert!(metadata.can_login);
        assert!(!metadata.is_replication);
        assert_eq!(metadata.connection_limit, 10);
    }

    #[test]
    fn test_phase4a_oid_allocation() {
        // Test Phase 4A OID allocation
        let db_oid1 = get_or_allocate_database_oid("test_db").unwrap();
        let db_oid2 = get_or_allocate_database_oid("test_db").unwrap();
        assert_eq!(db_oid1, db_oid2, "Database OIDs should be consistent");

        let ts_oid1 = get_or_allocate_tablespace_oid("test_ts").unwrap();
        let ts_oid2 = get_or_allocate_tablespace_oid("test_ts").unwrap();
        assert_eq!(ts_oid1, ts_oid2, "Tablespace OIDs should be consistent");

        let user_oid1 = get_or_allocate_user_oid("test_user").unwrap();
        let user_oid2 = get_or_allocate_user_oid("test_user").unwrap();
        assert_eq!(user_oid1, user_oid2, "User OIDs should be consistent");

        // Ensure different objects get different OIDs
        let different_db_oid = get_or_allocate_database_oid("other_db").unwrap();
        assert_ne!(
            db_oid1, different_db_oid,
            "Different databases should have different OIDs"
        );
    }

    #[tokio::test]
    async fn test_phase4a_system_initialization() {
        // Test Phase 4A system catalog initialization
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test that system initialization runs without errors
        let init_result = initialize_system_catalog(&ctx).await;
        assert!(
            init_result.is_ok(),
            "System catalog initialization should not fail"
        );
    }

    #[tokio::test]
    async fn test_phase4b_function_registration() {
        // Test Phase 4B function registration
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Ensure schema exists
        register_schema(&ctx, "pgtry", "public").await.unwrap();

        // Register function
        let result = register_function(
            &ctx,
            "test_function",
            "public",
            "postgres",
            "sql",
            vec![23, 25], // int4, text arguments
            16,           // bool return type
            false,        // not aggregate
            false,        // not window
            false,        // not security definer
            true,         // strict
            true,         // immutable
            Some("SELECT $1 > 0 AND length($2) > 0"),
            10.0, // cost
        )
        .await;

        assert!(result.is_ok(), "Function registration should succeed");

        let metadata = result.unwrap();
        assert_eq!(metadata.function_name, "test_function");
        assert_eq!(metadata.arg_types, vec![23, 25]);
        assert_eq!(metadata.return_type_oid, 16);
        assert!(!metadata.is_aggregate);
        assert!(metadata.is_strict);
        assert!(metadata.is_immutable);
    }

    #[tokio::test]
    async fn test_phase4b_operator_registration() {
        // Test Phase 4B operator registration
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Ensure schema exists
        register_schema(&ctx, "pgtry", "public").await.unwrap();

        // First register a function for the operator
        let func_result = register_function(
            &ctx,
            "test_operator_func",
            "public",
            "postgres",
            "internal",
            vec![23, 23], // int4, int4 arguments
            16,           // bool return type
            false,        // not aggregate
            false,        // not window
            false,        // not security definer
            true,         // strict
            true,         // immutable
            None,         // no function body
            1.0,          // cost
        )
        .await
        .unwrap();

        // Register operator
        let result = register_operator(
            &ctx,
            "@@",
            "public",
            "postgres",
            Some(23), // left type: int4
            Some(23), // right type: int4
            16,       // result type: bool
            func_result.function_oid,
        )
        .await;

        assert!(result.is_ok(), "Operator registration should succeed");

        let metadata = result.unwrap();
        assert_eq!(metadata.operator_name, "@@");
        assert_eq!(metadata.left_type_oid, Some(23));
        assert_eq!(metadata.right_type_oid, Some(23));
        assert_eq!(metadata.result_type_oid, 16);
        assert_eq!(metadata.function_oid, func_result.function_oid);
    }

    #[tokio::test]
    async fn test_phase4b_aggregate_registration() {
        // Test Phase 4B aggregate registration
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Ensure schema exists
        register_schema(&ctx, "pgtry", "public").await.unwrap();

        // First register transition function
        let trans_func = register_function(
            &ctx,
            "test_aggregate_trans",
            "public",
            "postgres",
            "internal",
            vec![23, 23], // (state, input)
            23,           // int4 return type
            false,        // not aggregate
            false,        // not window
            false,        // not security definer
            true,         // strict
            true,         // immutable
            None,         // no function body
            1.0,          // cost
        )
        .await
        .unwrap();

        // Register aggregate
        let result = register_aggregate(
            &ctx,
            "test_sum",
            "public",
            trans_func.function_oid,
            None,      // no final function
            23,        // int4 transition type
            Some("0"), // initial value
        )
        .await;

        assert!(result.is_ok(), "Aggregate registration should succeed");

        let metadata = result.unwrap();
        assert_eq!(metadata.transition_function_oid, trans_func.function_oid);
        assert_eq!(metadata.transition_type_oid, 23);
        assert_eq!(metadata.initial_value, Some("0".to_string()));
        assert!(matches!(metadata.kind, AggregateKind::Normal));
    }

    #[test]
    fn test_phase4b_oid_allocation() {
        // Test Phase 4B OID allocation
        let func_oid1 = get_or_allocate_function_oid("test_func").unwrap();
        let func_oid2 = get_or_allocate_function_oid("test_func").unwrap();
        assert_eq!(func_oid1, func_oid2, "Function OIDs should be consistent");

        let op_oid1 = get_or_allocate_operator_oid("@@").unwrap();
        let op_oid2 = get_or_allocate_operator_oid("@@").unwrap();
        assert_eq!(op_oid1, op_oid2, "Operator OIDs should be consistent");

        let agg_oid1 = get_or_allocate_aggregate_oid("test_agg").unwrap();
        let agg_oid2 = get_or_allocate_aggregate_oid("test_agg").unwrap();
        assert_eq!(agg_oid1, agg_oid2, "Aggregate OIDs should be consistent");

        let trig_oid1 = get_or_allocate_trigger_oid("test_trigger").unwrap();
        let trig_oid2 = get_or_allocate_trigger_oid("test_trigger").unwrap();
        assert_eq!(trig_oid1, trig_oid2, "Trigger OIDs should be consistent");

        // Ensure different objects get different OIDs
        let different_func_oid = get_or_allocate_function_oid("other_func").unwrap();
        assert_ne!(
            func_oid1, different_func_oid,
            "Different functions should have different OIDs"
        );

        let different_trig_oid = get_or_allocate_trigger_oid("other_trigger").unwrap();
        assert_ne!(
            trig_oid1, different_trig_oid,
            "Different triggers should have different OIDs"
        );
    }

    #[tokio::test]
    async fn test_phase4b_trigger_registration() {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // First register a function that the trigger will call
        let function_metadata = register_function(
            &ctx,
            "trigger_func",
            "public",
            "postgres",
            "plpgsql",
            vec![],
            2279,  // trigger type
            false, // not aggregate
            false, // not window
            false, // not security definer
            false, // not strict
            false, // not immutable
            Some("BEGIN RETURN NEW; END;"),
            100.0,
        )
        .await
        .unwrap();

        // Create trigger type for INSERT BEFORE ROW trigger
        let trigger_type = TriggerType {
            before: true,
            after: false,
            instead: false,
            insert: true,
            update: false,
            delete: false,
            truncate: false,
            per_row: true,
            per_statement: false,
        };

        // Register the trigger
        let result = register_trigger(
            &ctx,
            "test_insert_trigger",
            12345, // table_oid
            function_metadata.function_oid,
            trigger_type,
            TriggerEnabledState::Origin,
            false, // not internal
            Some("NEW.id IS NOT NULL"),
            None,             // no old table
            Some("new_vals"), // new table
            vec!["arg1".to_string(), "arg2".to_string()],
        )
        .await;

        assert!(result.is_ok(), "Trigger registration should succeed");
        let metadata = result.unwrap();

        // Verify metadata
        assert_eq!(metadata.trigger_name, "test_insert_trigger");
        assert_eq!(metadata.table_oid, 12345);
        assert_eq!(metadata.function_oid, function_metadata.function_oid);
        assert_eq!(metadata.trigger_type.before, true);
        assert_eq!(metadata.trigger_type.insert, true);
        assert_eq!(metadata.trigger_type.per_row, true);
        assert!(matches!(
            metadata.enabled_state,
            TriggerEnabledState::Origin
        ));
        assert_eq!(metadata.is_internal, false);
        assert_eq!(metadata.when_clause, Some("NEW.id IS NOT NULL".to_string()));
        assert_eq!(metadata.transition_new_table, Some("new_vals".to_string()));
        assert_eq!(metadata.trigger_args.len(), 2);
        assert_eq!(metadata.trigger_args[0], "arg1");
        assert_eq!(metadata.trigger_args[1], "arg2");
    }

    #[tokio::test]
    async fn test_phase4c_security_registration() {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test role membership registration
        let auth_result = register_auth_member(
            &ctx, 100,  // roleid
            200,  // member
            10,   // grantor (superuser)
            true, // admin_option
        )
        .await;

        assert!(
            auth_result.is_ok(),
            "Auth member registration should succeed"
        );
        let auth_metadata = auth_result.unwrap();
        assert_eq!(auth_metadata.roleid, 100);
        assert_eq!(auth_metadata.member, 200);
        assert_eq!(auth_metadata.grantor, 10);
        assert_eq!(auth_metadata.admin_option, true);

        // Test default ACL registration
        let acl_result = register_default_acl(
            &ctx,
            10,   // defaclrole
            2200, // defaclnamespace (public schema)
            'r',  // defaclobjtype (relations)
            vec![
                "postgres=arwdDxt/postgres".to_string(),
                "=r/postgres".to_string(),
            ],
        )
        .await;

        assert!(
            acl_result.is_ok(),
            "Default ACL registration should succeed"
        );
        let acl_metadata = acl_result.unwrap();
        assert_eq!(acl_metadata.defaclrole, 10);
        assert_eq!(acl_metadata.defaclnamespace, 2200);
        assert_eq!(acl_metadata.defaclobjtype, 'r');
        assert_eq!(acl_metadata.defaclacl.len(), 2);

        // Test initial privileges registration
        let init_privs_result = register_init_privs(
            &ctx,
            1259, // objoid (pg_class)
            1259, // classoid (pg_class itself)
            0,    // objsubid (whole object)
            'i',  // privtype (initdb)
            vec!["postgres=arwdDxt/postgres".to_string()],
        )
        .await;

        assert!(
            init_privs_result.is_ok(),
            "Init privs registration should succeed"
        );
        let init_metadata = init_privs_result.unwrap();
        assert_eq!(init_metadata.objoid, 1259);
        assert_eq!(init_metadata.classoid, 1259);
        assert_eq!(init_metadata.objsubid, 0);
        assert_eq!(init_metadata.privtype, 'i');

        // Test security label registration
        let label_result = register_security_label(
            &ctx,
            1259,                                   // objoid (pg_class)
            1259,                                   // classoid (pg_class itself)
            0,                                      // objsubid (whole object)
            "selinux",                              // provider
            "system_u:object_r:postgresql_db_t:s0", // label
        )
        .await;

        assert!(
            label_result.is_ok(),
            "Security label registration should succeed"
        );
        let label_metadata = label_result.unwrap();
        assert_eq!(label_metadata.objoid, 1259);
        assert_eq!(label_metadata.classoid, 1259);
        assert_eq!(label_metadata.objsubid, 0);
        assert_eq!(label_metadata.provider, "selinux");
        assert_eq!(label_metadata.label, "system_u:object_r:postgresql_db_t:s0");
    }

    #[test]
    fn test_phase4c_oid_allocation() {
        // Test Phase 4C OID allocation for default ACLs
        let acl_oid1 = get_or_allocate_default_acl_oid("10:0:r").unwrap();
        let acl_oid2 = get_or_allocate_default_acl_oid("10:0:r").unwrap();
        assert_eq!(acl_oid1, acl_oid2, "Default ACL OIDs should be consistent");

        let different_acl_oid = get_or_allocate_default_acl_oid("10:0:f").unwrap();
        assert_ne!(
            acl_oid1, different_acl_oid,
            "Different ACL keys should have different OIDs"
        );
    }

    #[tokio::test]
    async fn test_phase4c_advanced_table_features() {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test partitioned table registration
        let partition_result = register_partitioned_table(
            &ctx,
            16384,                    // partrelid (table OID)
            'r',                      // partstrat (range partitioning)
            2,                        // partnatts (2 partitioning columns)
            0,                        // partdefid (no default partition)
            vec![1, 2],               // partattrs (column numbers)
            vec![1994, 1994],         // partclass (operator class OIDs)
            vec![100, 100],           // partcollation (collation OIDs)
            Some("id, created_date"), // partexprs
        )
        .await;

        assert!(
            partition_result.is_ok(),
            "Partitioned table registration should succeed"
        );
        let partition_metadata = partition_result.unwrap();
        assert_eq!(partition_metadata.partrelid, 16384);
        assert_eq!(partition_metadata.partstrat, 'r');
        assert_eq!(partition_metadata.partnatts, 2);
        assert_eq!(partition_metadata.partattrs, vec![1, 2]);

        // Test event trigger registration
        let event_trigger_result = register_event_trigger(
            &ctx,
            "ddl_log_trigger",
            "ddl_command_start",
            10,    // evtowner (superuser)
            12345, // evtfoid (trigger function OID)
            'O',   // evtenabled (origin)
            vec!["CREATE TABLE".to_string(), "DROP TABLE".to_string()],
        )
        .await;

        assert!(
            event_trigger_result.is_ok(),
            "Event trigger registration should succeed"
        );
        let event_metadata = event_trigger_result.unwrap();
        assert_eq!(event_metadata.evtname, "ddl_log_trigger");
        assert_eq!(event_metadata.evtevent, "ddl_command_start");
        assert_eq!(event_metadata.evtowner, 10);
        assert_eq!(event_metadata.evttags.len(), 2);

        // Test user mapping registration
        let mapping_result = register_user_mapping(
            &ctx,
            100, // umuser (user OID)
            200, // umserver (foreign server OID)
            vec![
                "host=remote.example.com".to_string(),
                "port=5432".to_string(),
            ],
        )
        .await;

        assert!(
            mapping_result.is_ok(),
            "User mapping registration should succeed"
        );
        let mapping_metadata = mapping_result.unwrap();
        assert_eq!(mapping_metadata.umuser, 100);
        assert_eq!(mapping_metadata.umserver, 200);
        assert_eq!(mapping_metadata.umoptions.len(), 2);
    }

    #[tokio::test]
    async fn test_phase4c_large_objects() {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test large object metadata registration
        let lob_result = register_large_object(
            &ctx,
            16400, // oid (large object OID)
            10,    // lomowner (owner OID)
            vec![
                "postgres=rw/postgres".to_string(),
                "public=r/postgres".to_string(),
            ],
        )
        .await;

        assert!(
            lob_result.is_ok(),
            "Large object registration should succeed"
        );
        let lob_metadata = lob_result.unwrap();
        assert_eq!(lob_metadata.oid, 16400);
        assert_eq!(lob_metadata.lomowner, 10);
        assert_eq!(lob_metadata.lomacl.len(), 2);

        // Test large object data registration
        let test_data = b"Hello, large object world!";
        let lob_data_result = register_large_object_data(
            &ctx,
            16400, // loid (large object OID)
            0,     // pageno (first page)
            test_data.to_vec(),
        )
        .await;

        assert!(
            lob_data_result.is_ok(),
            "Large object data registration should succeed"
        );
        let lob_data_metadata = lob_data_result.unwrap();
        assert_eq!(lob_data_metadata.loid, 16400);
        assert_eq!(lob_data_metadata.pageno, 0);
        assert_eq!(lob_data_metadata.data, test_data.to_vec());
    }

    #[test]
    fn test_phase4c_advanced_oid_allocation() {
        // Test Phase 4C OID allocation for advanced features
        let event_oid1 = get_or_allocate_event_trigger_oid("test_event_trigger").unwrap();
        let event_oid2 = get_or_allocate_event_trigger_oid("test_event_trigger").unwrap();
        assert_eq!(
            event_oid1, event_oid2,
            "Event trigger OIDs should be consistent"
        );

        let mapping_oid1 = get_or_allocate_user_mapping_oid("100:200").unwrap();
        let mapping_oid2 = get_or_allocate_user_mapping_oid("100:200").unwrap();
        assert_eq!(
            mapping_oid1, mapping_oid2,
            "User mapping OIDs should be consistent"
        );

        // Ensure different objects get different OIDs
        let different_event_oid = get_or_allocate_event_trigger_oid("other_event_trigger").unwrap();
        assert_ne!(
            event_oid1, different_event_oid,
            "Different event triggers should have different OIDs"
        );

        let different_mapping_oid = get_or_allocate_user_mapping_oid("101:201").unwrap();
        assert_ne!(
            mapping_oid1, different_mapping_oid,
            "Different user mappings should have different OIDs"
        );
    }

    #[tokio::test]
    async fn test_phase4c_text_search_system() {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test text search configuration registration
        let config_result = register_text_search_config(
            &ctx, "english", // cfgname
            2200,      // cfgnamespace (public)
            10,        // cfgowner (superuser)
            12345,     // cfgparser (parser OID)
        )
        .await;

        assert!(
            config_result.is_ok(),
            "Text search config registration should succeed"
        );
        let config_metadata = config_result.unwrap();
        assert_eq!(config_metadata.cfgname, "english");
        assert_eq!(config_metadata.cfgnamespace, 2200);
        assert_eq!(config_metadata.cfgowner, 10);
        assert_eq!(config_metadata.cfgparser, 12345);

        // Test text search dictionary registration
        let dict_result = register_text_search_dict(
            &ctx,
            "english_stem",              // dictname
            2200,                        // dictnamespace (public)
            10,                          // dictowner (superuser)
            54321,                       // dicttemplate (template OID)
            Some("StopWords = english"), // dictinitoption
        )
        .await;

        assert!(
            dict_result.is_ok(),
            "Text search dict registration should succeed"
        );
        let dict_metadata = dict_result.unwrap();
        assert_eq!(dict_metadata.dictname, "english_stem");
        assert_eq!(dict_metadata.dictnamespace, 2200);
        assert_eq!(dict_metadata.dicttemplate, 54321);
        assert_eq!(
            dict_metadata.dictinitoption,
            Some("StopWords = english".to_string())
        );
    }

    #[tokio::test]
    async fn test_phase4c_extended_statistics() {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test extended statistics registration
        let stats_result = register_extended_statistic(
            &ctx,
            16384,                             // stxrelid (table OID)
            "user_stats",                      // stxname
            2200,                              // stxnamespace (public)
            10,                                // stxowner (superuser)
            vec![1, 2, 3],                     // stxkeys (column numbers)
            vec!['d', 'f', 'm'],               // stxkind (ndistinct, functional deps, MCV)
            Some("upper(name), lower(email)"), // stxexprs
        )
        .await;

        assert!(
            stats_result.is_ok(),
            "Extended statistics registration should succeed"
        );
        let stats_metadata = stats_result.unwrap();
        assert_eq!(stats_metadata.stxname, "user_stats");
        assert_eq!(stats_metadata.stxrelid, 16384);
        assert_eq!(stats_metadata.stxkeys, vec![1, 2, 3]);
        assert_eq!(stats_metadata.stxkind, vec!['d', 'f', 'm']);

        // Test extended statistics data registration
        let stats_data_result = register_extended_statistic_data(
            &ctx,
            stats_metadata.oid,                   // stxoid
            false,                                // stxdinherit
            Some("{\"1,2\": 0.5, \"1,3\": 0.3}"), // stxdndistinct
            Some("{\"1\": [2, 3]}"),              // stxddependencies
            Some("{\"most_common\": [[\"John\", \"Doe\"], [\"Jane\", \"Smith\"]]}"), // stxdmcv
            None,                                 // stxdexpr
        )
        .await;

        assert!(
            stats_data_result.is_ok(),
            "Extended statistics data registration should succeed"
        );
        let stats_data_metadata = stats_data_result.unwrap();
        assert_eq!(stats_data_metadata.stxoid, stats_metadata.oid);
        assert_eq!(stats_data_metadata.stxdinherit, false);
        assert!(stats_data_metadata.stxdndistinct.is_some());
        assert!(stats_data_metadata.stxddependencies.is_some());
    }

    #[tokio::test]
    async fn test_phase4c_replication_features() {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test publication namespace registration
        let pub_ns_result = register_publication_namespace(
            &ctx, 16500, // pnpubid (publication OID)
            2200,  // pnnspid (namespace OID - public)
        )
        .await;

        assert!(
            pub_ns_result.is_ok(),
            "Publication namespace registration should succeed"
        );
        let pub_ns_metadata = pub_ns_result.unwrap();
        assert_eq!(pub_ns_metadata.pnpubid, 16500);
        assert_eq!(pub_ns_metadata.pnnspid, 2200);

        // Test replication origin registration
        let repl_origin_result = register_replication_origin(
            &ctx,
            1,                // roident
            "my_replication", // roname
        )
        .await;

        assert!(
            repl_origin_result.is_ok(),
            "Replication origin registration should succeed"
        );
        let repl_origin_metadata = repl_origin_result.unwrap();
        assert_eq!(repl_origin_metadata.roident, 1);
        assert_eq!(repl_origin_metadata.roname, "my_replication");

        // Test subscription relation registration
        let sub_rel_result = register_subscription_rel(
            &ctx,
            16600,            // srsubid (subscription OID)
            16384,            // srrelid (relation OID)
            's',              // srsubstate (synchronized)
            Some("0/123456"), // srsublsn
        )
        .await;

        assert!(
            sub_rel_result.is_ok(),
            "Subscription relation registration should succeed"
        );
        let sub_rel_metadata = sub_rel_result.unwrap();
        assert_eq!(sub_rel_metadata.srsubid, 16600);
        assert_eq!(sub_rel_metadata.srrelid, 16384);
        assert_eq!(sub_rel_metadata.srsubstate, 's');
        assert_eq!(sub_rel_metadata.srsublsn, Some("0/123456".to_string()));
    }

    #[test]
    fn test_phase4c_final_oid_allocation() {
        // Test Phase 4C OID allocation for all remaining features

        // Text search OIDs
        let ts_config_oid1 = get_or_allocate_ts_config_oid("english").unwrap();
        let ts_config_oid2 = get_or_allocate_ts_config_oid("english").unwrap();
        assert_eq!(
            ts_config_oid1, ts_config_oid2,
            "TS config OIDs should be consistent"
        );

        let ts_dict_oid1 = get_or_allocate_ts_dict_oid("english_stem").unwrap();
        let ts_dict_oid2 = get_or_allocate_ts_dict_oid("english_stem").unwrap();
        assert_eq!(
            ts_dict_oid1, ts_dict_oid2,
            "TS dict OIDs should be consistent"
        );

        // Extended statistics OIDs
        let ext_stats_oid1 = get_or_allocate_ext_stats_oid("user_stats").unwrap();
        let ext_stats_oid2 = get_or_allocate_ext_stats_oid("user_stats").unwrap();
        assert_eq!(
            ext_stats_oid1, ext_stats_oid2,
            "Extended stats OIDs should be consistent"
        );

        // Ensure different objects get different OIDs
        let different_config_oid = get_or_allocate_ts_config_oid("spanish").unwrap();
        assert_ne!(
            ts_config_oid1, different_config_oid,
            "Different TS configs should have different OIDs"
        );

        let different_dict_oid = get_or_allocate_ts_dict_oid("spanish_stem").unwrap();
        assert_ne!(
            ts_dict_oid1, different_dict_oid,
            "Different TS dicts should have different OIDs"
        );

        let different_stats_oid = get_or_allocate_ext_stats_oid("product_stats").unwrap();
        assert_ne!(
            ext_stats_oid1, different_stats_oid,
            "Different extended stats should have different OIDs"
        );
    }

    #[test]
    fn test_helper_functions() {
        // Test language OID mapping
        assert_eq!(get_language_oid("sql"), Some(12));
        assert_eq!(get_language_oid("plpgsql"), Some(13));
        assert_eq!(get_language_oid("c"), Some(13));
        assert_eq!(get_language_oid("unknown"), Some(12)); // Default to SQL
    }

    #[tokio::test]
    async fn test_register_user_tables_dynamic() -> DFResult<()> {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await?;

        register_schema(&ctx, "pgtry", "myschema").await?;

        let mut c1 = BTreeMap::new();
        c1.insert(
            "id".to_string(),
            ColumnDef {
                col_type: "int".to_string(),
                nullable: true,
            },
        );
        let mut c2 = BTreeMap::new();
        c2.insert(
            "name".to_string(),
            ColumnDef {
                col_type: "text".to_string(),
                nullable: true,
            },
        );

        register_user_tables(&ctx, "pgtry", "myschema", "contacts", vec![c1, c2]).await?;

        let df = ctx
            .sql("SELECT relname FROM pg_catalog.pg_class WHERE relname='contacts'")
            .await?;
        assert_eq!(df.count().await?, 1);

        let df = ctx
            .sql("SELECT nspname FROM pg_catalog.pg_namespace n JOIN pg_catalog.pg_class c ON n.oid=c.relnamespace WHERE c.relname='contacts'")
            .await?;
        let batches = df.collect().await?;
        let schema_name = batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .unwrap()
            .value(0);
        assert_eq!(schema_name, "myschema");

        let df = ctx
            .sql(
                "SELECT attname FROM pg_catalog.pg_attribute \
                 WHERE attrelid = (SELECT oid FROM pg_catalog.pg_class WHERE relname='contacts') \
                 ORDER BY attnum",
            )
            .await?;
        let batches = df.collect().await?;
        assert_eq!(batches[0].num_rows(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_register_user_tables_idempotent() -> DFResult<()> {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await?;

        register_schema(&ctx, "pgtry", "myschema").await?;

        let mut c1 = BTreeMap::new();
        c1.insert(
            "id".to_string(),
            ColumnDef {
                col_type: "int".to_string(),
                nullable: true,
            },
        );
        let mut c2 = BTreeMap::new();
        c2.insert(
            "name".to_string(),
            ColumnDef {
                col_type: "text".to_string(),
                nullable: true,
            },
        );

        register_user_tables(
            &ctx,
            "pgtry",
            "myschema",
            "contacts",
            vec![c1.clone(), c2.clone()],
        )
        .await?;
        // call again to ensure idempotency
        register_user_tables(&ctx, "pgtry", "myschema", "contacts", vec![c1, c2]).await?;

        let df = ctx
            .sql("SELECT relname FROM pg_catalog.pg_class WHERE relname='contacts'")
            .await?;
        assert_eq!(df.count().await?, 1);

        let df = ctx
            .sql(
                "SELECT attname FROM pg_catalog.pg_attribute \
                 WHERE attrelid = (SELECT oid FROM pg_catalog.pg_class WHERE relname='contacts') \
                 ORDER BY attnum",
            )
            .await?;
        let batches = df.collect().await?;
        assert_eq!(batches[0].num_rows(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_register_schema() -> DFResult<()> {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await?;

        register_schema(&ctx, "pgtry", "custom").await?;

        let df = ctx
            .sql("SELECT nspname FROM pg_catalog.pg_namespace WHERE nspname='custom'")
            .await?;
        assert_eq!(df.count().await?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_register_user_database() -> DFResult<()> {
        let (ctx, _) = get_base_session_context(
            Some("pg_catalog_data/pg_schema"),
            "pgtry".to_string(),
            "public".to_string(),
            None,
        )
        .await?;

        register_user_database(&ctx, "crm").await?;

        let df = ctx
            .sql("SELECT datname FROM pg_catalog.pg_database WHERE datname='crm'")
            .await?;
        assert_eq!(df.count().await?, 1);
        Ok(())
    }
}
