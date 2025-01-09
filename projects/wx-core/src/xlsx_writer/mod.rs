use rust_xlsxwriter::{IntoExcelData, Workbook, Worksheet, XlsxError};
use std::{
    fmt::{Debug, Formatter},
    ops::AddAssign,
    path::Path,
};

/// Excel 导出
pub struct XlsxWriter {
    pub(crate) db: Workbook,
    pub(crate) table: Worksheet,
    current_line: u32,
}

impl Debug for XlsxWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XlsxWriter").finish()
    }
}

impl Default for XlsxWriter {
    fn default() -> Self {
        let mut wb = Workbook::new();
        let ws = wb.new_worksheet_with_constant_memory();
        Self { db: wb, table: ws, current_line: 0 }
    }
}
impl XlsxWriter {
    /// 写入标题
    pub fn write_title(&mut self, index: u16, data: impl IntoExcelData, width: f64) -> Result<(), XlsxError> {
        self.table.set_column_width(index, width)?;
        self.table.write(0, index, data)?;
        Ok(())
    }
    /// 写入数据
    pub fn write_data(&mut self, index: u16, data: impl IntoExcelData) -> Result<(), XlsxError> {
        self.table.write(self.current_line, index, data)?;
        Ok(())
    }
    /// 保存
    pub fn save(&mut self, path: &Path) -> Result<(), XlsxError> {
        let mut file = std::fs::File::create(path)?;
        self.db.save_to_writer(&mut file)
    }
    /// 下一行
    pub fn next_line(&mut self) {
        self.current_line.add_assign(1)
    }
}
