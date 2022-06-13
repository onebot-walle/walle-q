use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum WQMEDetail {
    Private {
        /// just for Deserialize
        sub_type: String,
        user_name: String,
    },
    Group {
        /// just for Deserialize
        sub_type: String,
        group_id: String,
        group_name: String,
        user_name: String,
    },
    GroupTemp {
        /// just for Deserialize
        sub_type: String,
        group_id: String,
        user_name: String,
    },
    // Channel {
    //     /// just for Deserialize
    //     sub_type: String,
    //     guild_id: String,
    //     channel_id: String,
    //     #[serde(flatten)]
    //     extra: ExtendedMap,
    // },
}
