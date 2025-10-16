/// Inserts a value into the specified database table using Diesel.
///
/// # Arguments
/// * `$table` - The path to the Diesel table into which the value will be inserted.
/// * `$value` - The value to insert. Should implement the appropriate Diesel traits for insertion.
/// * `$conn` - The database connection to use for the operation.
///
/// # Errors
/// Returns a `ModuleError::InternalError` if the insertion fails, including the error message and table name.
///
/// # Example
/// ```rust
/// let mut conn = pool.get().await.map_err(|e| ModuleError::InternalError(e.to_string()))?;
/// let user_task = UserTasks { user_id: payload.user_id,task_id: payload.subject};
/// insert!(user_tasks::table, user_task, conn);
/// ```
#[macro_export]
macro_rules! insert {
    ($table:path, $value:ident, $conn:ident) => {
        diesel::insert_into($table)
            .values(&$value)
            .execute($conn)
            .map_err(|e| {
                ModuleError::InternalError(format!(
                    "Error Inserting into {} : {}",
                    e,
                    stringify!($table)
                ))
            })?;
    };
}

/// Updates a record in the specified database table using Diesel ORM.
///
/// # Arguments
/// * `$table` - The path to the database table to update.
/// * `$value` - The struct containing updated values to set.
/// * `$conn` - The database connection to use for the update.
///
/// # Errors
/// Returns a `ModuleError::InternalError` if the update operation fails, including the error message and table name.
///
/// # Example
/// ```rust
/// update!(users::table, updated_user, conn);
/// ```
#[macro_export]
macro_rules! update {
    ($table:path, $value:ident, $conn:ident) => {
        diesel::update($table)
            .set(&$value)
            .execute($conn)
            .map_err(|e| {
                ModuleError::InternalError(format!("Error Updating {} : {}", e, stringify!($table)))
            })?;
    };
}

/// Macro to simplify fetching data from a database table using Diesel ORM.
///
/// # Variants
///
/// ## 1. Fetch all rows from a table
/// ```rust
/// fetch!(table_path, ReturnType, conn)
/// ```
/// - `table_path`: Path to the Diesel table.
/// - `ReturnType`: Type to map the result to (must implement `Selectable`).
/// - `conn`: Database connection (must be mutable).
///
/// Returns a `Vec<ReturnType>` or propagates a `ModuleError::InternalError` on failure.
///
/// ## 2. Fetch a single row by filter
/// ```rust
/// fetch!(table_path, filter_path, value, ReturnType, conn)
/// ```
/// - `table_path`: Path to the Diesel table.
/// - `filter_path`: Path to the column to filter by.
/// - `value`: Value to match for the filter.
/// - `ReturnType`: Type to map the result to (must implement `Selectable`).
/// - `conn`: Database connection (must be mutable).
///
/// Returns a single `ReturnType` if found, or propagates a `ModuleError::ItemNotFound` if not found,
/// or a `ModuleError::InternalError` on query failure.
///
/// # Errors
/// - Returns `ModuleError::InternalError` if the database query fails.
/// - Returns `ModuleError::ItemNotFound` if no matching item is found (for the filtered variant).
#[macro_export]
macro_rules! fetch {
    ($table:path, $return_type:ty, $conn:ident) => {
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::SelectableHelper;
        $table
            .select(<$return_type>::as_select())
            .load(&mut $conn)
            .map_err(|e| {
                ModuleError::InternalError(format!(
                    "Error fetching {} from {}: {}",
                    stringify!($return_type),
                    stringify!($table),
                    e
                ))
            })?
    };
    ($table:path, $filter:path, $value:expr, $return_type:ty, $conn:ident) => {{
        use diesel::ExpressionMethods;
        use diesel::OptionalExtension;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::SelectableHelper;
        $table
            .filter($filter.eq($value))
            .select(<$return_type>::as_select())
            .first(&mut $conn)
            .optional()
            .map_err(|e| {
                ModuleError::InternalError(format!(
                    "Error fetching {}: {}",
                    $value,
                    e
                ))
            })?
            .ok_or(ModuleError::ItemNotFound(format!(
                "{} not found in system",
                $value
            )))?
    }};
}
