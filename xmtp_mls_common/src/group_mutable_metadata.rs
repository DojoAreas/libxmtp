use openmls::{
    extensions::{Extension, Extensions, UnknownExtension},
    group::MlsGroup as OpenMlsGroup,
};
use prost::Message;
use std::{collections::HashMap, fmt};
use thiserror::Error;
use xmtp_proto::xmtp::mls::message_contents::{
    GroupMutableMetadataV1 as GroupMutableMetadataProto, Inboxes as InboxesProto,
};

use super::group::{DMMetadataOptions, GroupMetadataOptions};
use crate::config::{
    DEFAULT_GROUP_DESCRIPTION, DEFAULT_GROUP_IMAGE_URL_SQUARE, DEFAULT_GROUP_NAME,
    MUTABLE_METADATA_EXTENSION_ID,
};

/// Errors that can occur when working with GroupMutableMetadata.
#[derive(Debug, Error)]
pub enum GroupMutableMetadataError {
    #[error("serialization: {0}")]
    Serialization(#[from] prost::EncodeError),
    #[error("deserialization: {0}")]
    Deserialization(#[from] prost::DecodeError),
    #[error("missing extension")]
    MissingExtension,
    #[error("mutable extension updates only")]
    NonMutableExtensionUpdate,
    #[error("only one change per update permitted")]
    TooManyUpdates,
    #[error("no changes in this update")]
    NoUpdates,
    #[error("missing metadata field")]
    MissingMetadataField,
}

/// Represents the "updateable" metadata fields for a group.
/// Members ability to update metadata is gated by the group permissions.
///
/// New fields should be added to the `supported_fields` function for Metadata Update Support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataField {
    GroupName,
    Description,
    GroupImageUrlSquare,
    MessageDisappearFromNS,
    MessageDisappearInNS,
    MinimumSupportedProtocolVersion,
}

impl MetadataField {
    /// String representations used as keys in the GroupMutableMetadata attributes map.
    pub const fn as_str(&self) -> &'static str {
        match self {
            MetadataField::GroupName => "group_name",
            MetadataField::Description => "description",
            MetadataField::GroupImageUrlSquare => "group_image_url_square",
            MetadataField::MessageDisappearFromNS => "message_disappear_from_ns",
            MetadataField::MessageDisappearInNS => "message_disappear_in_ns",
            MetadataField::MinimumSupportedProtocolVersion => "minimum_supported_protocol_version",
        }
    }
}

impl fmt::Display for MetadataField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Settings for disappearing messages in a conversation.
///
/// # Fields
///
/// * `from_ns` - The timestamp (in nanoseconds) from when messages should be tracked for deletion.
/// * `in_ns` - The duration (in nanoseconds) after which tracked messages will be deleted.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct MessageDisappearingSettings {
    pub from_ns: i64,
    pub in_ns: i64,
}

impl MessageDisappearingSettings {
    pub fn new(from_ns: i64, in_ns: i64) -> Self {
        Self { from_ns, in_ns }
    }
}

/// Represents the mutable metadata for a group.
///
/// This struct is stored as an MLS Unknown Group Context Extension.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupMutableMetadata {
    /// Map to store various metadata attributes (e.g., group name, description).
    /// Allows libxmtp to receive attributes from updated versions not yet captured in MetadataField.
    pub attributes: HashMap<String, String>,
    /// List of admin inbox IDs for this group.
    /// See [GroupMutablePermissions](crate::groups::GroupMutablePermissions) for more details on admin permissions.
    pub admin_list: Vec<String>,
    /// List of super admin inbox IDs for this group.
    /// See [GroupMutablePermissions](crate::groups::GroupMutablePermissions) for more details on super admin permissions.
    pub super_admin_list: Vec<String>,
}

impl GroupMutableMetadata {
    /// Creates a new GroupMutableMetadata instance.
    pub fn new(
        attributes: HashMap<String, String>,
        admin_list: Vec<String>,
        super_admin_list: Vec<String>,
    ) -> Self {
        Self {
            attributes,
            admin_list,
            super_admin_list,
        }
    }

    /// Creates a new GroupMutableMetadata instance with default values.
    /// The creator is automatically added as a super admin.
    /// See [GroupMutablePermissions](crate::groups::GroupMutablePermissions) for more details on super admin permissions.
    pub fn new_default(creator_inbox_id: String, opts: GroupMetadataOptions) -> Self {
        let mut attributes = HashMap::new();
        attributes.insert(
            MetadataField::GroupName.to_string(),
            opts.name.unwrap_or_else(|| DEFAULT_GROUP_NAME.to_string()),
        );
        attributes.insert(
            MetadataField::Description.to_string(),
            opts.description
                .unwrap_or_else(|| DEFAULT_GROUP_DESCRIPTION.to_string()),
        );
        attributes.insert(
            MetadataField::GroupImageUrlSquare.to_string(),
            opts.image_url_square
                .unwrap_or_else(|| DEFAULT_GROUP_IMAGE_URL_SQUARE.to_string()),
        );

        if let Some(message_disappearing_settings) = opts.message_disappearing_settings {
            attributes.insert(
                MetadataField::MessageDisappearFromNS.to_string(),
                message_disappearing_settings.from_ns.to_string(),
            );
            attributes.insert(
                MetadataField::MessageDisappearInNS.to_string(),
                message_disappearing_settings.in_ns.to_string(),
            );
        }

        let admin_list = vec![];
        let super_admin_list = vec![creator_inbox_id.clone()];
        Self {
            attributes,
            admin_list,
            super_admin_list,
        }
    }

    // Admin / super admin is not needed for a DM
    pub fn new_dm_default(
        _creator_inbox_id: String,
        _dm_target_inbox_id: &str,
        opts: DMMetadataOptions,
    ) -> Self {
        let mut attributes = HashMap::new();
        // TODO: would it be helpful to incorporate the dm inbox ids in the name or description?
        attributes.insert(
            MetadataField::GroupName.to_string(),
            DEFAULT_GROUP_NAME.to_string(),
        );
        attributes.insert(
            MetadataField::Description.to_string(),
            DEFAULT_GROUP_DESCRIPTION.to_string(),
        );
        attributes.insert(
            MetadataField::GroupImageUrlSquare.to_string(),
            DEFAULT_GROUP_IMAGE_URL_SQUARE.to_string(),
        );
        if let Some(message_disappearing_settings) = opts.message_disappearing_settings {
            attributes.insert(
                MetadataField::MessageDisappearFromNS.to_string(),
                message_disappearing_settings.from_ns.to_string(),
            );
            attributes.insert(
                MetadataField::MessageDisappearInNS.to_string(),
                message_disappearing_settings.in_ns.to_string(),
            );
        }

        let admin_list = vec![];
        let super_admin_list = vec![];
        Self {
            attributes,
            admin_list,
            super_admin_list,
        }
    }

    /// Returns a vector of supported metadata fields.
    ///
    /// These fields will receive default permission policies for new groups.
    pub fn supported_fields() -> Vec<MetadataField> {
        vec![
            MetadataField::GroupName,
            MetadataField::Description,
            MetadataField::GroupImageUrlSquare,
            MetadataField::MessageDisappearFromNS,
            MetadataField::MessageDisappearInNS,
            MetadataField::MinimumSupportedProtocolVersion,
        ]
    }

    /// Checks if the given inbox ID is an admin.
    pub fn is_admin(&self, inbox_id: &String) -> bool {
        self.admin_list.contains(inbox_id)
    }

    /// Checks if the given inbox ID is a super admin.
    pub fn is_super_admin(&self, inbox_id: &String) -> bool {
        self.super_admin_list.contains(inbox_id)
    }
}

impl TryFrom<GroupMutableMetadata> for Vec<u8> {
    type Error = GroupMutableMetadataError;

    /// Converts GroupMutableMetadata to a byte vector for storage as an MLS Unknown Group Context Extension.
    fn try_from(value: GroupMutableMetadata) -> Result<Self, Self::Error> {
        let mut buf = Vec::new();
        let proto_val = GroupMutableMetadataProto {
            attributes: value.attributes.clone(),
            admin_list: Some(InboxesProto {
                inbox_ids: value.admin_list,
            }),
            super_admin_list: Some(InboxesProto {
                inbox_ids: value.super_admin_list,
            }),
            commit_log_signer: None,
        };
        proto_val.encode(&mut buf)?;

        Ok(buf)
    }
}

impl TryFrom<&Vec<u8>> for GroupMutableMetadata {
    type Error = GroupMutableMetadataError;

    /// Converts a byte vector to GroupMutableMetadata.
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let proto_val = GroupMutableMetadataProto::decode(value.as_slice())?;
        Self::try_from(proto_val)
    }
}

impl TryFrom<GroupMutableMetadataProto> for GroupMutableMetadata {
    type Error = GroupMutableMetadataError;

    /// Converts a GroupMutableMetadataProto to GroupMutableMetadata.
    fn try_from(value: GroupMutableMetadataProto) -> Result<Self, Self::Error> {
        let admin_list = value
            .admin_list
            .ok_or(GroupMutableMetadataError::MissingMetadataField)?
            .inbox_ids;

        let super_admin_list = value
            .super_admin_list
            .ok_or(GroupMutableMetadataError::MissingMetadataField)?
            .inbox_ids;

        Ok(Self::new(
            value.attributes.clone(),
            admin_list,
            super_admin_list,
        ))
    }
}

impl TryFrom<&Extensions> for GroupMutableMetadata {
    type Error = GroupMutableMetadataError;

    /// Attempts to extract GroupMutableMetadata from MLS Extensions.
    fn try_from(value: &Extensions) -> Result<Self, Self::Error> {
        match find_mutable_metadata_extension(value) {
            Some(metadata) => GroupMutableMetadata::try_from(metadata),
            None => Err(GroupMutableMetadataError::MissingExtension),
        }
    }
}

impl TryFrom<&OpenMlsGroup> for GroupMutableMetadata {
    type Error = GroupMutableMetadataError;

    /// Attempts to extract GroupMutableMetadata from an OpenMlsGroup.
    fn try_from(group: &OpenMlsGroup) -> Result<Self, Self::Error> {
        let extensions = group.extensions();
        extensions.try_into()
    }
}

/// Finds the mutable metadata extension in the given MLS Extensions.
///
/// This function searches for an Unknown Extension with the
/// [MUTABLE_METADATA_EXTENSION_ID](crate::configuration::MUTABLE_METADATA_EXTENSION_ID).
pub fn find_mutable_metadata_extension(extensions: &Extensions) -> Option<&Vec<u8>> {
    extensions.iter().find_map(|extension| {
        if let Extension::Unknown(MUTABLE_METADATA_EXTENSION_ID, UnknownExtension(metadata)) =
            extension
        {
            return Some(metadata);
        }
        None
    })
}

pub fn extract_group_mutable_metadata(
    group: &OpenMlsGroup,
) -> Result<GroupMutableMetadata, GroupMutableMetadataError> {
    find_mutable_metadata_extension(group.extensions())
        .ok_or(GroupMutableMetadataError::MissingExtension)?
        .try_into()
}
