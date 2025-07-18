pub use super::schema_gen::*;

diesel::table! {
    events (created_at_ns) {
        created_at_ns -> BigInt,
        group_id -> Nullable<Binary>,
        event -> Text,
        details -> Jsonb,
        level -> Integer,
        icon -> Nullable<Text>
    }
}

diesel::table! {
  conversation_list (id) {
    id -> Binary,
    created_at_ns -> BigInt,
    membership_state -> Integer,
    installations_last_checked -> BigInt,
    added_by_inbox_id -> Text,
    welcome_id -> Nullable<BigInt>,
    dm_id -> Nullable<Text>,
    rotated_at_ns -> BigInt,
    conversation_type -> Integer,
    message_id -> Nullable<Binary>,
    decrypted_message_bytes -> Nullable<Binary>,
    sent_at_ns -> Nullable<BigInt>,
    message_kind -> Nullable<Integer>,
    sender_installation_id -> Nullable<Binary>,
    sender_inbox_id -> Nullable<Text>,
    delivery_status -> Nullable<Integer>,
    content_type -> Nullable<Integer>,
    version_major -> Nullable<Integer>,
    version_minor -> Nullable<Integer>,
    authority_id -> Nullable<Text>
  }
}

diesel::allow_tables_to_appear_in_same_query!(consent_records, conversation_list);
