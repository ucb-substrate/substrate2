use scir::schema::Schema;
use scir::{CellId, Library};
use std::io::Write;
use std::path::Path;

pub fn export_verilog_shells<S: Schema, W: Write>(
    lib: &Library<S>,
    cells: &[CellId],
    out: &mut W,
) -> std::io::Result<()> {
    for cell in cells {
        let cell = lib.cell(*cell);

        writeln!(out, "module {} (", cell.name())?;

        writeln!(
            out,
            "{}",
            cell.ports()
                .map(|port| {
                    // TODO: Handle bus signals.
                    let signal = cell.signal(port.signal());
                    format!("inout {}", &signal.name)
                })
                .collect::<Vec<_>>()
                .join(",\n")
        )?;
        writeln!(out, ");")?;
        writeln!(out, "endmodule")?;
    }
    Ok(())
}

pub fn export_verilog_shells_by_name<S: Schema, N: AsRef<str>, W: Write>(
    lib: &Library<S>,
    cells: &[N],
    out: &mut W,
) -> std::io::Result<()> {
    let ids = cells
        .iter()
        .map(|cell| lib.cell_id_named(cell.as_ref()))
        .collect::<Vec<_>>();
    export_verilog_shells(lib, &ids, out)
}

pub fn export_verilog_shells_to_file<S: Schema, P: AsRef<Path>>(
    lib: &Library<S>,
    cells: &[CellId],
    path: P,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::File::create(path)?;
    export_verilog_shells(lib, cells, &mut f)
}

pub fn export_verilog_shells_by_name_to_file<S: Schema, N: AsRef<str>, P: AsRef<Path>>(
    lib: &Library<S>,
    cells: &[N],
    path: P,
) -> std::io::Result<()> {
    let ids = cells
        .iter()
        .map(|cell| lib.cell_id_named(cell.as_ref()))
        .collect::<Vec<_>>();
    crate::export_verilog_shells_to_file(lib, &ids, path)
}
