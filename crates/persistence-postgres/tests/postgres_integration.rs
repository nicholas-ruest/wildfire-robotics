//! `PostgreSQL` integration tests. Set `TEST_DATABASE_URL` to run locally or in CI.
//! A missing database is reported as a skip so developer machines do not require Docker,
//! unless `REQUIRE_INTEGRATION_TESTS=true` makes container execution a release gate.

use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;
use wildfire_persistence_postgres::{
    AggregateRecord, NewOutboxMessage, PersistenceError, acquire_lease, begin_serializable,
    claim_outbox, enqueue_outbox, fail_or_quarantine, mark_published, migrate, record_inbox,
    store_aggregate,
};

struct TestDatabase {
    pool: PgPool,
    _container: Option<ContainerAsync<Postgres>>,
}

#[tokio::test]
async fn contradictory_duplicate_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let Some(database) = test_pool().await? else {
        return Ok(());
    };
    let tenant = Uuid::new_v4();
    let message = Uuid::new_v4();
    let mut first = database.pool.begin().await?;
    assert!(
        record_inbox(
            &mut first,
            tenant,
            "consumer",
            message,
            Some("operation"),
            b"first"
        )
        .await?
    );
    first.commit().await?;
    let mut replay = database.pool.begin().await?;
    assert!(matches!(
        record_inbox(
            &mut replay,
            tenant,
            "consumer",
            message,
            Some("operation"),
            b"changed"
        )
        .await,
        Err(PersistenceError::ContradictoryDuplicate)
    ));
    replay.rollback().await?;
    Ok(())
}

async fn test_pool() -> Result<Option<TestDatabase>, Box<dyn std::error::Error>> {
    if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        let pool = wildfire_persistence_postgres::connect(&url, 5).await?;
        migrate(&pool).await?;
        return Ok(Some(TestDatabase {
            pool,
            _container: None,
        }));
    }
    let container = match Postgres::default().start().await {
        Ok(container) => container,
        Err(error) => {
            if std::env::var("REQUIRE_INTEGRATION_TESTS").as_deref() == Ok("true") {
                return Err(format!(
                    "PostgreSQL integration tests are required but Docker is unavailable: {error}"
                )
                .into());
            }
            eprintln!("skipped PostgreSQL integration test: Docker unavailable: {error}");
            return Ok(None);
        }
    };
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let pool = wildfire_persistence_postgres::connect(&url, 5).await?;
    migrate(&pool).await?;
    migrate(&pool).await?;
    Ok(Some(TestDatabase {
        pool,
        _container: Some(container),
    }))
}

#[tokio::test]
async fn aggregate_and_outbox_commit_or_roll_back_together()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(database) = test_pool().await? else {
        return Ok(());
    };
    let pool = database.pool;
    let tenant = Uuid::new_v4();
    let aggregate = Uuid::new_v4();
    let message = Uuid::new_v4();
    let mut transaction = begin_serializable(&pool).await?;
    let version = store_aggregate(
        &mut transaction,
        &AggregateRecord {
            tenant_id: tenant,
            aggregate_type: "fixture".into(),
            aggregate_id: aggregate,
            expected_version: 0,
            state: json!({"state":"ready"}),
        },
    )
    .await?;
    enqueue_outbox(
        &mut transaction,
        &NewOutboxMessage {
            tenant_id: tenant,
            message_id: message,
            aggregate_type: "fixture",
            aggregate_id: aggregate,
            aggregate_version: version,
            subject: "wr.test.fixture",
            content_type: "application/protobuf",
            classification: "INTERNAL",
            payload: b"event",
            occurred_at: Utc::now(),
        },
    )
    .await?;
    transaction.rollback().await?;
    let aggregate_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM wr_infra.aggregate_state WHERE tenant_id=$1 AND aggregate_id=$2",
    )
    .bind(tenant)
    .bind(aggregate)
    .fetch_one(&pool)
    .await?;
    let outbox_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM wr_infra.outbox WHERE tenant_id=$1 AND message_id=$2",
    )
    .bind(tenant)
    .bind(message)
    .fetch_one(&pool)
    .await?;
    assert_eq!((aggregate_count, outbox_count), (0, 0));
    Ok(())
}

#[tokio::test]
async fn duplicate_inbox_has_one_business_effect() -> Result<(), Box<dyn std::error::Error>> {
    let Some(database) = test_pool().await? else {
        return Ok(());
    };
    let pool = database.pool;
    let tenant = Uuid::new_v4();
    let message = Uuid::new_v4();
    let mut first = pool.begin().await?;
    assert!(
        record_inbox(
            &mut first,
            tenant,
            "fixture-consumer",
            message,
            Some("business-1"),
            b"event"
        )
        .await?
    );
    first.commit().await?;
    let mut duplicate = pool.begin().await?;
    assert!(
        !record_inbox(
            &mut duplicate,
            tenant,
            "fixture-consumer",
            message,
            Some("business-1"),
            b"event"
        )
        .await?
    );
    duplicate.commit().await?;
    Ok(())
}

#[tokio::test]
async fn expired_claim_is_recovered_and_lease_tokens_fence_stale_holders()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(database) = test_pool().await? else {
        return Ok(());
    };
    let pool = database.pool;
    let tenant = Uuid::new_v4();
    let aggregate = Uuid::new_v4();
    let message = Uuid::new_v4();
    let mut transaction = pool.begin().await?;
    enqueue_outbox(
        &mut transaction,
        &NewOutboxMessage {
            tenant_id: tenant,
            message_id: message,
            aggregate_type: "fixture",
            aggregate_id: aggregate,
            aggregate_version: 1,
            subject: "wr.test.fixture",
            content_type: "application/protobuf",
            classification: "INTERNAL",
            payload: b"event",
            occurred_at: Utc::now(),
        },
    )
    .await?;
    transaction.commit().await?;
    assert_eq!(
        claim_outbox(&pool, "relay-a", 1, Duration::milliseconds(1))
            .await?
            .len(),
        1
    );
    sqlx::query("UPDATE wr_infra.outbox SET claimed_at=transaction_timestamp()-interval '1 second' WHERE tenant_id=$1 AND message_id=$2").bind(tenant).bind(message).execute(&pool).await?;
    assert_eq!(
        claim_outbox(&pool, "relay-b", 1, Duration::milliseconds(1))
            .await?
            .len(),
        1
    );
    assert!(!mark_published(&pool, tenant, message, "relay-a").await?);
    assert!(mark_published(&pool, tenant, message, "relay-b").await?);

    let first = acquire_lease(&pool, tenant, "resource", "holder-a", Duration::seconds(30))
        .await?
        .ok_or("first lease was not acquired")?;
    assert!(
        acquire_lease(&pool, tenant, "resource", "holder-b", Duration::seconds(30))
            .await?
            .is_none()
    );
    sqlx::query("UPDATE wr_infra.lease SET acquired_at=transaction_timestamp()-interval '2 seconds',expires_at=transaction_timestamp()-interval '1 second' WHERE tenant_id=$1 AND resource_key='resource'").bind(tenant).execute(&pool).await?;
    let second = acquire_lease(&pool, tenant, "resource", "holder-b", Duration::seconds(30))
        .await?
        .ok_or("expired lease was not acquired")?;
    assert!(second.fencing_token > first.fencing_token);
    Ok(())
}

#[tokio::test]
async fn business_keys_are_unique_within_but_not_across_tenants()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(database) = test_pool().await? else {
        return Ok(());
    };
    let pool = database.pool;
    let tenant = Uuid::new_v4();
    let other_tenant = Uuid::new_v4();
    let mut first = pool.begin().await?;
    assert!(
        record_inbox(
            &mut first,
            tenant,
            "consumer",
            Uuid::new_v4(),
            Some("business-key"),
            b"payload"
        )
        .await?
    );
    first.commit().await?;
    let mut duplicate = pool.begin().await?;
    assert!(
        !record_inbox(
            &mut duplicate,
            tenant,
            "consumer",
            Uuid::new_v4(),
            Some("business-key"),
            b"payload"
        )
        .await?
    );
    duplicate.commit().await?;
    let mut other = pool.begin().await?;
    assert!(
        record_inbox(
            &mut other,
            other_tenant,
            "consumer",
            Uuid::new_v4(),
            Some("business-key"),
            b"payload"
        )
        .await?
    );
    other.commit().await?;
    Ok(())
}

#[tokio::test]
async fn poison_message_is_quarantined_without_tenant_leakage()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(database) = test_pool().await? else {
        return Ok(());
    };
    let pool = database.pool;
    let tenant = Uuid::new_v4();
    let other_tenant = Uuid::new_v4();
    let message = Uuid::new_v4();
    let mut transaction = pool.begin().await?;
    enqueue_outbox(
        &mut transaction,
        &NewOutboxMessage {
            tenant_id: tenant,
            message_id: message,
            aggregate_type: "fixture",
            aggregate_id: Uuid::new_v4(),
            aggregate_version: 1,
            subject: "wr.test.fixture",
            content_type: "application/protobuf",
            classification: "RESTRICTED",
            payload: b"poison",
            occurred_at: Utc::now(),
        },
    )
    .await?;
    transaction.commit().await?;
    assert_eq!(
        claim_outbox(&pool, "relay", 1, Duration::seconds(30))
            .await?
            .len(),
        1
    );
    assert!(
        fail_or_quarantine(
            &pool,
            tenant,
            message,
            "relay",
            "invalid schema",
            1,
            Duration::zero()
        )
        .await?
    );
    assert!(
        claim_outbox(&pool, "relay", 1, Duration::seconds(30))
            .await?
            .is_empty()
    );
    let quarantined: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM wr_infra.poison_message WHERE tenant_id=$1 AND message_id=$2",
    )
    .bind(tenant)
    .bind(message)
    .fetch_one(&pool)
    .await?;
    let leaked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM wr_infra.poison_message WHERE tenant_id=$1 AND message_id=$2",
    )
    .bind(other_tenant)
    .bind(message)
    .fetch_one(&pool)
    .await?;
    assert_eq!((quarantined, leaked), (1, 0));
    Ok(())
}
