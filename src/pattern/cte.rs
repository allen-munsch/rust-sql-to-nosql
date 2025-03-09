// pattern/cte.rs - Common Table Expression pattern matchers
// Declarative patterns for matching SQL WITH clauses (CTEs)

use sqlparser::ast::{
    ObjectNamePart, Query, Statement, TableWithJoins, With
};

use super::combinators::{Pattern, extract};

/// Information about a CTE
#[derive(Debug, Clone)]
pub struct CteInfo {
    /// The name of the CTE
    pub name: String,
    /// Optional column names
    pub column_names: Vec<String>,
    /// The query that defines the CTE
    pub query: Box<Query>,
    /// Whether this is a recursive CTE
    pub recursive: bool,
}

/// Pattern that matches a statement with a WITH clause
pub fn statement_with_cte() -> impl Pattern<Statement, With> {
    extract(|stmt: &Statement| {
        match stmt {
            Statement::Query(query) => {
                query.with.clone()
            },
            _ => None,
        }
    })
}


/// Pattern that extracts all CTEs from a WITH clause
pub fn extract_ctes() -> impl Pattern<With, Vec<CteInfo>> {
    extract(|with: &With| {
        let ctes: Vec<CteInfo> = with.cte_tables.iter()
            .map(|cte| CteInfo {
                name: cte.alias.name.value.clone(),
                column_names: cte.alias.columns.iter()
                    .map(|col| col.name.value.clone())
                    .collect(),
                query: cte.query.clone(),
                recursive: with.recursive,
            })
            .collect();
        if ctes.is_empty() {
            None
        } else {
            Some(ctes)
        }
    })
 }

/// Pattern that matches a specific CTE by name
pub fn cte_by_name(name: &str) -> impl Pattern<With, CteInfo> {
    let name = name.to_string(); // Clone the name for the closure
    extract(move |with: &With| {
        with.cte_tables.iter()
            .find(|cte| cte.alias.name.value == name)
            .map(|cte| CteInfo {
                name: cte.alias.name.value.clone(),
                column_names: cte.alias.columns.iter()
                    .map(|col| col.name.value.clone())
                    .collect(),
                query: cte.query.clone(),
                recursive: with.recursive,
            })
    })
}

/// Pattern that matches a table reference to a CTE
pub fn cte_reference() -> impl Pattern<TableWithJoins, String> {
    extract(|table: &TableWithJoins| {
        match &table.relation {
            sqlparser::ast::TableFactor::Table { name, .. } => {
                if !name.0.is_empty() {
                    match &name.0[0] {
                        ObjectNamePart::Identifier(ident) => {
                            // In a real implementation, we would check if this table name
                            // references a CTE defined in a WITH clause. Here we're just
                            // extracting the name for demonstration purposes.
                            Some(ident.value.clone())
                        }
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    })
}

/// Extract all CTEs from a statement
pub fn extract_all_ctes(stmt: &Statement) -> Vec<CteInfo> {
    match statement_with_cte().match_pattern(stmt) {
        Ok(with) => {
            match extract_ctes().match_pattern(&with) {
                Ok(ctes) => ctes,
                Err(_) => Vec::new(),
            }
        },
        Err(_) => Vec::new(),
    }
}

/// Check if a query references a CTE
pub fn find_cte_references(query: &Query, cte_names: &[String]) -> Vec<String> {
    let mut references = Vec::new();
    // Check FROM clause
    if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
        for table in &select.from {
            if let sqlparser::ast::TableFactor::Table { name, .. } = &table.relation {
                if !name.0.is_empty() {
                    match &name.0[0] {
                        ObjectNamePart::Identifier(ident) => {
                            let table_name = &ident.value;
                            if cte_names.contains(table_name) {
                                references.push(table_name.clone());
                            }
                        }
                    }
                }
            }
            // Check JOINed tables
            for join in &table.joins {
                if let sqlparser::ast::TableFactor::Table { name, .. } = &join.relation {
                    if !name.0.is_empty() {
                        match &name.0[0] {
                            ObjectNamePart::Identifier(ident) => {
                                let table_name = &ident.value;
                                if cte_names.contains(table_name) {
                                    references.push(table_name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    references
}

/// Check if a CTE is recursive (references itself)
pub fn is_recursive_cte(cte: &CteInfo, all_ctes: &[CteInfo]) -> bool {
    // Simple case - already marked as recursive
    if cte.recursive {
        return true;
    }
    
    // Check if the CTE references itself directly
    let cte_names: Vec<String> = all_ctes.iter()
        .map(|c| c.name.clone())
        .collect();
    
    let references = find_cte_references(&cte.query, &cte_names);
    references.contains(&cte.name)
}