use sqlx::{Executor, Sqlite};

/// Base trait for all repository types
pub trait Repository: Send + Sync {
    type Model;
    type Id;
}

/// Create operations (implementers define their own signatures)
pub trait RepositoryCreate: Repository {}

/// Find/read operations
pub trait RepositoryFind: Repository {
    async fn find_by_id<'e, E>(executor: E, id: Self::Id) -> anyhow::Result<Option<Self::Model>>
    where
        E: Executor<'e, Database = Sqlite>;
}

/// List operations
pub trait RepositoryList: Repository {
    async fn list<'e, E>(executor: E) -> anyhow::Result<Vec<Self::Model>>
    where
        E: Executor<'e, Database = Sqlite>;
}

/// Delete operations
pub trait RepositoryDelete: Repository {
    async fn delete<'e, E>(executor: E, id: Self::Id) -> anyhow::Result<()>
    where
        E: Executor<'e, Database = Sqlite>;
}
