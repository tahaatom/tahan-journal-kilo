//! سیستم مهاجرت مبتنی بر `PRAGMA user_version`.
//!
//! قرارداد مهاجرت (`docs/database/migration-policy.md`):
//!
//! - مهاجرت‌ها نسخه‌بندی‌شده، مرتب و قابل تست هستند.
//! - هر مهاجرت اتمیک است: در یک تراکنش اجرا می‌شود و فقط در صورت موفقیت
//!   `PRAGMA user_version` به‌روزرسانی می‌شود؛ پس شکست، هیچ اثری باقی نمی‌گذارد.
//! - هیچ مهاجرتی نباید داده از دست بدهد.
//! - پشتیبانی از پشتیبان‌گیری پیش از مهاجرت‌های پرریسک با قلاب
//!   [`SchemaMigrator::run_with_backup_hook`].
//!
//! نسخه اسکیما در `PRAGMA user_version` نگه‌داری می‌شود (بدون وابستگی سنگین به
//! refinery؛ ثبت دلیل در `docs/ASSUMPTIONS.md`).

use rusqlite::Connection;

use crate::error::{StorageError, StorageResult};

/// یک مهاجرت نسخه‌بندی‌شده.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// نسخه هدف این مهاجرت (بزرگ‌تر از نسخه قبلی، پیوسته از ۱).
    pub version: u32,
    /// نام توصیفی مهاجرت.
    pub name: &'static str,
    /// دستورات SQL مهاجرت (یک یا چند دستور).
    pub sql: &'static str,
    /// آیا پیش از اعمال، پشتیبان‌گیری لازم است؟
    pub requires_backup: bool,
}

/// فهرست مرتب مهاجرت‌های نسخه ۱.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "v1_schema",
    sql: include_str!("migrations/0001_v1_schema.sql"),
    requires_backup: false,
}];

/// بالاترین نسخه اسکیمای شناخته‌شده.
pub fn latest_schema_version() -> u32 {
    MIGRATIONS.iter().map(|m| m.version).max().unwrap_or(0)
}

/// گزارش اجرای مهاجرت‌ها.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    /// نسخه اسکیما پیش از اجرا.
    pub from_version: u32,
    /// نسخه اسکیما پس از اجرا.
    pub to_version: u32,
    /// نسخه مهاجرت‌هایی که در این اجرا اعمال شدند.
    pub applied: Vec<u32>,
}

impl MigrationReport {
    /// آیا در این اجرا مهاجرتی اعمال شد؟
    pub fn changed(&self) -> bool {
        !self.applied.is_empty()
    }
}

/// اجراکننده مهاجرت‌ها روی یک اتصال.
pub struct SchemaMigrator<'a> {
    connection: &'a Connection,
}

impl<'a> SchemaMigrator<'a> {
    /// ساخت اجراکننده مهاجرت برای یک اتصال.
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    /// نسخه فعلی اسکیما از `PRAGMA user_version`.
    pub fn current_version(&self) -> StorageResult<u32> {
        read_user_version(self.connection)
    }

    /// همه مهاجرت‌های معلق را بدون قلاب پشتیبان اجرا می‌کند.
    pub fn run(&self) -> StorageResult<MigrationReport> {
        self.run_with_backup_hook(|_| Ok(()))
    }

    /// همه مهاجرت‌های معلق را اجرا می‌کند و پیش از هر مهاجرت پرریسک،
    /// قلاب پشتیبان‌گیری را فرا می‌خواند.
    pub fn run_with_backup_hook<F>(&self, backup_hook: F) -> StorageResult<MigrationReport>
    where
        F: FnMut(u32) -> StorageResult<()>,
    {
        self.run_migrations_from(MIGRATIONS, backup_hook)
    }

    /// مهاجرت‌ها را از یک فهرست دلخواه اجرا می‌کند.
    ///
    /// این نقطه ورود، اجرای فهرست سفارشی مهاجرت‌ها (برای تست و ابزار) را
    /// ممکن می‌کند؛ مسیر معمول [`SchemaMigrator::run`] است.
    ///
    /// هر مهاجرت در یک تراکنش اتمیک اعمال می‌شود؛ در صورت شکست، نه تغییرات
    /// مهاجرت باقی می‌ماند و نه نسخه تغییر می‌کند.
    pub fn run_migrations_from<F>(
        &self,
        migrations: &[Migration],
        mut backup_hook: F,
    ) -> StorageResult<MigrationReport>
    where
        F: FnMut(u32) -> StorageResult<()>,
    {
        let from_version = self.current_version()?;
        let latest = migrations.iter().map(|m| m.version).max().unwrap_or(0);
        if from_version > latest {
            return Err(StorageError::MigrationVersionUnknown {
                context: Some(serde_json::json!({
                    "current": from_version,
                    "supported": latest,
                })),
                source: None,
            });
        }

        let mut applied = Vec::new();
        for migration in migrations.iter().filter(|m| m.version > from_version) {
            if migration.requires_backup {
                backup_hook(migration.version).map_err(|error| {
                    StorageError::MigrationBackupFailed {
                        context: Some(serde_json::json!({
                            "version": migration.version,
                            "cause": error.to_payload(),
                        })),
                        source: None,
                    }
                })?;
            }
            self.apply(migration)?;
            applied.push(migration.version);
        }

        Ok(MigrationReport {
            from_version,
            to_version: self.current_version()?,
            applied,
        })
    }

    /// یک مهاجرت را در یک تراکنش اتمیک اعمال می‌کند و سپس نسخه را ثبت می‌کند.
    fn apply(&self, migration: &Migration) -> StorageResult<()> {
        self.connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|source| migration_error(migration, source))?;

        if let Err(source) = self.connection.execute_batch(migration.sql) {
            let _ = self.connection.execute_batch("ROLLBACK");
            return Err(migration_error(migration, source));
        }

        // ثبت نسخه در همان تراکنش تا اتمیک بماند.
        if let Err(source) = set_user_version_in_transaction(self.connection, migration.version) {
            let _ = self.connection.execute_batch("ROLLBACK");
            return Err(migration_error(migration, source));
        }

        self.connection
            .execute_batch("COMMIT")
            .map_err(|source| migration_error(migration, source))?;
        Ok(())
    }
}

fn migration_error(migration: &Migration, source: rusqlite::Error) -> StorageError {
    StorageError::MigrationFailed {
        context: Some(serde_json::json!({
            "version": migration.version,
            "name": migration.name,
        })),
        source: Some(Box::new(source)),
    }
}

/// خواندن `PRAGMA user_version`.
pub fn read_user_version(connection: &Connection) -> StorageResult<u32> {
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "pragma": "user_version" })),
            source: Some(Box::new(source)),
        })?;
    Ok(version.max(0) as u32)
}

/// تنظیم `PRAGMA user_version` (باید داخل تراکنش فراخوانی شود؛ پارامترپذیر نیست
/// بنابراین مقدار اعتبارسنجی و به‌صورت عدد صحیح تزریق می‌شود).
fn set_user_version_in_transaction(connection: &Connection, version: u32) -> rusqlite::Result<()> {
    connection.execute_batch(&format!("PRAGMA user_version = {}", version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_ordered_and_contiguous() {
        let mut expected = 0;
        for migration in MIGRATIONS {
            expected += 1;
            assert_eq!(
                migration.version, expected,
                "migration versions must be contiguous"
            );
            assert!(!migration.name.is_empty());
            assert!(!migration.sql.trim().is_empty());
        }
        assert_eq!(latest_schema_version(), expected);
    }

    #[test]
    fn latest_version_is_one_for_v1() {
        assert_eq!(latest_schema_version(), 1);
    }
}
