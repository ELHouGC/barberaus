use cqrs_es::CqrsFramework;
use cqrs_es::persist::PersistedEventStore;
use postgres_es::PostgresEventRepository;
use sqlx::PgPool;

use crate::domain::user::account_aggregate::UserAccountAggregate;

pub type UserEventStore = PersistedEventStore<PostgresEventRepository, UserAccountAggregate>;
pub type UserCqrsFramework = CqrsFramework<UserAccountAggregate, UserEventStore>;

pub fn create_cqrs_framework(pool: PgPool) -> UserCqrsFramework {
    let repo = PostgresEventRepository::new(pool);
    let event_store = PersistedEventStore::new_event_store(repo);

    CqrsFramework::new(event_store, vec![], ())
}
