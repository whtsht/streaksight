use serde::{Deserialize, Serialize};
use sqlparser::ast::{
    BinaryOperator, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArgumentList,
    FunctionArguments, GroupByExpr, Ident, LimitClause, ObjectName, OrderBy, OrderByExpr,
    OrderByKind, OrderByOptions, SelectItem, SetExpr, Statement, UnaryOperator, Value,
    ValueWithSpan,
};
use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Span;

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeGraph {
    pub selected_node_id: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Deserialize)]
struct TableNodeData {
    table_name: String,
}

#[derive(Debug, Deserialize)]
struct SelectNodeData {
    #[serde(default)]
    columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SortNodeData {
    #[serde(default)]
    order: Vec<OrderByData>,
}

#[derive(Debug, Deserialize)]
struct LimitNodeData {
    #[serde(default)]
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize)]
struct OrderByData {
    column: String,
    direction: OrderDirection,
}

#[derive(Debug, Deserialize)]
enum FilterOperator {
    #[serde(rename = "==")]
    Eq,
    #[serde(rename = "!=")]
    NotEq,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = ">=")]
    GtEq,
    #[serde(rename = "<=")]
    LtEq,
    #[serde(rename = "in")]
    In,
}

#[derive(Debug, Deserialize)]
struct FilterNodeData {
    #[serde(default)]
    conditions: Vec<FilterCondition>,
}

#[derive(Debug, Deserialize)]
struct FilterCondition {
    column: String,
    operator: FilterOperator,
    value: serde_json::Value,
    #[serde(default)]
    negate: bool,
}

#[derive(Debug, Deserialize)]
enum AggregateFunction {
    #[serde(rename = "COUNT(*)")]
    CountAll,
    #[serde(rename = "COUNT")]
    Count,
    #[serde(rename = "SUM")]
    Sum,
    #[serde(rename = "AVG")]
    Avg,
    #[serde(rename = "MAX")]
    Max,
    #[serde(rename = "MIN")]
    Min,
}

#[derive(Debug, Deserialize)]
struct AggregationNodeData {
    #[serde(default)]
    dimensions: Vec<String>,
    #[serde(default)]
    metrics: Vec<Metric>,
}

#[derive(Debug, Deserialize)]
struct Metric {
    function: AggregateFunction,
    #[serde(default)]
    column: String,
}

pub fn generate_sql(
    node_graph: &NodeGraph,
    pagination: Option<(i64, i64)>,
) -> Result<String, String> {
    let path = build_path(node_graph)?;

    let mut table_name = String::new();
    let mut columns = Vec::<String>::new();
    let mut order_by_list = Vec::<OrderByData>::new();
    let mut limit_value: Option<i64> = None;
    let mut filter_conditions = Vec::<FilterCondition>::new();
    let mut aggregation_data: Option<AggregationNodeData> = None;
    let mut has_select_before_aggregation = false;

    for node in &path {
        match node.node_type.as_str() {
            "table" => {
                let table_data: TableNodeData = serde_json::from_value(node.data.clone())
                    .map_err(|e| format!("Failed to parse table node data: {}", e))?;
                table_name = table_data.table_name;
            }
            "select" => {
                let select_data: SelectNodeData = serde_json::from_value(node.data.clone())
                    .map_err(|e| format!("Failed to parse select node data: {}", e))?;
                columns = select_data.columns;
                if aggregation_data.is_none() {
                    has_select_before_aggregation = true;
                }
            }
            "sort" => {
                let sort_data: SortNodeData = serde_json::from_value(node.data.clone())
                    .map_err(|e| format!("Failed to parse sort node data: {}", e))?;
                order_by_list = sort_data.order;
            }
            "limit" => {
                let limit_data: LimitNodeData = serde_json::from_value(node.data.clone())
                    .map_err(|e| format!("Failed to parse limit node data: {}", e))?;
                limit_value = limit_data.limit;
            }
            "filter" => {
                let filter_data: FilterNodeData = serde_json::from_value(node.data.clone())
                    .map_err(|e| format!("Failed to parse filter node data: {}", e))?;
                filter_conditions.extend(filter_data.conditions);
            }
            "aggregation" => {
                let agg_data: AggregationNodeData = serde_json::from_value(node.data.clone())
                    .map_err(|e| format!("Failed to parse aggregation node data: {}", e))?;

                if has_select_before_aggregation {
                    return Err("Cannot use Aggregation after Select node. Please remove the Select node or reorder the nodes.".to_string());
                }

                aggregation_data = Some(agg_data);
            }
            _ => {
                return Err(format!("Unsupported node type: {}", node.node_type));
            }
        }
    }

    if table_name.is_empty() {
        return Err("No table node found in path".to_string());
    }

    let dialect = DuckDbDialect {};
    let base_sql = format!("SELECT * FROM {}", table_name);
    let mut ast = Parser::parse_sql(&dialect, &base_sql)
        .map_err(|e| format!("Failed to parse base SQL: {}", e))?;

    if ast.is_empty() {
        return Err("Failed to generate base AST".to_string());
    }

    if let Statement::Query(ref mut query) = ast[0] {
        if let SetExpr::Select(ref mut select) = *query.body {
            if let Some(agg) = &aggregation_data {
                if !agg.dimensions.is_empty() || !agg.metrics.is_empty() {
                    select.projection = build_aggregation_projection(agg)?;
                }
            } else if !columns.is_empty() {
                select.projection = columns
                    .iter()
                    .map(|col| SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(col))))
                    .collect();
            }

            if !filter_conditions.is_empty() {
                if let Ok(where_expr) = build_where_expr(&filter_conditions) {
                    select.selection = Some(where_expr);
                }
            }

            if let Some(agg) = &aggregation_data {
                if !agg.dimensions.is_empty() {
                    select.group_by = GroupByExpr::Expressions(
                        agg.dimensions
                            .iter()
                            .map(|dim| Expr::Identifier(Ident::new(dim)))
                            .collect(),
                        vec![],
                    );
                }
            }
        }

        if !order_by_list.is_empty() {
            let order_by_exprs: Vec<OrderByExpr> = order_by_list
                .iter()
                .map(|o| OrderByExpr {
                    expr: Expr::Identifier(Ident::new(&o.column)),
                    options: OrderByOptions {
                        asc: Some(matches!(o.direction, OrderDirection::Asc)),
                        nulls_first: None,
                    },
                    with_fill: None,
                })
                .collect();
            query.order_by = Some(OrderBy {
                kind: OrderByKind::Expressions(order_by_exprs),
                interpolate: None,
            });
        }

        if let Some(limit) = limit_value {
            query.limit_clause = Some(LimitClause::LimitOffset {
                limit: Some(Expr::Value(ValueWithSpan {
                    value: Value::Number(limit.to_string(), false),
                    span: Span::empty(),
                })),
                offset: None,
                limit_by: Vec::new(),
            });
        }
    }

    let inner_sql = ast[0].to_string();

    if let Some((limit, offset)) = pagination {
        Ok(format!(
            "SELECT * FROM ({}) AS subquery LIMIT {} OFFSET {}",
            inner_sql, limit, offset
        ))
    } else {
        Ok(inner_sql)
    }
}

fn build_path(node_graph: &NodeGraph) -> Result<Vec<&Node>, String> {
    let mut path = Vec::new();
    let mut current_id = node_graph.selected_node_id.clone();

    loop {
        let current_node = node_graph
            .nodes
            .iter()
            .find(|n| n.id == current_id)
            .ok_or_else(|| format!("Node not found: {}", current_id))?;

        path.push(current_node);

        if let Some(edge) = node_graph.edges.iter().find(|e| e.target == current_id) {
            current_id = edge.source.clone();
        } else {
            break;
        }
    }

    path.reverse();

    Ok(path)
}

fn build_where_expr(conditions: &[FilterCondition]) -> Result<Expr, String> {
    if conditions.is_empty() {
        return Err("No filter conditions provided".to_string());
    }

    let valid_conditions: Vec<&FilterCondition> = conditions
        .iter()
        .filter(|c| !is_empty_value(&c.value))
        .collect();

    if valid_conditions.is_empty() {
        return Err("No valid filter conditions (all have empty values)".to_string());
    }

    let exprs: Result<Vec<Expr>, String> = valid_conditions
        .iter()
        .map(|c| condition_to_expr(c))
        .collect();
    let exprs = exprs?;

    // Note: Combine all conditions with AND
    let mut result = exprs[0].clone();
    for expr in &exprs[1..] {
        result = Expr::BinaryOp {
            left: Box::new(result),
            op: BinaryOperator::And,
            right: Box::new(expr.clone()),
        };
    }

    Ok(result)
}

fn is_empty_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Array(arr) => arr.is_empty() || arr.iter().all(is_empty_value),
        _ => false,
    }
}

fn filter_operator_to_binary_operator(op: &FilterOperator) -> Option<BinaryOperator> {
    match op {
        FilterOperator::Eq => Some(BinaryOperator::Eq),
        FilterOperator::NotEq => Some(BinaryOperator::NotEq),
        FilterOperator::Gt => Some(BinaryOperator::Gt),
        FilterOperator::Lt => Some(BinaryOperator::Lt),
        FilterOperator::GtEq => Some(BinaryOperator::GtEq),
        FilterOperator::LtEq => Some(BinaryOperator::LtEq),
        FilterOperator::In => None,
    }
}

fn condition_to_expr(condition: &FilterCondition) -> Result<Expr, String> {
    let column_expr = Expr::Identifier(Ident::new(&condition.column));

    let base_expr = if let Some(binary_op) = filter_operator_to_binary_operator(&condition.operator)
    {
        let value = parse_value(&condition.value)?;
        Expr::BinaryOp {
            left: Box::new(column_expr),
            op: binary_op,
            right: Box::new(value),
        }
    } else {
        let values = parse_array_values(&condition.value)?;
        Expr::InList {
            expr: Box::new(column_expr),
            list: values,
            negated: false,
        }
    };

    if condition.negate {
        Ok(Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr: Box::new(base_expr),
        })
    } else {
        Ok(base_expr)
    }
}

fn parse_value(value: &serde_json::Value) -> Result<Expr, String> {
    match value {
        serde_json::Value::String(s) => Ok(Expr::Value(ValueWithSpan {
            value: Value::SingleQuotedString(s.clone()),
            span: Span::empty(),
        })),
        serde_json::Value::Number(n) => Ok(Expr::Value(ValueWithSpan {
            value: Value::Number(n.to_string(), false),
            span: Span::empty(),
        })),
        serde_json::Value::Bool(b) => Ok(Expr::Value(ValueWithSpan {
            value: Value::Boolean(*b),
            span: Span::empty(),
        })),
        _ => Err(format!("Unsupported value type: {:?}", value)),
    }
}

fn parse_array_values(value: &serde_json::Value) -> Result<Vec<Expr>, String> {
    match value {
        serde_json::Value::Array(arr) => arr.iter().map(parse_value).collect(),
        _ => Err("Expected array for 'in' operator".to_string()),
    }
}

fn build_aggregation_projection(agg: &AggregationNodeData) -> Result<Vec<SelectItem>, String> {
    let mut projection = Vec::new();

    for dim in &agg.dimensions {
        projection.push(SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(dim))));
    }

    for metric in &agg.metrics {
        let func_expr = create_aggregate_function(metric)?;
        projection.push(SelectItem::UnnamedExpr(func_expr));
    }

    Ok(projection)
}

fn aggregate_function_name(func: &AggregateFunction) -> &'static str {
    match func {
        AggregateFunction::CountAll => "COUNT",
        AggregateFunction::Count => "COUNT",
        AggregateFunction::Sum => "SUM",
        AggregateFunction::Avg => "AVG",
        AggregateFunction::Max => "MAX",
        AggregateFunction::Min => "MIN",
    }
}

fn create_aggregate_args(metric: &Metric) -> Vec<FunctionArg> {
    match &metric.function {
        AggregateFunction::CountAll => vec![FunctionArg::Unnamed(FunctionArgExpr::Wildcard)],
        _ => vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(
            Expr::Identifier(Ident::new(&metric.column)),
        ))],
    }
}

fn create_aggregate_function(metric: &Metric) -> Result<Expr, String> {
    let func_name = aggregate_function_name(&metric.function);
    let args = create_aggregate_args(metric);

    Ok(Expr::Function(Function {
        name: ObjectName(vec![sqlparser::ast::ObjectNamePart::Identifier(
            Ident::new(func_name),
        )]),
        parameters: sqlparser::ast::FunctionArguments::None,
        args: FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args,
            clauses: vec![],
        }),
        filter: None,
        null_treatment: None,
        over: None,
        within_group: vec![],
        uses_odbc_syntax: false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sql_table_only() {
        let json = r#"{
            "selected_node_id": "1",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}}
            ],
            "edges": []
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_generate_sql_table_with_select_and_limit() {
        let json = r#"{
            "selected_node_id": "3",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "select", "data": {"columns": ["id", "name"]}},
                {"id": "3", "type": "limit", "data": {"limit": 10}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(sql, "SELECT id, name FROM users LIMIT 10");
    }

    #[test]
    fn test_generate_sql_table_with_select_sort_and_limit() {
        let json = r#"{
            "selected_node_id": "4",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "select", "data": {"columns": ["id", "name"]}},
                {"id": "3", "type": "sort", "data": {"order": [{"column": "id", "direction": "desc"}]}},
                {"id": "4", "type": "limit", "data": {"limit": 5}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"},
                {"source": "3", "target": "4"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(sql, "SELECT id, name FROM users ORDER BY id DESC LIMIT 5");
    }

    #[test]
    fn test_generate_sql_table_with_multiple_order() {
        let json = r#"{
            "selected_node_id": "4",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "orders"}},
                {"id": "2", "type": "select", "data": {"columns": ["id", "customer", "total"]}},
                {"id": "3", "type": "sort", "data": {
                    "order": [
                        {"column": "customer", "direction": "asc"},
                        {"column": "total", "direction": "desc"}
                    ]
                }},
                {"id": "4", "type": "limit", "data": {"limit": 20}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"},
                {"source": "3", "target": "4"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT id, customer, total FROM orders ORDER BY customer ASC, total DESC LIMIT 20"
        );
    }

    #[test]
    fn test_generate_sql_select_table_node() {
        let json = r#"{
            "selected_node_id": "1",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "select", "data": {"columns": ["id", "name"]}},
                {"id": "3", "type": "limit", "data": {"limit": 10}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(sql, "SELECT * FROM products");
    }

    #[test]
    fn test_generate_sql_order_independent() {
        let json = r#"{
            "selected_node_id": "4",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "limit", "data": {"limit": 10}},
                {"id": "3", "type": "sort", "data": {"order": [{"column": "name", "direction": "asc"}]}},
                {"id": "4", "type": "select", "data": {"columns": ["id", "name"]}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"},
                {"source": "3", "target": "4"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(sql, "SELECT id, name FROM users ORDER BY name ASC LIMIT 10");
    }

    #[test]
    fn test_generate_sql_with_single_filter() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "filter", "data": {"conditions": [{"column": "price", "operator": ">=", "value": 1000}]}}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(sql, "SELECT * FROM users WHERE price >= 1000");
    }

    #[test]
    fn test_generate_sql_with_multiple_filters() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "filter", "data": {"conditions": [
                    {"column": "price", "operator": ">=", "value": 1000},
                    {"column": "city", "operator": "==", "value": "Tokyo"}
                ]}}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM users WHERE price >= 1000 AND city = 'Tokyo'"
        );
    }

    #[test]
    fn test_generate_sql_with_in_operator() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "filter", "data": {"conditions": [
                    {"column": "name", "operator": "in", "value": ["Taro", "Jiro", "Saburo"]}
                ]}}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM users WHERE name IN ('Taro', 'Jiro', 'Saburo')"
        );
    }

    #[test]
    fn test_generate_sql_with_negated_condition() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "filter", "data": {"conditions": [
                    {"column": "city", "operator": "==", "value": "Tokyo", "negate": true}
                ]}}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(sql, "SELECT * FROM users WHERE NOT city = 'Tokyo'");
    }

    #[test]
    fn test_generate_sql_filter_with_select_sort_limit() {
        let json = r#"{
            "selected_node_id": "5",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "filter", "data": {"conditions": [{"column": "price", "operator": ">", "value": 100}]}},
                {"id": "3", "type": "select", "data": {"columns": ["id", "name", "price"]}},
                {"id": "4", "type": "sort", "data": {"order": [{"column": "price", "direction": "desc"}]}},
                {"id": "5", "type": "limit", "data": {"limit": 10}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"},
                {"source": "3", "target": "4"},
                {"source": "4", "target": "5"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT id, name, price FROM products WHERE price > 100 ORDER BY price DESC LIMIT 10"
        );
    }

    #[test]
    fn test_aggregation_basic() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "aggregation", "data": {
                    "dimensions": ["category"],
                    "metrics": [{"function": "COUNT(*)", "column": ""}]
                }}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY category"
        );
    }

    #[test]
    fn test_aggregation_multiple_metrics() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "aggregation", "data": {
                    "dimensions": ["category"],
                    "metrics": [
                        {"function": "COUNT(*)", "column": ""},
                        {"function": "SUM", "column": "price"},
                        {"function": "AVG", "column": "price"}
                    ]
                }}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT category, COUNT(*), SUM(price), AVG(price) FROM products GROUP BY category"
        );
    }

    #[test]
    fn test_aggregation_multiple_dimensions() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "aggregation", "data": {
                    "dimensions": ["category", "region"],
                    "metrics": [{"function": "COUNT(*)", "column": ""}]
                }}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT category, region, COUNT(*) FROM products GROUP BY category, region"
        );
    }

    #[test]
    fn test_aggregation_with_filter() {
        let json = r#"{
            "selected_node_id": "3",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "filter", "data": {"conditions": [{"column": "price", "operator": ">", "value": 100}]}},
                {"id": "3", "type": "aggregation", "data": {
                    "dimensions": ["category"],
                    "metrics": [{"function": "COUNT(*)", "column": ""}]
                }}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products WHERE price > 100 GROUP BY category"
        );
    }

    #[test]
    fn test_aggregation_then_select() {
        let json = r#"{
            "selected_node_id": "3",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "aggregation", "data": {
                    "dimensions": ["category"],
                    "metrics": [{"function": "COUNT(*)", "column": ""}]
                }},
                {"id": "3", "type": "select", "data": {"columns": ["category"]}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY category"
        );
    }

    #[test]
    fn test_select_then_aggregation_error() {
        let json = r#"{
            "selected_node_id": "3",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "select", "data": {"columns": ["id", "name"]}},
                {"id": "3", "type": "aggregation", "data": {
                    "dimensions": ["category"],
                    "metrics": [{"function": "COUNT(*)", "column": ""}]
                }}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let result = generate_sql(&node_graph, None);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot use Aggregation after Select node. Please remove the Select node or reorder the nodes."
        );
    }

    #[test]
    fn test_aggregation_all_functions() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "aggregation", "data": {
                    "dimensions": ["category"],
                    "metrics": [
                        {"function": "COUNT", "column": "id"},
                        {"function": "SUM", "column": "price"},
                        {"function": "AVG", "column": "price"},
                        {"function": "MAX", "column": "price"},
                        {"function": "MIN", "column": "price"}
                    ]
                }}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, None).unwrap();

        assert_eq!(
            sql,
            "SELECT category, COUNT(id), SUM(price), AVG(price), MAX(price), MIN(price) FROM products GROUP BY category"
        );
    }

    #[test]
    fn test_pagination_without_limit_node() {
        let json = r#"{
            "selected_node_id": "1",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}}
            ],
            "edges": []
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, Some((100, 0))).unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM (SELECT * FROM users) AS subquery LIMIT 100 OFFSET 0"
        );
    }

    #[test]
    fn test_pagination_with_offset() {
        let json = r#"{
            "selected_node_id": "1",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}}
            ],
            "edges": []
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, Some((100, 200))).unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM (SELECT * FROM users) AS subquery LIMIT 100 OFFSET 200"
        );
    }

    #[test]
    fn test_pagination_with_limit_node() {
        let json = r#"{
            "selected_node_id": "2",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "users"}},
                {"id": "2", "type": "limit", "data": {"limit": 10}}
            ],
            "edges": [
                {"source": "1", "target": "2"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, Some((100, 0))).unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM (SELECT * FROM users LIMIT 10) AS subquery LIMIT 100 OFFSET 0"
        );
    }

    #[test]
    fn test_pagination_with_complex_query() {
        let json = r#"{
            "selected_node_id": "4",
            "nodes": [
                {"id": "1", "type": "table", "data": {"table_name": "products"}},
                {"id": "2", "type": "filter", "data": {"conditions": [{"column": "price", "operator": ">", "value": 100}]}},
                {"id": "3", "type": "sort", "data": {"order": [{"column": "price", "direction": "desc"}]}},
                {"id": "4", "type": "select", "data": {"columns": ["id", "name", "price"]}}
            ],
            "edges": [
                {"source": "1", "target": "2"},
                {"source": "2", "target": "3"},
                {"source": "3", "target": "4"}
            ]
        }"#;

        let node_graph: NodeGraph = serde_json::from_str(json).unwrap();
        let sql = generate_sql(&node_graph, Some((50, 100))).unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM (SELECT id, name, price FROM products WHERE price > 100 ORDER BY price DESC) AS subquery LIMIT 50 OFFSET 100"
        );
    }
}
