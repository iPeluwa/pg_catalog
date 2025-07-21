# PostgreSQL Catalog Hybrid Approach Design

## Overview

This document outlines the hybrid approach for implementing PostgreSQL's 64 system catalog tables, balancing performance, completeness, and maintainability.

## Table Classification

### Static Tables (Exported Data Approach)
These tables contain PostgreSQL system data that rarely changes and can be pre-exported:

**Core Type System (High Priority):**
- `pg_type` - Type definitions (~475KB YAML)
- `pg_description` - Object descriptions (~485KB YAML)
- `pg_operator` - Operator definitions
- `pg_proc` - Function definitions
- `pg_cast` - Type casting rules

**System Configuration:**
- `pg_language` - Procedural languages
- `pg_opclass` - Operator classes
- `pg_opfamily` - Operator families
- `pg_collation` - Collation information

### Dynamic Tables (TableProvider Approach)
These tables reflect current database state and must be dynamically generated:

**Schema Metadata (High Priority):**
- `pg_class` - Tables, indexes, sequences, views
- `pg_attribute` - Table columns
- `pg_namespace` - Schemas
- `pg_constraint` - Table constraints
- `pg_index` - Index definitions

**Database Structure:**
- `pg_database` - Available databases
- `pg_tablespace` - Tablespace definitions
- `pg_user` / `pg_authid` - User accounts
- `pg_depend` - Object dependencies

## Implementation Strategy

### Phase 1: Foundation (Complete ✅)
- ✅ OID cache system for consistent allocation
- ✅ Range validation (static: 1-50000, dynamic: 50000+)
- ✅ Lazy loading for large static tables
- ✅ Integration with existing pg_class/pg_namespace dynamic tables

### Phase 2: Core Static Tables (Complete ✅)
- ✅ Implement lazy loading for `pg_type` and `pg_description`
- ✅ Add OID consistency across static table references
- ✅ Optimize YAML parsing and caching strategies
- ✅ Performance monitoring and timing optimization
- ✅ Cross-table consistency validation
- ✅ Thread-safe caching with pre-population

### Phase 3: Enhanced Dynamic Tables (Complete ✅)
- ✅ Extend dynamic table providers with richer metadata
- ✅ Add support for indexes, constraints, dependencies  
- ✅ Implement cross-table OID consistency
- ✅ Enhanced metadata caching system
- ✅ Constraint and index OID allocation
- ✅ Dependency tracking in pg_depend
- ✅ Resilient registration for missing tables

### Phase 4A: Critical Tables (Complete ✅)
- ✅ Core database introspection tables (pg_database, pg_tablespace, pg_authid)
- ✅ Enhanced metadata with full PostgreSQL compatibility
- ✅ Bootstrap system objects and template databases
- ✅ User authentication and role management system
- ✅ Tablespace management and storage locations

### Phase 4B: High Priority Tables (Complete ✅)
- ✅ Function and operator catalog tables (pg_proc, pg_operator, pg_cast)
- ✅ Advanced constraint and trigger tables (pg_trigger, pg_rewrite)
- ✅ Aggregate function metadata (pg_aggregate)
- ✅ Enhanced static table integration
- ✅ Comprehensive OID allocation and caching
- ✅ Full test coverage for all registration functions

### Phase 4C: Complete Coverage (Complete ✅)
- ✅ Security & access control tables (pg_auth_members, pg_default_acl, pg_init_privs, pg_seclabel)
- ✅ Comprehensive OID allocation and metadata caching for security objects
- ✅ Full test coverage for security registration functions
- ✅ Advanced table features (pg_partitioned_table, pg_event_trigger, pg_user_mapping)
- ✅ Large object support (pg_largeobject, pg_largeobject_metadata)
- ✅ Binary data handling with hex encoding for large object storage
- ✅ Comprehensive partitioning support (range, list, hash strategies)
- ✅ Event trigger system for DDL command monitoring
- ✅ Text search system (pg_ts_config, pg_ts_dict, pg_ts_parser, pg_ts_template)
- ✅ Extended statistics (pg_statistic_ext, pg_statistic_ext_data)
- ✅ Advanced replication features (pg_publication_namespace, pg_replication_origin, pg_subscription_rel)
- ✅ Complete PostgreSQL catalog coverage (60+ tables implemented)
- ✅ Comprehensive test coverage for all Phase 4C features
- ✅ Performance monitoring and optimization enhancements
- ✅ Support for PostgreSQL version-specific differences

## Technical Architecture

### OID Management
```
Static OIDs:     1 - 50,000     (from YAML data)
Dynamic OIDs: 50,001 - ∞        (allocated via NEXT_OID counter)
```

### Lazy Loading Pattern
```rust
static TABLE_DATA: Lazy<Mutex<Option<TableData>>> = Lazy::new(|| Mutex::new(None));

fn load_table_data() -> Result<()> {
    let mut data = TABLE_DATA.lock().unwrap();
    if data.is_none() {
        // Load YAML, parse, cache
        *data = Some(parsed_data);
    }
    Ok(())
}
```

### Integration Points
- Session context initialization
- Query planning and optimization
- PostgreSQL wire protocol compatibility
- CLI tools (`\dt`, `\d+`, etc.)

## Performance Considerations

### Startup Time
- Lazy loading reduces initial memory footprint
- Critical tables loaded on-demand
- Background pre-warming for frequently accessed tables

### Memory Usage
- Static tables: Load once, cache indefinitely
- Dynamic tables: Regenerate as needed
- Configurable cache eviction policies

### Query Performance
- OID cache ensures consistent joins
- Pre-built indexes on commonly queried columns
- Materialized view patterns for complex queries

## Compatibility Matrix

| Table | PostgreSQL 13 | PostgreSQL 14 | PostgreSQL 15 | PostgreSQL 16 |
|-------|---------------|---------------|---------------|---------------|
| pg_type | ✅ | ✅ | ✅ | ✅ |
| pg_description | ✅ | ✅ | ✅ | ✅ |
| pg_class | ✅ | ✅ | ✅ | ✅ |
| pg_attribute | ✅ | ✅ | ✅ | ✅ |
| pg_namespace | ✅ | ✅ | ✅ | ✅ |

## Usage Examples

### Static Table Access
```sql
-- Loads pg_type lazily on first access
SELECT typname, typlen FROM pg_catalog.pg_type WHERE typname = 'int4';
```

### Dynamic Table Access
```sql
-- Uses current schema state
SELECT relname FROM pg_catalog.pg_class WHERE relnamespace = 
  (SELECT oid FROM pg_catalog.pg_namespace WHERE nspname = 'public');
```

### Cross-Table Queries (With Consistent OIDs)
```sql
-- OID cache ensures consistent joins
SELECT c.relname, t.typname 
FROM pg_catalog.pg_class c
JOIN pg_catalog.pg_type t ON c.reltype = t.oid;
```

## Implementation Priority

1. **Phase 1** (Complete ✅): OID foundation, lazy loading
2. **Phase 2** (Complete ✅): pg_type and pg_description optimization  
3. **Phase 3** (Complete ✅): Enhanced pg_class, pg_attribute, pg_constraint
4. **Phase 4A** (Complete ✅): Critical database, tablespace, user tables
5. **Phase 4B** (Complete ✅): Function, operator, aggregate, trigger tables
6. **Phase 4C** (Complete ✅): Security, partitioning, text search, large objects, replication

## Benefits

- **Performance**: Lazy loading, efficient caching
- **Completeness**: Support for all 64 PostgreSQL catalog tables
- **Consistency**: OID cache prevents join failures
- **Maintainability**: Clear separation of static vs dynamic data
- **Compatibility**: Works with existing PostgreSQL tools and queries
