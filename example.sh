#!/bin/bash

wsm="./paste4sqlite.wasm"

hdir="$PWD"
gdir="/guest.d"
iname="inp.sqlite.db3"
oname="out.sqlite.db3"

hpath="${hdir}/${iname}"
ppath="${hdir}/${oname}"

ipath="${gdir}/${iname}"
opath="${gdir}/${oname}"

select="${gdir}/select.sql"
insert="${gdir}/insert.sql"

max_sql_size=65536

run_wasmtime(){
  cat /dev/stdin |
    wasmtime \
      run \
      --env ENV_I_SQL_FILE_PATH="${ipath}" \
      --env ENV_O_SQL_FILE_PATH="${opath}" \
      --env ENV_SEL_SQL_PATH="${select}" \
      --env ENV_INS_SQL_PATH="${insert}" \
      --env ENV_MAX_SQL_SIZE=${max_sql_size} \
      --dir "${hdir}::${gdir}" \
      "${wsm}"
}

run_wasi(){
  run_wasmtime
}

input(){
  echo helo wrld
  echo hh
}

geninput(){
  echo generating input sqlite file...

  echo "CREATE TABLE IF NOT EXISTS tab1(val TEXT NOT NULL)" | sqlite3 "${hpath}"
  echo "INSERT INTO tab1 VALUES('hello')" | sqlite3 "${hpath}"
  echo "INSERT INTO tab1 VALUES('world')" | sqlite3 "${hpath}"
}

prepareoutput(){
  echo generating empty output sqlite file...

  echo "CREATE TABLE IF NOT EXISTS tab2(val TEXT NOT NULL, ext TEXT NOT NULL)" |
    sqlite3 "${ppath}"
}

test -f "${hpath}" || geninput
test -f "${ppath}" || prepareoutput

echo original sqlite
echo "SELECT rowid,* FROM tab1" | sqlite3 -markdown "${hpath}"
echo

echo truncating...
echo "DELETE FROM tab2" | sqlite3 "${ppath}"

echo pasting...
input | run_wasi
echo

echo pasted sqlite
echo "SELECT rowid,* FROM tab2" | sqlite3 -markdown "${ppath}"
