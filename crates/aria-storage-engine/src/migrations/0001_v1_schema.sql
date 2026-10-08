-- =============================================================================
-- مهاجرت ۰۰۰۱ — اسکیمای فیزیکی نسخه ۱ کرنل آریا (فاز ۱.۳)
-- =============================================================================
-- قوانین:
--   * همه شناسه‌ها UUID v4 هستند و به‌صورت TEXT (قالب متعارف خط‌تیره‌دار) ذخیره می‌شوند.
--   * همه زمان‌ها UTC و ISO 8601 (RFC 3339) و به‌صورت TEXT ذخیره می‌شوند.
--   * قیمت‌ها، حجم‌ها و مبالغ دفتر دامنه به‌صورت TEXT (رشته اعشاری دقیق) ذخیره
--     می‌شوند تا دقت ورودی کاربر حفظ شود؛ پارس و محاسبه در موتور دامنه (فاز ۱.۶).
--   * داده خام وارداتی (source_records) غیرقابل تغییر است.
-- =============================================================================

-- -----------------------------------------------------------------------------
-- پروفایل و حساب و نماد
-- -----------------------------------------------------------------------------
CREATE TABLE profiles (
    id           TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE trading_accounts (
    id         TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    broker     TEXT NOT NULL DEFAULT '',
    currency   TEXT NOT NULL DEFAULT 'USD',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE symbols (
    id          TEXT PRIMARY KEY,
    profile_id  TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    symbol_code TEXT NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    market      TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE (profile_id, symbol_code)
);

-- -----------------------------------------------------------------------------
-- معامله ژورنال
-- -----------------------------------------------------------------------------
CREATE TABLE journal_trades (
    id                TEXT PRIMARY KEY,
    account_id        TEXT NOT NULL REFERENCES trading_accounts(id) ON DELETE CASCADE,
    symbol_id         TEXT NOT NULL REFERENCES symbols(id) ON DELETE RESTRICT,
    direction         TEXT NOT NULL CHECK (direction IN ('buy', 'sell')),
    status            TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed', 'cancelled')),
    strategy          TEXT NOT NULL DEFAULT '',
    timeframe         TEXT NOT NULL DEFAULT '',
    session           TEXT NOT NULL DEFAULT '',
    market_condition  TEXT NOT NULL DEFAULT '',
    entry_type        TEXT NOT NULL DEFAULT 'manual' CHECK (entry_type IN ('manual', 'imported')),
    note              TEXT NOT NULL DEFAULT '',
    tags              TEXT NOT NULL DEFAULT '[]',
    emotions          TEXT NOT NULL DEFAULT '[]',
    mistakes          TEXT NOT NULL DEFAULT '[]',
    entry_time        TEXT,
    exit_time         TEXT,
    -- رزروشده برای نسخه ۲ (Position Group صریح)
    position_group_id TEXT,
    -- ریسک (نسخه ۱)
    risk_calculation_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (risk_calculation_status IN ('pending', 'ok', 'no_stop_loss', 'manual_risk', 'error')),
    risk_basis        TEXT NOT NULL DEFAULT 'stop_loss' CHECK (risk_basis IN ('stop_loss', 'manual_risk')),
    -- رزروشده برای نسخه ۳ (MAE/MFE و مسیر معامله)
    mae_price         TEXT,
    mfe_price         TEXT,
    mae_amount        TEXT,
    mfe_amount        TEXT,
    mae_r             TEXT,
    mfe_r             TEXT,
    max_drawdown_inside_trade TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

-- -----------------------------------------------------------------------------
-- لگ‌های ورود و خروج
-- -----------------------------------------------------------------------------
CREATE TABLE entry_legs (
    id          TEXT PRIMARY KEY,
    trade_id    TEXT NOT NULL REFERENCES journal_trades(id) ON DELETE CASCADE,
    entry_price TEXT NOT NULL,
    volume      TEXT NOT NULL,
    stop_loss   TEXT,
    take_profit TEXT,
    entry_time  TEXT NOT NULL,
    note        TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL
);

CREATE TABLE exit_legs (
    id         TEXT PRIMARY KEY,
    trade_id   TEXT NOT NULL REFERENCES journal_trades(id) ON DELETE CASCADE,
    exit_price TEXT NOT NULL,
    volume     TEXT NOT NULL,
    exit_time  TEXT NOT NULL,
    note       TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

-- -----------------------------------------------------------------------------
-- رکوردهای مبدا (داده خام وارداتی — غیرقابل تغییر)
-- -----------------------------------------------------------------------------
CREATE TABLE source_records (
    id               TEXT PRIMARY KEY,
    import_batch_id  TEXT NOT NULL,
    source_file_hash TEXT NOT NULL DEFAULT '',
    raw_payload      TEXT NOT NULL,
    broker           TEXT NOT NULL DEFAULT '',
    status           TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processed', 'failed', 'duplicate')),
    imported_at      TEXT NOT NULL,
    created_at       TEXT NOT NULL
);

-- -----------------------------------------------------------------------------
-- اجراها (بُرد واقعی بروکر)
-- leg_id به یکی از دو جدول entry_legs/exit_legs اشاره می‌کند؛ یکپارچگی
-- رابطه لگ در لایه دامنه (فاز ۱.۶) اعمال می‌شود، نه با کلید خارجی.
-- -----------------------------------------------------------------------------
CREATE TABLE executions (
    id                TEXT PRIMARY KEY,
    trade_id          TEXT NOT NULL REFERENCES journal_trades(id) ON DELETE CASCADE,
    leg_id            TEXT,
    leg_type          TEXT CHECK (leg_type IS NULL OR leg_type IN ('entry', 'exit')),
    assignment_status TEXT NOT NULL DEFAULT 'needs_assignment'
        CHECK (assignment_status IN ('assigned', 'needs_assignment')),
    ticket            TEXT NOT NULL DEFAULT '',
    position_id       TEXT NOT NULL DEFAULT '',
    magic             TEXT NOT NULL DEFAULT '',
    execution_time    TEXT NOT NULL,
    price             TEXT NOT NULL,
    volume            TEXT NOT NULL,
    commission        TEXT NOT NULL DEFAULT '0',
    swap              TEXT NOT NULL DEFAULT '0',
    profit            TEXT NOT NULL DEFAULT '0',
    comment           TEXT NOT NULL DEFAULT '',
    source_record_id  TEXT REFERENCES source_records(id) ON DELETE SET NULL,
    created_at        TEXT NOT NULL
);

-- -----------------------------------------------------------------------------
-- دفتر دست‌نویس‌های دستی (Override Ledger)
-- -----------------------------------------------------------------------------
CREATE TABLE manual_overrides (
    id             TEXT PRIMARY KEY,
    entity_type    TEXT NOT NULL,
    entity_id      TEXT NOT NULL,
    field_name     TEXT NOT NULL,
    previous_value TEXT,
    new_value      TEXT NOT NULL,
    reason         TEXT NOT NULL DEFAULT '',
    source         TEXT NOT NULL DEFAULT 'user',
    priority       INTEGER NOT NULL DEFAULT 0,
    reversible     INTEGER NOT NULL DEFAULT 1,
    created_at     TEXT NOT NULL,
    reverted_at    TEXT
);

-- -----------------------------------------------------------------------------
-- فیلدهای سفارشی (اسکیما)
-- -----------------------------------------------------------------------------
CREATE TABLE custom_fields (
    id               TEXT PRIMARY KEY,
    technical_key    TEXT NOT NULL UNIQUE,
    display_label    TEXT NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    storage_type     TEXT NOT NULL,
    semantic_type    TEXT NOT NULL,
    unit             TEXT NOT NULL DEFAULT '',
    default_value    TEXT,
    required         INTEGER NOT NULL DEFAULT 0,
    active           INTEGER NOT NULL DEFAULT 1,
    filterable       INTEGER NOT NULL DEFAULT 0,
    stat_enabled     INTEGER NOT NULL DEFAULT 0,
    analysis_enabled INTEGER NOT NULL DEFAULT 0,
    display_order    INTEGER NOT NULL DEFAULT 0,
    form_group       TEXT NOT NULL DEFAULT '',
    validation_rules TEXT NOT NULL DEFAULT '{}',
    schema_version   INTEGER NOT NULL DEFAULT 1,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    archived_at      TEXT
);

CREATE TABLE field_options (
    id            TEXT PRIMARY KEY,
    field_id      TEXT NOT NULL REFERENCES custom_fields(id) ON DELETE CASCADE,
    value         TEXT NOT NULL,
    label         TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    active        INTEGER NOT NULL DEFAULT 1,
    UNIQUE (field_id, value)
);

-- یک ردیف به‌ازای هر (معامله، فیلد)؛ حداکثر یک ستون تایپ‌شده مقدار دارد.
CREATE TABLE field_values (
    trade_id       TEXT NOT NULL REFERENCES journal_trades(id) ON DELETE CASCADE,
    field_id       TEXT NOT NULL REFERENCES custom_fields(id) ON DELETE CASCADE,
    text_value     TEXT,
    integer_value  INTEGER,
    decimal_value  REAL,
    boolean_value  INTEGER,
    datetime_value TEXT,
    json_value     TEXT,
    updated_at     TEXT NOT NULL,
    PRIMARY KEY (trade_id, field_id),
    CHECK (
        (text_value IS NOT NULL) + (integer_value IS NOT NULL) + (decimal_value IS NOT NULL) +
        (boolean_value IS NOT NULL) + (datetime_value IS NOT NULL) + (json_value IS NOT NULL) <= 1
    )
);

-- -----------------------------------------------------------------------------
-- پیوست‌ها (فایل بیرون از دیتابیس، متادیتا داخل دیتابیس)
-- -----------------------------------------------------------------------------
CREATE TABLE attachments (
    id             TEXT PRIMARY KEY,
    filename       TEXT NOT NULL,
    content_type   TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_bytes     INTEGER NOT NULL,
    blake3_hash    TEXT NOT NULL,
    storage_path   TEXT NOT NULL,
    thumbnail_path TEXT,
    created_at     TEXT NOT NULL
);

CREATE TABLE attachment_trade_links (
    attachment_id TEXT NOT NULL REFERENCES attachments(id) ON DELETE CASCADE,
    trade_id      TEXT NOT NULL REFERENCES journal_trades(id) ON DELETE CASCADE,
    link_type     TEXT NOT NULL DEFAULT 'other'
        CHECK (link_type IN ('before_trade', 'after_trade', 'chart', 'news', 'other')),
    created_at    TEXT NOT NULL,
    PRIMARY KEY (attachment_id, trade_id)
);

-- -----------------------------------------------------------------------------
-- حسابرسی، رویدادهای سیستم، پلاگین‌ها، تنظیمات و پیش‌نمایش‌های داشبورد
-- -----------------------------------------------------------------------------
CREATE TABLE audit_logs (
    id        TEXT PRIMARY KEY,
    operation TEXT NOT NULL,
    actor     TEXT NOT NULL,
    outcome   TEXT NOT NULL CHECK (outcome IN ('success', 'failure')),
    timestamp TEXT NOT NULL,
    details   TEXT
);

CREATE TABLE system_events (
    id               TEXT PRIMARY KEY,
    event_type       TEXT NOT NULL,
    event_version    INTEGER NOT NULL,
    source_engine    TEXT NOT NULL,
    source_component TEXT,
    correlation_id   TEXT,
    timestamp        TEXT NOT NULL,
    payload          TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE plugins (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    version      TEXT NOT NULL,
    api_version  TEXT NOT NULL,
    entrypoint   TEXT NOT NULL,
    runtime_mode TEXT NOT NULL CHECK (runtime_mode IN ('out_of_process', 'in_process_trusted')),
    status       TEXT NOT NULL DEFAULT 'installed'
        CHECK (status IN ('installed', 'enabled', 'running', 'stopped', 'crashed', 'quarantined', 'disabled')),
    capabilities TEXT NOT NULL DEFAULT '[]',
    permissions  TEXT NOT NULL DEFAULT '[]',
    trust_level  TEXT NOT NULL DEFAULT 'community'
        CHECK (trust_level IN ('official', 'certified', 'community')),
    last_error   TEXT,
    crash_count  INTEGER NOT NULL DEFAULT 0,
    installed_at TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- پیش‌نمایش‌های داشبورد: بازتولیدپذیر و منبع حقیقت نیستند.
CREATE TABLE dashboard_projections (
    summary_key TEXT PRIMARY KEY,
    payload     TEXT NOT NULL DEFAULT '{}',
    computed_at TEXT NOT NULL
);

-- -----------------------------------------------------------------------------
-- ایندکس‌ها (کارایی فیلتر، لیست و آمار)
-- -----------------------------------------------------------------------------
CREATE INDEX idx_trades_account_id ON journal_trades(account_id);
CREATE INDEX idx_trades_symbol_id ON journal_trades(symbol_id);
CREATE INDEX idx_trades_status ON journal_trades(status);
CREATE INDEX idx_trades_entry_time ON journal_trades(entry_time);
CREATE INDEX idx_entry_legs_trade_id ON entry_legs(trade_id);
CREATE INDEX idx_exit_legs_trade_id ON exit_legs(trade_id);
CREATE INDEX idx_executions_trade_id ON executions(trade_id);
CREATE INDEX idx_executions_leg_id ON executions(leg_id);
CREATE INDEX idx_executions_assignment ON executions(assignment_status);
CREATE INDEX idx_field_values_field_id ON field_values(field_id);
CREATE INDEX idx_field_values_integer ON field_values(field_id, integer_value);
CREATE INDEX idx_field_values_decimal ON field_values(field_id, decimal_value);
CREATE INDEX idx_field_values_text ON field_values(field_id, text_value);
CREATE INDEX idx_field_values_datetime ON field_values(field_id, datetime_value);
CREATE INDEX idx_attachments_hash ON attachments(blake3_hash);
CREATE INDEX idx_audit_logs_time ON audit_logs(timestamp);
CREATE INDEX idx_source_records_batch ON source_records(import_batch_id);
CREATE INDEX idx_source_records_status ON source_records(status);