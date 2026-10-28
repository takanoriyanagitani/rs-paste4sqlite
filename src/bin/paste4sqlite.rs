use std::io;
use std::process::ExitCode;

use rs_paste4sqlite::Config;
use rs_paste4sqlite::MAX_SQL_SIZE_DEFAULT;

use rs_paste4sqlite::stdin2strings;

fn io_envkey2val(key: &'static str) -> impl Fn() -> Result<String, io::Error> {
    move || std::env::var(key).or_else(|_| Ok("".into()))
}

fn io_i_db_file_path() -> impl Fn() -> Result<String, io::Error> {
    io_envkey2val("ENV_I_SQL_FILE_PATH")
}

fn io_o_db_file_path() -> impl Fn() -> Result<String, io::Error> {
    io_envkey2val("ENV_O_SQL_FILE_PATH")
}

fn io_sel_sql_path() -> impl Fn() -> Result<String, io::Error> {
    io_envkey2val("ENV_SEL_SQL_PATH")
}

fn io_ins_sql_path() -> impl Fn() -> Result<String, io::Error> {
    io_envkey2val("ENV_INS_SQL_PATH")
}

fn string2u64(s: String) -> Result<u64, io::Error> {
    str::parse(&s).map_err(io::Error::other)
}

fn bind<F, G, T, U, E>(mut iofn: F, pure: G) -> impl FnMut() -> Result<U, E>
where
    F: FnMut() -> Result<T, E>,
    G: Fn(T) -> Result<U, E>,
{
    move || {
        let t: T = iofn()?;
        pure(t)
    }
}

fn io_max_sql_size_str() -> impl Fn() -> Result<String, io::Error> {
    io_envkey2val("ENV_MAX_SQL_SIZE")
}

fn io_max_sql_size() -> impl FnMut() -> Result<u64, io::Error> {
    bind(io_max_sql_size_str(), string2u64)
}

fn io_config() -> impl FnMut() -> Config {
    || {
        let i_db_file_path: String = io_i_db_file_path()().unwrap_or_default();
        let o_db_file_path: String = io_o_db_file_path()().unwrap_or_default();
        let select_sqlpath: String = io_sel_sql_path()().unwrap_or_default();
        let insert_sqlpath: String = io_ins_sql_path()().unwrap_or_default();

        let max_sql_size: u64 = io_max_sql_size()().unwrap_or(MAX_SQL_SIZE_DEFAULT);
        Config {
            i_db_file_path,
            o_db_file_path,
            select_sqlpath,
            insert_sqlpath,
            max_sql_size,
        }
    }
}

fn io_changed() -> impl FnMut() -> Result<usize, io::Error> {
    || {
        let values = stdin2strings();
        let cfg: Config = io_config()();
        cfg.paste(values)
    }
}

fn io_print_changed() -> impl FnMut(usize) -> Result<(), io::Error> {
    |chg: usize| {
        println!("changed: {chg}");
        Ok(())
    }
}

fn io_main() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        let chg: usize = io_changed()()?;
        io_print_changed()(chg)
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");

        let cfg: Config = io_config()();

        let Config {
            i_db_file_path,
            o_db_file_path,
            select_sqlpath,
            insert_sqlpath,
            max_sql_size,
        } = cfg;

        eprintln!("ENV_I_SQL_FILE_PATH: {i_db_file_path}");
        eprintln!("ENV_O_SQL_FILE_PATH: {o_db_file_path}");
        eprintln!("ENV_SEL_SQL_PATH: {select_sqlpath}");
        eprintln!("ENV_INS_SQL_PATH: {insert_sqlpath}");
        eprintln!("ENV_MAX_SQL_SIZE: {max_sql_size}");

        ExitCode::FAILURE
    })
}
