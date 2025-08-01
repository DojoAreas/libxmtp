use crate::group::ConversationType;
use crate::local_commit_log::LocalCommitLog;
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use diesel::prelude::SqliteConnection;
use mockall::mock;
use parking_lot::Mutex;

use crate::{ConnectionError, ConnectionExt, TransactionGuard};
pub type MockDb = MockDbQuery<MockConnection>;

#[derive(Clone)]
pub struct MockConnection {
    inner: Arc<Mutex<SqliteConnection>>,
    in_transaction: Arc<AtomicBool>,
}

impl std::fmt::Debug for MockConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockConnection")
    }
}

impl AsRef<MockConnection> for MockConnection {
    fn as_ref(&self) -> &MockConnection {
        self
    }
}

// TODO: We should use diesels test transaction
impl ConnectionExt for MockConnection {
    type Connection = SqliteConnection;

    fn start_transaction(&self) -> Result<crate::TransactionGuard, crate::ConnectionError> {
        self.in_transaction.store(true, Ordering::SeqCst);

        Ok(TransactionGuard {
            in_transaction: self.in_transaction.clone(),
        })
    }

    fn raw_query_read<T, F>(&self, fun: F) -> Result<T, crate::ConnectionError>
    where
        F: FnOnce(&mut Self::Connection) -> Result<T, diesel::result::Error>,
        Self: Sized,
    {
        let mut conn = self.inner.lock();
        fun(&mut conn).map_err(ConnectionError::from)
    }

    fn raw_query_write<T, F>(&self, fun: F) -> Result<T, crate::ConnectionError>
    where
        F: FnOnce(&mut Self::Connection) -> Result<T, diesel::result::Error>,
        Self: Sized,
    {
        let mut conn = self.inner.lock();
        fun(&mut conn).map_err(ConnectionError::from)
    }

    fn is_in_transaction(&self) -> bool {
        self.in_transaction.load(Ordering::SeqCst)
    }

    fn disconnect(&self) -> Result<(), ConnectionError> {
        Ok(())
    }

    fn reconnect(&self) -> Result<(), ConnectionError> {
        Ok(())
    }
}

use crate::StorageError;
use crate::prelude::*;
mock! {
    pub DbQuery<C: ConnectionExt + 'static> {}

    impl<C: ConnectionExt + 'static> ReadOnly<C> for DbQuery<C> {
        fn enable_readonly(&self) -> Result<(), StorageError>;
        fn disable_readonly(&self) -> Result<(), StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryConsentRecord<C> for DbQuery<C> {
        fn get_consent_record(
            &self,
            entity: String,
            entity_type: crate::consent_record::ConsentType,
        ) -> Result<Option<crate::consent_record::StoredConsentRecord>, crate::ConnectionError>;

        fn consent_records(
            &self,
        ) -> Result<Vec<crate::consent_record::StoredConsentRecord>, crate::ConnectionError>;

        fn consent_records_paged(
            &self,
            limit: i64,
            offset: i64,
        ) -> Result<Vec<crate::consent_record::StoredConsentRecord>, crate::ConnectionError>;

        fn insert_newer_consent_record(
            &self,
            record: crate::consent_record::StoredConsentRecord,
        ) -> Result<bool, crate::ConnectionError>;

        fn insert_or_replace_consent_records(
            &self,
            records: &[crate::consent_record::StoredConsentRecord],
        ) -> Result<Vec<crate::consent_record::StoredConsentRecord>, crate::ConnectionError>;

        fn maybe_insert_consent_record_return_existing(
            &self,
            record: &crate::consent_record::StoredConsentRecord,
        ) -> Result<Option<crate::consent_record::StoredConsentRecord>, crate::ConnectionError>;

        fn find_consent_by_dm_id(
            &self,
            dm_id: &str,
        ) -> Result<Vec<crate::consent_record::StoredConsentRecord>, crate::ConnectionError>;
    }

    impl<C: ConnectionExt + 'static> QueryConversationList<C> for DbQuery<C> {
        #[mockall::concretize]
        fn fetch_conversation_list<A: AsRef<crate::group::GroupQueryArgs>>(
            &self,
            args: A,
        ) -> Result<Vec<crate::conversation_list::ConversationListItem>, StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryDms<C> for DbQuery<C> {
        fn fetch_stitched(
            &self,
            key: &[u8],
        ) -> Result<Option<crate::group::StoredGroup>, ConnectionError>;

        #[mockall::concretize]
        fn find_dm_group<M>(
            &self,
            members: M,
        ) -> Result<Option<crate::group::StoredGroup>, ConnectionError>
        where
            M: std::fmt::Display;

        fn other_dms(&self, group_id: &[u8])
        -> Result<Vec<crate::group::StoredGroup>, ConnectionError>;
    }

    impl<C: ConnectionExt + 'static> QueryGroup<C> for DbQuery<C> {
        #[mockall::concretize]
        fn find_groups<A: AsRef<crate::group::GroupQueryArgs>>(
            &self,
            args: A,
        ) -> Result<Vec<crate::group::StoredGroup>, crate::ConnectionError>;

        #[mockall::concretize]
        fn find_groups_by_id_paged<A: AsRef<crate::group::GroupQueryArgs>>(
            &self,
            args: A,
            offset: i64,
        ) -> Result<Vec<crate::group::StoredGroup>, crate::ConnectionError>;

        #[mockall::concretize]
        fn update_group_membership<GroupId: AsRef<[u8]>>(
            &self,
            group_id: GroupId,
            state: crate::group::GroupMembershipState,
        ) -> Result<(), crate::ConnectionError>;

        fn all_sync_groups(&self) -> Result<Vec<crate::group::StoredGroup>, crate::ConnectionError>;

        fn find_sync_group(
            &self,
            id: &[u8],
        ) -> Result<Option<crate::group::StoredGroup>, crate::ConnectionError>;

        fn primary_sync_group(
            &self,
        ) -> Result<Option<crate::group::StoredGroup>, crate::ConnectionError>;

        fn find_group(
            &self,
            id: &[u8],
        ) -> Result<Option<crate::group::StoredGroup>, crate::ConnectionError>;

        fn find_group_by_welcome_id(
            &self,
            welcome_id: i64,
        ) -> Result<Option<crate::group::StoredGroup>, crate::ConnectionError>;

        fn get_rotated_at_ns(&self, group_id: Vec<u8>) -> Result<i64, StorageError>;

        fn update_rotated_at_ns(&self, group_id: Vec<u8>) -> Result<(), StorageError>;

        fn get_installations_time_checked(&self, group_id: Vec<u8>) -> Result<i64, StorageError>;

        fn update_installations_time_checked(&self, group_id: Vec<u8>) -> Result<(), StorageError>;

        fn update_message_disappearing_from_ns(
            &self,
            group_id: Vec<u8>,
            from_ns: Option<i64>,
        ) -> Result<(), StorageError>;

        fn update_message_disappearing_in_ns(
            &self,
            group_id: Vec<u8>,
            in_ns: Option<i64>,
        ) -> Result<(), StorageError>;

        fn insert_or_replace_group(
            &self,
            group: crate::group::StoredGroup,
        ) -> Result<crate::group::StoredGroup, StorageError>;

        fn group_welcome_ids(&self) -> Result<Vec<i64>, crate::ConnectionError>;

        fn mark_group_as_maybe_forked(
            &self,
            group_id: &[u8],
            fork_details: String,
        ) -> Result<(), StorageError>;

        fn clear_fork_flag_for_group(&self, group_id: &[u8]) -> Result<(), crate::ConnectionError>;

        fn has_duplicate_dm(&self, group_id: &[u8]) -> Result<bool, crate::ConnectionError>;

        fn get_conversation_ids_for_remote_log_publish(&self) -> Result<Vec<Vec<u8>>, crate::ConnectionError>;

        fn get_conversation_ids_for_remote_log_download(&self) -> Result<Vec<Vec<u8>>, crate::ConnectionError>;

        fn get_conversation_type(&self, group_id: &[u8]) -> Result<ConversationType, crate::ConnectionError>;
    }

    impl<C: ConnectionExt + 'static> QueryGroupVersion<C> for DbQuery<C> {
        fn set_group_paused(&self, group_id: &[u8], min_version: &str) -> Result<(), StorageError>;

        fn unpause_group(&self, group_id: &[u8]) -> Result<(), StorageError>;

        fn get_group_paused_version(&self, group_id: &[u8]) -> Result<Option<String>, StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryGroupIntent<C> for DbQuery<C> {
        fn insert_group_intent(
            &self,
            to_save: crate::group_intent::NewGroupIntent,
        ) -> Result<crate::group_intent::StoredGroupIntent, crate::ConnectionError>;

        fn find_group_intents(
            &self,
            group_id: Vec<u8>,
            allowed_states: Option<Vec<crate::group_intent::IntentState>>,
            allowed_kinds: Option<Vec<crate::group_intent::IntentKind>>,
        ) -> Result<Vec<crate::group_intent::StoredGroupIntent>, crate::ConnectionError>;

        fn set_group_intent_published(
            &self,
            intent_id: crate::group_intent::ID,
            payload_hash: &[u8],
            post_commit_data: Option<Vec<u8>>,
            staged_commit: Option<Vec<u8>>,
            published_in_epoch: i64,
        ) -> Result<(), StorageError>;

        fn set_group_intent_committed(
            &self,
            intent_id: crate::group_intent::ID,
        ) -> Result<(), StorageError>;

        fn set_group_intent_processed(
            &self,
            intent_id: crate::group_intent::ID,
        ) -> Result<(), StorageError>;

        fn set_group_intent_to_publish(
            &self,
            intent_id: crate::group_intent::ID,
        ) -> Result<(), StorageError>;

        fn set_group_intent_error(
            &self,
            intent_id: crate::group_intent::ID,
        ) -> Result<(), StorageError>;

        fn find_group_intent_by_payload_hash(
            &self,
            payload_hash: &[u8],
        ) -> Result<Option<crate::group_intent::StoredGroupIntent>, StorageError>;

        fn increment_intent_publish_attempt_count(
            &self,
            intent_id: crate::group_intent::ID,
        ) -> Result<(), StorageError>;

        fn set_group_intent_error_and_fail_msg(
            &self,
            intent: &crate::group_intent::StoredGroupIntent,
            msg_id: Option<Vec<u8>>,
        ) -> Result<(), StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryGroupMessage<C> for DbQuery<C> {
        fn get_group_messages(
            &self,
            group_id: &[u8],
            args: &crate::group_message::MsgQueryArgs,
        ) -> Result<Vec<crate::group_message::StoredGroupMessage>, crate::ConnectionError>;

        fn group_messages_paged(
            &self,
            args: &crate::group_message::MsgQueryArgs,
            offset: i64,
        ) -> Result<Vec<crate::group_message::StoredGroupMessage>, crate::ConnectionError>;

        fn get_group_messages_with_reactions(
            &self,
            group_id: &[u8],
            args: &crate::group_message::MsgQueryArgs,
        ) -> Result<Vec<crate::group_message::StoredGroupMessageWithReactions>, crate::ConnectionError>;

        #[mockall::concretize]
        fn get_group_message<MessageId: AsRef<[u8]>>(
            &self,
            id: MessageId,
        ) -> Result<Option<crate::group_message::StoredGroupMessage>, crate::ConnectionError>;

        #[mockall::concretize]
        fn write_conn_get_group_message<MessageId: AsRef<[u8]>>(
            &self,
            id: MessageId,
        ) -> Result<Option<crate::group_message::StoredGroupMessage>, crate::ConnectionError>;

        #[mockall::concretize]
        fn get_group_message_by_timestamp<GroupId: AsRef<[u8]>>(
            &self,
            group_id: GroupId,
            timestamp: i64,
        ) -> Result<Option<crate::group_message::StoredGroupMessage>, crate::ConnectionError>;

        #[mockall::concretize]
        fn get_group_message_by_sequence_id<GroupId: AsRef<[u8]>>(
            &self,
            group_id: GroupId,
            sequence_id: i64,
        ) -> Result<Option<crate::group_message::StoredGroupMessage>, crate::ConnectionError>;

        fn get_sync_group_messages(
            &self,
            group_id: &[u8],
            offset: i64,
        ) -> Result<Vec<crate::group_message::StoredGroupMessage>, crate::ConnectionError>;

        #[mockall::concretize]
        fn set_delivery_status_to_published<MessageId: AsRef<[u8]>>(
            &self,
            msg_id: &MessageId,
            timestamp: u64,
            sequence_id: i64,
            message_expire_at_ns: Option<i64>
        ) -> Result<usize, crate::ConnectionError>;

        #[mockall::concretize]
        fn set_delivery_status_to_failed<MessageId: AsRef<[u8]>>(
            &self,
            msg_id: &MessageId,
        ) -> Result<usize, crate::ConnectionError>;

        fn delete_expired_messages(&self) -> Result<usize, crate::ConnectionError>;
    }

    impl<C: ConnectionExt + 'static> QueryIdentity<C> for DbQuery<C> {
        fn queue_key_package_rotation(&self) -> Result<(), StorageError>;

        fn reset_key_package_rotation_queue(&self, rotation_interval: i64) -> Result<(), StorageError>;

        fn is_identity_needs_rotation(&self) -> Result<bool, StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryIdentityCache<C> for DbQuery<C> {
        #[mockall::concretize]
        fn fetch_cached_inbox_ids<T>(
            &self,
            identifiers: &[T],
        ) -> Result<std::collections::HashMap<String, String>, StorageError>
        where
            T: std::fmt::Display,
            for<'a> &'a T: Into<crate::identity_cache::StoredIdentityKind>;

        #[mockall::concretize]
        fn cache_inbox_id<T, S>(
            &self,
            identifier: &T,
            inbox_id: S,
        ) -> Result<(), StorageError>
        where
            T: std::fmt::Display,
            S: ToString,
            for<'a> &'a T: Into<crate::identity_cache::StoredIdentityKind>;
    }

    impl<C: ConnectionExt + 'static> QueryKeyPackageHistory<C> for DbQuery<C> {
        fn store_key_package_history_entry(
            &self,
            key_package_hash_ref: Vec<u8>,
            post_quantum_public_key: Option<Vec<u8>>,
        ) -> Result<crate::key_package_history::StoredKeyPackageHistoryEntry, StorageError>;

        fn find_key_package_history_entry_by_hash_ref(
            &self,
            hash_ref: Vec<u8>,
        ) -> Result<crate::key_package_history::StoredKeyPackageHistoryEntry, StorageError>;

        fn find_key_package_history_entries_before_id(
            &self,
            id: i32,
        ) -> Result<Vec<crate::key_package_history::StoredKeyPackageHistoryEntry>, StorageError>;

        fn mark_key_package_before_id_to_be_deleted(&self, id: i32) -> Result<(), StorageError>;

        fn get_expired_key_packages(
            &self,
        ) -> Result<Vec<crate::key_package_history::StoredKeyPackageHistoryEntry>, StorageError>;

        fn delete_key_package_history_up_to_id(&self, id: i32) -> Result<(), StorageError>;

        fn delete_key_package_entry_with_id(&self, id: i32) -> Result<(), StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryKeyStoreEntry<C> for DbQuery<C> {
        fn insert_or_update_key_store_entry(
            &self,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> Result<(), StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryDeviceSyncMessages<C> for DbQuery<C> {
        fn unprocessed_sync_group_messages(
            &self,
        ) -> Result<Vec<crate::group_message::StoredGroupMessage>, StorageError>;
    }

    impl<C: ConnectionExt + 'static> QueryRefreshState<C> for DbQuery<C> {
        #[mockall::concretize]
        fn get_refresh_state<EntityId: AsRef<[u8]>>(
            &self,
            entity_id: EntityId,
            entity_kind: crate::refresh_state::EntityKind,
        ) -> Result<Option<crate::refresh_state::RefreshState>, StorageError>;

        #[mockall::concretize]
        fn get_last_cursor_for_id<Id: AsRef<[u8]>>(
            &self,
            id: Id,
            entity_kind: crate::refresh_state::EntityKind,
        ) -> Result<i64, StorageError>;

        #[mockall::concretize]
        fn update_cursor<Id: AsRef<[u8]>>(
            &self,
            entity_id: Id,
            entity_kind: crate::refresh_state::EntityKind,
            cursor: i64,
        ) -> Result<bool, StorageError>;

        #[mockall::concretize]
        fn get_remote_log_cursors(
            &self,
            conversation_ids: &[Vec<u8>],
        ) -> Result<HashMap<Vec<u8>, i64>, crate::ConnectionError>;
    }

    impl<C: ConnectionExt + 'static> QueryIdentityUpdates<C> for DbQuery<C> {
        #[mockall::concretize]
        fn get_identity_updates<InboxId: AsRef<str>>(
            &self,
            inbox_id: InboxId,
            from_sequence_id: Option<i64>,
            to_sequence_id: Option<i64>,
        ) -> Result<Vec<crate::identity_update::StoredIdentityUpdate>, crate::ConnectionError>;

        fn insert_or_ignore_identity_updates(
            &self,
            updates: &[crate::identity_update::StoredIdentityUpdate],
        ) -> Result<(), crate::ConnectionError>;

        fn get_latest_sequence_id_for_inbox(
            &self,
            inbox_id: &str,
        ) -> Result<i64, crate::ConnectionError>;

        fn get_latest_sequence_id<'a>(
            &'a self,
            inbox_ids: &'a [&'a str],
        ) -> Result<std::collections::HashMap<String, i64>, crate::ConnectionError>;
    }

    impl<C: ConnectionExt + 'static> QueryLocalCommitLog<C> for DbQuery<C> {
        fn get_group_logs(
            &self,
            group_id: &[u8],
        ) -> Result<Vec<LocalCommitLog>, crate::ConnectionError>;

        // Local commit log entries are returned sorted in ascending order of `rowid`
        // Entries with `commit_sequence_id` = 0 should not be published to the remote commit log
        fn get_group_logs_for_publishing(
            &self,
            group_id: &[u8],
            after_cursor: i64,
        ) -> Result<Vec<LocalCommitLog>, crate::ConnectionError>;

        fn get_latest_log_for_group(
            &self,
            group_id: &[u8],
        ) -> Result<Option<LocalCommitLog>, crate::ConnectionError>;

        fn get_local_commit_log_cursor(
            &self,
            group_id: &[u8],
        ) -> Result<Option<i32>, crate::ConnectionError>;
    }
}

impl<C: ConnectionExt> ConnectionExt for MockDbQuery<C> {
    type Connection = <C as ConnectionExt>::Connection;

    fn start_transaction(&self) -> Result<TransactionGuard, crate::ConnectionError> {
        panic!("start tx");
    }

    fn raw_query_read<T, F>(&self, _fun: F) -> Result<T, crate::ConnectionError>
    where
        F: FnOnce(&mut Self::Connection) -> Result<T, diesel::result::Error>,
        Self: Sized,
    {
        todo!()
    }

    fn raw_query_write<T, F>(&self, _fun: F) -> Result<T, crate::ConnectionError>
    where
        F: FnOnce(&mut Self::Connection) -> Result<T, diesel::result::Error>,
        Self: Sized,
    {
        todo!()
    }

    fn is_in_transaction(&self) -> bool {
        todo!()
    }

    fn disconnect(&self) -> Result<(), ConnectionError> {
        todo!()
    }

    fn reconnect(&self) -> Result<(), ConnectionError> {
        todo!()
    }
}

impl<C: ConnectionExt> IntoConnection for MockDbQuery<C> {
    type Connection = MockConnection;

    fn into_connection(self) -> Self::Connection {
        todo!()
    }
}
