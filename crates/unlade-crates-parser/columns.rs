use crate::error::Error;
use std::io::Read;
use unlade_parser::Column;
use unlade_parser::CsvParser;

pub const ID: &str = "id";
pub const NAME: &str = "name";
pub const UPDATED_AT: &str = "updated_at";

/// Where each column the parser reads sits in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Columns {
    id: Column,
    name: Column,
    updated_at: Column,
}

impl Columns {
    pub fn locate(parser: &CsvParser<impl Read, Error>) -> Result<Self, Error> {
        Ok(Self {
            id: parser.column(ID)?,
            name: parser.column(NAME)?,
            updated_at: parser.column(UPDATED_AT)?,
        })
    }

    pub fn id(&self) -> Column {
        self.id
    }

    pub fn name(&self) -> Column {
        self.name
    }

    pub fn updated_at(&self) -> Column {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn locate(headers: &[&str]) -> Result<Columns, Error> {
        let text = format!("{}\n", headers.join(","));
        let parser = CsvParser::new(text.as_bytes(), Path::new("crates.csv"))?;
        Columns::locate(&parser)
    }

    fn missing_column(headers: &[&str]) -> Option<&'static str> {
        match locate(headers) {
            Err(Error::MissingColumn { column, .. }) => Some(column),
            _ => None,
        }
    }

    #[test]
    fn columns_are_found_by_name() {
        let columns = locate(&["id", "name", "updated_at"]).unwrap();
        assert_eq!(columns.id().index(), 0);
        assert_eq!(columns.name().index(), 1);
        assert_eq!(columns.updated_at().index(), 2);
    }

    #[test]
    fn column_order_does_not_matter() {
        let columns = locate(&["updated_at", "id", "name"]).unwrap();
        assert_eq!(columns.id().index(), 1);
        assert_eq!(columns.name().index(), 2);
        assert_eq!(columns.updated_at().index(), 0);
    }

    #[test]
    fn unrelated_columns_are_skipped() {
        let columns = locate(&["description", "id", "readme", "name", "updated_at"]).unwrap();
        assert_eq!(columns.id().index(), 1);
        assert_eq!(columns.name().index(), 3);
    }

    #[test]
    fn an_absent_column_is_reported_by_name() {
        assert_eq!(missing_column(&["id", "name"]), Some("updated_at"));
        assert_eq!(missing_column(&["name", "updated_at"]), Some("id"));
        assert_eq!(missing_column(&[]), Some("id"));
    }
}
