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
    current_column: u16,
}

impl Debug for XlsxWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XlsxWriter")
            .field("line", &self.current_line)
            .field("column", &self.current_column)
            .finish()
    }
}

impl Default for XlsxWriter {
    fn default() -> Self {
        let mut wb = Workbook::new();
        let ws = wb.new_worksheet_with_constant_memory();
        Self { db: wb, table: ws, current_line: 0, current_column: 0 }
    }
}
impl XlsxWriter {
    /// 写入标题
    pub fn write_title(&mut self, data: impl IntoExcelData, width: f64) -> Result<(), XlsxError> {
        self.table.set_column_width(self.current_column, width)?;
        self.table.write(0, self.current_column, data)?;
        self.current_column.add_assign(1);
        Ok(())
    }
    /// 写入数据
    pub fn write_data(&mut self, data: impl IntoExcelData) -> Result<(), XlsxError> {
        self.table.write(self.current_line, self.current_column, data)?;
        self.current_column.add_assign(1);
        Ok(())
    }
    /// 保存
    pub fn save(self, path: &Path) -> Result<(), XlsxError> {
        let Self { mut db, table, .. } = self;
        db.push_worksheet(table);
        let mut file = std::fs::File::create(path)?;
        db.save_to_writer(&mut file)
    }
    /// 下一行
    pub fn next_line(&mut self) {
        self.current_line.add_assign(1);
        self.current_column = 0;
    }
}
