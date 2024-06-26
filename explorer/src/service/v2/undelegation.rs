use crate::service::error::Result;
use crate::service::QueryResult;
use crate::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetUndelegationByTxHashParams {
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct UndelegationResponse {
    pub tx_hash: String,
    pub block_hash: String,
    pub from: String,
    pub new_delegator: String,
    pub target_validator: String,
    pub amount: u64,
    pub height: i64,
    pub timestamp: i64,
    pub value: Value,
}

pub async fn get_undelegation_by_tx_hash(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetUndelegationByTxHashParams>,
) -> Result<Json<UndelegationResponse>> {
    let mut conn = state.pool.acquire().await?;
    let sql_query = r#"SELECT tx,block,sender,amount,target_validator,new_delegator,height,timestamp,content
        FROM undelegations WHERE tx=$1"#;

    let row = sqlx::query(sql_query)
        .bind(params.hash)
        .fetch_one(&mut *conn)
        .await?;

    let tx_hash: String = row.try_get("tx")?;
    let block_hash: String = row.try_get("block")?;
    let from: String = row.try_get("sender")?;
    let amount: i64 = row.try_get("amount")?;
    let target_validator: String = row.try_get("target_validator")?;
    let new_delegator: String = row.try_get("new_delegator")?;
    let height: i64 = row.try_get("height")?;
    let timestamp: i64 = row.try_get("timestamp")?;
    let value: Value = row.try_get("content")?;

    let undelegation = UndelegationResponse {
        tx_hash,
        block_hash,
        from,
        new_delegator,
        target_validator,
        amount: amount as u64,
        height,
        timestamp,
        value,
    };

    Ok(Json(undelegation))
}

#[derive(Serialize, Deserialize)]
pub struct GetUndelegationsParams {
    pub from: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

pub async fn get_undelegations(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetUndelegationsParams>,
) -> Result<Json<QueryResult<Vec<UndelegationResponse>>>> {
    let mut conn = state.pool.acquire().await?;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

    let (sql_count, sql_query) = if let Some(from) = params.from {
        (
            format!(
                "SELECT count(height) FROM undelegations WHERE sender='{}'",
                from.to_lowercase()
            ),
            format!(
            "SELECT tx,block,sender,amount,target_validator,new_delegator,height,timestamp,content \
            FROM undelegations WHERE sender='{}' ORDER BY timestamp DESC LIMIT {} OFFSET {}",
            from.to_lowercase(), page_size, (page-1)*page_size),
        )
    } else {
        (
            "SELECT count(height) FROM undelegations".to_string(),
            format!("SELECT tx,block,sender,amount,target_validator,new_delegator,height,timestamp,content \
            FROM undelegations ORDER BY timestamp DESC LIMIT {} OFFSET {}", page_size, (page-1)*page_size)
        )
    };

    let row_cnt = sqlx::query(sql_count.as_str())
        .fetch_one(&mut *conn)
        .await?;
    let total: i64 = row_cnt.try_get("count")?;

    let mut undelegations: Vec<UndelegationResponse> = vec![];
    let rows = sqlx::query(sql_query.as_str())
        .fetch_all(&mut *conn)
        .await?;
    for row in rows {
        let tx_hash: String = row.try_get("tx")?;
        let block_hash: String = row.try_get("block")?;
        let from: String = row.try_get("sender")?;
        let amount: i64 = row.try_get("amount")?;
        let target_validator: String = row.try_get("target_validator")?;
        let new_delegator: String = row.try_get("new_delegator")?;
        let height: i64 = row.try_get("height")?;
        let timestamp: i64 = row.try_get("timestamp")?;
        let value: Value = row.try_get("content")?;
        undelegations.push(UndelegationResponse {
            tx_hash,
            block_hash,
            from,
            new_delegator,
            target_validator,
            amount: amount as u64,
            height,
            timestamp,
            value,
        });
    }

    Ok(Json(QueryResult {
        total,
        page,
        page_size,
        data: undelegations,
    }))
}
