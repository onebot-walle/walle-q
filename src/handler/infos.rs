use dashmap::DashMap;

use ricq::{
    structs::{GroupInfo as Rqgi, GroupMemberPermission},
    Client, RQError,
};
use walle_core::{
    resp::RespError,
    structs::{GroupInfo, UserInfo},
};

use crate::error;

#[derive(Default)]
pub struct Infos {
    pub owned_groups: DashMap<i64, GroupInfo>,
    pub admined_groups: DashMap<i64, GroupInfo>,
    pub groups: DashMap<i64, GroupInfo>,
    pub friends: DashMap<i64, UserInfo>,
}

impl Infos {
    pub(crate) async fn update_friends(&self, cli: &Client) -> Result<(), RQError> {
        self.friends.clear();
        for info in cli
            .get_friend_list()
            .await?
            .friends
            .into_iter()
            .map(|info| {
                (
                    info.uin,
                    UserInfo {
                        user_id: info.uin.to_string(),
                        nickname: info.nick,
                    },
                )
            })
        {
            self.friends.insert(info.0, info.1);
        }
        Ok(())
    }
    pub(crate) async fn update_groups(&self, cli: &Client) -> Result<(), RQError> {
        fn to_info(info: Rqgi) -> GroupInfo {
            GroupInfo {
                group_id: info.code.to_string(),
                group_name: info.name,
            }
        }
        let groups = cli.get_group_list().await?;
        let self_id = cli.uin().await;
        self.owned_groups.clear();
        self.admined_groups.clear();
        self.groups.clear();
        for info in groups {
            let roles = cli.get_group_admin_list(info.uin).await?;
            match roles.get(&self_id) {
                Some(GroupMemberPermission::Owner) => {
                    self.owned_groups.insert(info.uin, to_info(info))
                }
                Some(GroupMemberPermission::Administrator) => {
                    self.admined_groups.insert(info.uin, to_info(info))
                }
                _ => self.groups.insert(info.uin, to_info(info)),
            };
        }
        Ok(())
    }
    pub(crate) async fn update(&self, cli: &Client) -> Result<(), RQError> {
        self.update_friends(cli).await?;
        self.update_groups(cli).await
    }
    pub(crate) fn check_admin(&self, group_id: i64) -> Result<(), RespError> {
        if !(self.owned_groups.contains_key(&group_id)
            || self.admined_groups.contains_key(&group_id))
        {
            Err(error::permission_denied(""))
        } else {
            Ok(())
        }
    }
    pub(crate) fn check_owner(&self, group_id: i64) -> Result<(), RespError> {
        if !self.owned_groups.contains_key(&group_id) {
            Err(error::permission_denied(""))
        } else {
            Ok(())
        }
    }
}
