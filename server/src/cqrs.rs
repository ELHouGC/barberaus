use cqrs_es::CqrsFramework;
use cqrs_es::persist::PersistedEventStore;
use postgres_es::PostgresEventRepository;
use sqlx::PgPool;

use crate::domain::user::user_aggregate::UserAggregate;

pub type UserEventStore = PersistedEventStore<PostgresEventRepository, UserAggregate>;
pub type UserCqrsFramework = CqrsFramework<UserAggregate, UserEventStore>;

pub fn create_cqrs_framework(pool: PgPool) -> UserCqrsFramework {
    let repo = PostgresEventRepository::new(pool);
    let event_store = PersistedEventStore::new_event_store(repo);

    CqrsFramework::new(event_store, vec![], ())
}
