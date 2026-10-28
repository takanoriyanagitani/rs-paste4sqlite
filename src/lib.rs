use std::fs::File;
use std::io;
use std::path::Path;

use io::BufRead;
use io::Read;

use rusqlite::Connection;
use rusqlite::OpenFlags;

use rusqlite::Statement;
use rusqlite::Transaction;

use rusqlite::Row;
use rusqlite::Rows;

use rusqlite::types::ToSql;
use rusqlite::types::ValueRef;

pub fn paste<I, T>(mut i: Statement, o: &mut Statement, mut values: I) -> Result<usize, io::Error>
where
    I: Iterator<Item = Result<T, io::Error>>,
    T: ToSql,
{
    let colcnt: usize = i.column_count();

    let mut rows: Rows = i.query(()).map_err(io::Error::other)?;
    let mut tot: usize = 0;
    loop {
        let orow: Option<&Row> = rows.next().map_err(io::Error::other)?;
        let Some(row) = orow else {
            drop(rows);
            i.finalize().map_err(io::Error::other)?;
            return Ok(tot);
        };

        let orval: Option<Result<T, _>> = values.next();
        let Some(rval) = orval else {
            drop(rows);
            i.finalize().map_err(io::Error::other)?;
            return Ok(tot);
        };
        let val: T = rval?;

        o.clear_bindings();

        for i in 0..colcnt {
            let vr: ValueRef = row.get_ref(i).map_err(io::Error::other)?;
            let j: usize = i + 1;
            match vr {
                ValueRef::Null => Ok(()),
                ValueRef::Integer(v) => o.raw_bind_parameter(j, v).map_err(io::Error::other),
                ValueRef::Real(f) => o.raw_bind_parameter(j, f).map_err(io::Error::other),
                ValueRef::Text(s) => o.raw_bind_parameter(j, s).map_err(io::Error::other),
                ValueRef::Blob(s) => o.raw_bind_parameter(j, s).map_err(io::Error::other),
            }?;
        }
        o.raw_bind_parameter(colcnt + 1, val)
            .map_err(io::Error::other)?;

        let chg: usize = o.raw_execute().map_err(io::Error::other)?;
        tot += chg;
    }
}

pub fn brdr2strings<R>(brdr: R) -> impl Iterator<Item = Result<String, io::Error>>
where
    R: BufRead,
{
    brdr.lines()
}

pub fn stdin2strings() -> impl Iterator<Item = Result<String, io::Error>> {
    brdr2strings(io::stdin().lock())
}

pub fn paste_tx<I, T>(
    i: Connection,
    o: Transaction,
    select: &str,
    insert: &str,
    values: I,
) -> Result<usize, io::Error>
where
    I: Iterator<Item = Result<T, io::Error>>,
    T: ToSql,
{
    let istmt: Statement = i.prepare(select).map_err(io::Error::other)?;
    let mut ostmt: Statement = o.prepare(insert).map_err(io::Error::other)?;
    let tot: usize = paste(istmt, &mut ostmt, values)?;
    ostmt.finalize().map_err(io::Error::other)?;
    o.commit().map_err(io::Error::other)?;
    i.close().map_err(|p| p.1).map_err(io::Error::other)?;
    Ok(tot)
}

pub fn paste_path<P, I, T>(
    i: P,
    o: P,
    select: &str,
    insert: &str,
    values: I,
) -> Result<usize, io::Error>
where
    P: AsRef<Path>,
    I: Iterator<Item = Result<T, io::Error>>,
    T: ToSql,
{
    let iflags: OpenFlags = OpenFlags::SQLITE_OPEN_READ_ONLY;
    let ic: Connection = Connection::open_with_flags(i, iflags).map_err(io::Error::other)?;
    let mut oc: Connection = Connection::open(o).map_err(io::Error::other)?;
    let tx: Transaction = oc.transaction().map_err(io::Error::other)?;
    let tot: usize = paste_tx(ic, tx, select, insert, values)?;
    oc.close().map_err(|p| p.1).map_err(io::Error::other)?;
    Ok(tot)
}

pub const MAX_SQL_SIZE_DEFAULT: u64 = 1048576;

#[derive(Debug)]
pub struct Config {
    pub i_db_file_path: String,
    pub o_db_file_path: String,
    pub select_sqlpath: String,
    pub insert_sqlpath: String,

    pub max_sql_size: u64,
}

pub fn path2sql(max_size: u64, sql_path: String) -> impl FnMut() -> Result<String, io::Error> {
    move || {
        let f: File = File::open(&sql_path).map_err(io::Error::other)?;
        let mut taken = f.take(max_size);
        let mut buf: String = String::default();
        taken.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

impl Config {
    pub fn paste<I, T>(self, values: I) -> Result<usize, io::Error>
    where
        I: Iterator<Item = Result<T, io::Error>>,
        T: ToSql,
    {
        let mut io_get_select_sql = path2sql(self.max_sql_size, self.select_sqlpath);
        let mut io_get_insert_sql = path2sql(self.max_sql_size, self.insert_sqlpath);
        let sel: String = io_get_select_sql()?;
        let ins: String = io_get_insert_sql()?;
        paste_path(self.i_db_file_path, self.o_db_file_path, &sel, &ins, values)
    }
}
