#[test]
fn ready() {
    println!("it works!")
}



use rust_xlsxwriter::{Workbook, XlsxError};
#[test]
fn main() -> Result<(), XlsxError> {
    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    // Add a worksheet in "constant memory" mode.
    let worksheet = workbook.add_worksheet_with_constant_memory();
    worksheet.write(0, 0, "标题1")?;
    worksheet.write(0, 1, "标题2")?;



    let mut file = std::fs::File::create("worksheet.xlsx")?;
    workbook.save_to_writer(&mut file)?;

    Ok(())
}