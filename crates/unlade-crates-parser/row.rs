use crate::columns::Columns;
use crate::date_time;
use crate::error::Error;
use jiff::Timestamp;
use std::num::ParseIntError;
use unlade_core::CrateId;
use unlade_parser::Fields;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: CrateId,
    pub name: String,
    pub updated_at: Timestamp,
}

pub fn read_row(fields: &Fields<'_, Error>, columns: &Columns) -> Result<Row, Error> {
    Ok(Row {
        id: read_crate_id(fields, columns.id())?,
        name: fields.text(columns.name())?.to_owned(),
        updated_at: read_update_date(fields, columns.updated_at())?,
    })
}

fn read_crate_id(
    fields: &Fields<'_, Error>,
    column: unlade_parser::Column,
) -> Result<CrateId, Error> {
    fields
        .text(column)?
        .parse()
        .map(CrateId::new)
        .map_err(|source: ParseIntError| Error::InvalidCrateId {
            path: fields.path().to_path_buf(),
            line: fields.line(),
            source,
        })
}

fn read_update_date(
    fields: &Fields<'_, Error>,
    column: unlade_parser::Column,
) -> Result<Timestamp, Error> {
    let text = fields.text(column)?;

    date_time::parse(text).map_err(|source| Error::InvalidUpdateDate {
        path: fields.path().to_path_buf(),
        line: fields.line(),
        source,
    })
}
