use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

#[derive(Iden, EnumIter)]
enum AlertStatus {
    Table,
    #[iden = "resolved"]
    Resolved,
    #[iden = "firing"]
    Firing,
}

#[derive(DeriveIden)]
enum Groups {
    Table,
    Id,
    Timestamp,
    GroupKey,
    Receiver,
    Status,
    ExternalUrl,
}

#[derive(DeriveIden)]
enum Alerts {
    Table,
    Id,
    GroupId,
    GroupKey,
    Status,
    StartsAt,
    EndsAt,
    GeneratorUrl,
    Fingerprint,
}

#[derive(DeriveIden)]
enum Labels {
    Table,
    Id,
    Name,
    Value,
}

#[derive(DeriveIden)]
enum Annotations {
    Table,
    Id,
    Name,
    Value,
}

#[derive(DeriveIden)]
enum CommonLabels {
    Table,
    Id,
    Name,
    Value,
}

#[derive(DeriveIden)]
enum CommonAnnotations {
    Table,
    Id,
    Name,
    Value,
}

#[derive(DeriveIden)]
enum GroupsLabels {
    Table,
    GroupId,
    LabelId,
}

#[derive(DeriveIden)]
enum GroupsCommonLabels {
    Table,
    GroupId,
    CommonLabelId,
}

#[derive(DeriveIden)]
enum GroupsCommonAnnotations {
    Table,
    GroupId,
    CommonAnnotationId,
}

#[derive(DeriveIden)]
enum AlertsLabels {
    Table,
    AlertId,
    LabelId,
}

#[derive(DeriveIden)]
enum AlertsAnnotations {
    Table,
    AlertId,
    AnnotationId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AlertStatus::Table)
                    .values(AlertStatus::iter().skip(1))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Groups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Groups::Timestamp)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Groups::GroupKey)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Groups::Receiver).string().not_null())
                    .col(
                        ColumnDef::new(Groups::Status)
                            .enumeration(
                                AlertStatus::Table,
                                [AlertStatus::Resolved, AlertStatus::Firing],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Groups::ExternalUrl).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alerts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alerts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Alerts::GroupId).integer().not_null())
                    .col(ColumnDef::new(Alerts::GroupKey).string().not_null())
                    .col(
                        ColumnDef::new(Alerts::Status)
                            .enumeration(
                                AlertStatus::Table,
                                [AlertStatus::Resolved, AlertStatus::Firing],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alerts::StartsAt).timestamp().not_null())
                    .col(ColumnDef::new(Alerts::EndsAt).timestamp())
                    .col(ColumnDef::new(Alerts::GeneratorUrl).string().not_null())
                    .col(ColumnDef::new(Alerts::Fingerprint).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(Alerts::GroupKey)
                            .col(Alerts::Fingerprint),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(Alerts::Table)
                            .from_col(Alerts::GroupKey)
                            .to_tbl(Groups::Table)
                            .to_col(Groups::GroupKey),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(Alerts::Table)
                            .from_col(Alerts::GroupId)
                            .to_tbl(Groups::Table)
                            .to_col(Groups::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Labels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Labels::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Labels::Name).string().not_null())
                    .col(ColumnDef::new(Labels::Value).string().not_null())
                    .primary_key(Index::create().col(Labels::Name).col(Labels::Value))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Annotations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Annotations::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Annotations::Name).string().not_null())
                    .col(ColumnDef::new(Annotations::Value).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(Annotations::Name)
                            .col(Annotations::Value),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CommonLabels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CommonLabels::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(CommonLabels::Name).string().not_null())
                    .col(ColumnDef::new(CommonLabels::Value).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(CommonLabels::Name)
                            .col(CommonLabels::Value),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CommonAnnotations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CommonAnnotations::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(CommonAnnotations::Name).string().not_null())
                    .col(ColumnDef::new(CommonAnnotations::Value).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(CommonAnnotations::Name)
                            .col(CommonAnnotations::Value),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GroupsLabels::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GroupsLabels::GroupId).integer().not_null())
                    .col(ColumnDef::new(GroupsLabels::LabelId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(GroupsLabels::GroupId)
                            .col(GroupsLabels::LabelId),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(GroupsLabels::Table)
                            .from_col(GroupsLabels::GroupId)
                            .to_tbl(Groups::Table)
                            .to_col(Groups::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(GroupsLabels::Table)
                            .from_col(GroupsLabels::LabelId)
                            .to_tbl(Labels::Table)
                            .to_col(Labels::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GroupsCommonLabels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupsCommonLabels::GroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsCommonLabels::CommonLabelId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(GroupsCommonLabels::GroupId)
                            .col(GroupsCommonLabels::CommonLabelId),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(GroupsCommonLabels::Table)
                            .from_col(GroupsCommonLabels::CommonLabelId)
                            .to_tbl(CommonLabels::Table)
                            .to_col(CommonLabels::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(GroupsCommonLabels::Table)
                            .from_col(GroupsCommonLabels::GroupId)
                            .to_tbl(Groups::Table)
                            .to_col(Groups::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GroupsCommonAnnotations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupsCommonAnnotations::GroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsCommonAnnotations::CommonAnnotationId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(GroupsCommonAnnotations::GroupId)
                            .col(GroupsCommonAnnotations::CommonAnnotationId),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(GroupsCommonAnnotations::Table)
                            .from_col(GroupsCommonAnnotations::CommonAnnotationId)
                            .to_tbl(CommonAnnotations::Table)
                            .to_col(CommonAnnotations::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(GroupsCommonAnnotations::Table)
                            .from_col(GroupsCommonAnnotations::GroupId)
                            .to_tbl(Groups::Table)
                            .to_col(Groups::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AlertsLabels::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AlertsLabels::AlertId).integer().not_null())
                    .col(ColumnDef::new(AlertsLabels::LabelId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(AlertsLabels::AlertId)
                            .col(AlertsLabels::LabelId),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(AlertsLabels::Table)
                            .from_col(AlertsLabels::AlertId)
                            .to_tbl(Alerts::Table)
                            .to_col(Alerts::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(AlertsLabels::Table)
                            .from_col(AlertsLabels::LabelId)
                            .to_tbl(Labels::Table)
                            .to_col(Labels::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AlertsAnnotations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AlertsAnnotations::AlertId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AlertsAnnotations::AnnotationId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(AlertsAnnotations::AlertId)
                            .col(AlertsAnnotations::AnnotationId),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(AlertsAnnotations::Table)
                            .from_col(AlertsAnnotations::AlertId)
                            .to_tbl(Alerts::Table)
                            .to_col(Alerts::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(AlertsAnnotations::Table)
                            .from_col(AlertsAnnotations::AnnotationId)
                            .to_tbl(Annotations::Table)
                            .to_col(Annotations::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AlertsAnnotations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AlertsLabels::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(GroupsCommonAnnotations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(GroupsCommonLabels::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GroupsLabels::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(CommonAnnotations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(CommonLabels::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Annotations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Labels::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Alerts::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Groups::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AlertStatus::Table).to_owned())
            .await?;

        Ok(())
    }
}
