//! Palette tokens and semantic color definitions.

pub struct Palette;

impl Palette {
  // red-600
  pub const AMBER: u32 = 0xd97706;
  // amber-600
  pub const AMBER_HOVER: u32 = 0xb45309;
  // slate-50 (soft background)
  pub const BG_CARD: u32 = 0xffffff;
  // pure white (elevated cards)
  pub const BG_HELP: u32 = 0xf1f5f9;
  // slate-100 (help container)
  pub const BORDER_CARD: u32 = 0xe2e8f0;
  // slate-200 (card border)
  pub const BORDER_CONTROL: u32 = 0xcbd5e1;
  // slate-500 (hints & empty states)
  pub const BTN_SECONDARY: u32 = 0xe2e8f0;
  // slate-200
  pub const BTN_SECONDARY_HOVER: u32 = 0xcbd5e1;
  pub const CANVAS: u32 = 0xf8fafc;
  pub const ERROR_BG: u32 = 0xfee2e2;
  // red-50
  pub const ERROR_BORDER: u32 = 0xfca5a5;
  // red-300
  pub const ERROR_TEXT: u32 = 0xb91c1c;
  // blue-700
  pub const GREEN: u32 = 0x16a34a;
  // green-600
  pub const GREEN_HOVER: u32 = 0x15803d;
  // slate-300
  pub const PRIMARY: u32 = 0x2563eb;
  // blue-600
  pub const PRIMARY_HOVER: u32 = 0x1d4ed8;
  // amber-700
  pub const RECORD_IDLE: u32 = 0x475569;
  // slate-600
  pub const RECORD_IDLE_HOVER: u32 = 0x334155;
  // green-700
  pub const RED: u32 = 0xef4444;
  // red-500
  pub const RED_HOVER: u32 = 0xdc2626;
  // slate-700 (body & instructions)
  pub const TEXT_MUTED: u32 = 0x64748b;
  // slate-300 (controls border)
  pub const TEXT_PRIMARY: u32 = 0x0f172a;
  // slate-900 (headings & labels)
  pub const TEXT_SECONDARY: u32 = 0x334155;
  // slate-700
  pub const WHITE: u32 = 0xffffff; // red-700
}
