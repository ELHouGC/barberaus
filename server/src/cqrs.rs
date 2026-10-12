use cqrs_es::persist::PersistedEventStore;
use cqrs_es::{CqrsFramework, persist::GenericQuery};
use postgres_es::{PostgresEventRepository, PostgresViewRepository};
use sqlx::PgPool;
use std::sync::Arc;

use crate::application::segas::email_verification::EmailVerificationSaga;
use crate::domain::user::{
    account_aggregate::UserAccountAggregate, client_profile_aggregate::ClientProfileAggregate,
};
use crate::domain::views::unified_user_view::UnifiedUserView;

pub type UserEventStore = PersistedEventStore<PostgresEventRepository, UserAccountAggregate>;
pub type UserCqrsFramework = CqrsFramework<UserAccountAggregate, UserEventStore>;

pub type ClientProfileEventStore =
    PersistedEventStore<PostgresEventRepository, ClientProfileAggregate>;
pub type ClientProfileCqrsFramework =
    CqrsFramework<ClientProfileAggregate, ClientProfileEventStore>;

// View repository type for read-side queries
pub type UserViewRepository = PostgresViewRepository<UnifiedUserView, UserAccountAggregate>;

pub fn create_cqrs_framework(
    pool: PgPool,
) -> (
    UserCqrsFramework,
    Arc<ClientProfileCqrsFramework>,
    Arc<UserViewRepository>,
) {
    // 1. Client Profile View Query Setup
    let client_view_repo = Arc::new(PostgresViewRepository::<
        UnifiedUserView,
        ClientProfileAggregate,
    >::new("user_views", pool.clone()));
    let client_view_query = GenericQuery::new(Arc::clone(&client_view_repo));

    let client_repo = PostgresEventRepository::new(pool.clone());
    let client_event_store = PersistedEventStore::new_event_store(client_repo);
    let client_cqrs = Arc::new(CqrsFramework::new(
        client_event_store,
        vec![Box::new(client_view_query)],
        (),
    ));

    // 2. User Account View Query & Saga Setup
    let user_view_repo = Arc::new(PostgresViewRepository::<
        UnifiedUserView,
        UserAccountAggregate,
    >::new("user_views", pool.clone()));
    let user_view_query = GenericQuery::new(Arc::clone(&user_view_repo));

    let user_repo = PostgresEventRepository::new(pool);
    let user_event_store = PersistedEventStore::new_event_store(user_repo);
    let email_verification_saga = EmailVerificationSaga::new(Arc::clone(&client_cqrs));

    let user_cqrs = CqrsFramework::new(
        user_event_store,
        vec![Box::new(user_view_query), Box::new(email_verification_saga)],
        (),
    );

    (user_cqrs, client_cqrs, user_view_repo)
}
