//! The terminal and the account it holds.

wire_enum! {
    /// `ACCOUNT_TRADE_MODE`: what kind of account this is.
    AccountTradeMode: i32 {
        Demo = 0, "DEMO";
        Contest = 1, "CONTEST";
        /// Real money.
        Real = 2, "REAL";
    }
}

/// The logged-in trading account, as `account_info` reports it. Money
/// figures are in the account currency.
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub login: i64,
    pub trade_mode: AccountTradeMode,
    pub leverage: i32,
    pub limit_orders: i32,
    pub margin_so_mode: i32,
    pub trade_allowed: bool,
    pub trade_expert: bool,
    pub margin_mode: i32,
    pub currency_digits: i32,
    pub fifo_close: bool,
    pub balance: f64,
    pub credit: f64,
    pub profit: f64,
    pub equity: f64,
    pub margin: f64,
    pub margin_free: f64,
    pub margin_level: f64,
    pub margin_so_call: f64,
    pub margin_so_so: f64,
    pub margin_initial: f64,
    pub margin_maintenance: f64,
    pub assets: f64,
    pub liabilities: f64,
    pub commission_blocked: f64,
    pub name: String,
    pub server: String,
    pub currency: String,
    pub company: String,
}

/// `version()`: IPC API version, terminal build, release date as formatted
/// by the terminal (`31 Jul 2026`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminalVersion {
    pub api: u32,
    pub build: u32,
    pub released: String,
}

/// The terminal program itself: what it is connected to, what it permits,
/// and where it keeps its files.
#[derive(Debug, Clone, Default)]
pub struct TerminalInfo {
    pub build: u16,
    pub community_account: bool,
    pub community_connection: bool,
    pub notifications_enabled: bool,
    pub mqid: bool,
    /// Whether the terminal has a live broker session. The only reliable
    /// verdict on a login.
    pub connected: bool,
    pub dlls_allowed: bool,
    /// AutoTrading. Off, and the server refuses every order with 10027.
    pub trade_allowed: bool,
    /// Not trustworthy: reads the same whether the API is disabled or not.
    pub tradeapi_disabled: bool,
    pub email_enabled: bool,
    pub ftp_enabled: bool,
    pub maxbars: u32,
    pub codepage: u16,
    pub ping_last: u32,
    pub community_balance: f64,
    pub retransmission: f64,
    pub company: String,
    pub name: String,
    pub language: String,
    pub path: String,
    pub data_path: String,
    pub commondata_path: String,
}
