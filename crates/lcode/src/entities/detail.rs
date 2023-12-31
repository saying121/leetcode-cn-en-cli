use sea_orm::entity::prelude::*;

#[derive(Default, Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "detail")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id:      u32,
    pub content: String,
}

// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Problem,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Problem => Entity::belongs_to(super::index::Entity)
                .from(Column::Id)
                .to(super::index::Column::QuestionId)
                .on_update(ForeignKeyAction::Cascade)
                .on_delete(ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::index::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Problem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
