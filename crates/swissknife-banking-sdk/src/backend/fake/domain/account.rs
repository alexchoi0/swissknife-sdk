use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "fake_accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub external_id: String,
    pub user_id: String,
    pub institution_id: String,
    pub name: String,
    pub account_type: String,
    pub subtype: Option<String>,
    pub currency: String,
    pub balance_available: Option<f64>,
    pub balance_current: Option<f64>,
    pub balance_limit: Option<f64>,
    pub iban: Option<String>,
    pub account_number: Option<String>,
    pub routing_number: Option<String>,
    pub sort_code: Option<String>,
    pub mask: Option<String>,
    pub status: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::institution::Entity",
        from = "Column::InstitutionId",
        to = "super::institution::Column::Id"
    )]
    Institution,
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transactions,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::institution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Institution.def()
    }
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transactions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
