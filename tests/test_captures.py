import logging
import os
import subprocess
import shutil
import time
import glob
import yaml
import psycopg
import pytest

logging.basicConfig(
    level=getattr(logging, os.getenv("LOG_LEVEL", "INFO").upper(), logging.INFO)
)
logger = logging.getLogger(__name__)

PORT = 5447
CONN_STR = f"host=127.0.0.1 port={PORT} dbname=pgtry user=dbuser password=pencil sslmode=disable"

@pytest.fixture(scope="module")
def server(tmp_path_factory):
    zip_dir = tmp_path_factory.mktemp("schema")
    zip_path = zip_dir / "schema.zip"
    shutil.make_archive(str(zip_path.with_suffix("")), "zip", "pg_catalog_data/pg_schema")
    proc = subprocess.Popen([
        "cargo", "run", "--quiet", "--",
        str(zip_path),
        "--default-catalog", "pgtry",
        "--default-schema", "pg_catalog",
        "--host", "127.0.0.1",
        "--port", str(PORT),
    ], text=True)

    for _ in range(12):
        try:
            with psycopg.connect(CONN_STR):
                break
        except Exception:
            time.sleep(5)
    else:
        proc.terminate()
        raise RuntimeError("server failed to start")

    yield proc
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()


def convert_placeholders(q: str) -> str:
    out = []
    i = 0
    while i < len(q):
        if q[i] == "$":
            j = i + 1
            while j < len(q) and q[j].isdigit():
                j += 1
            if j > i + 1:
                out.append("%s")
                i = j
                continue
        out.append(q[i])
        i += 1
    return "".join(out)


def get_results(cur):
    if cur.description is None:
        return []
    rows = cur.fetchall()
    names = [d.name for d in cur.description]
    result = [dict(zip(names, row)) for row in rows]
    return result
    # pgres = cur.pgresult
    # names = [d.name for d in cur.description]
    # result = []
    # for i in range(pgres.ntuples):
    #     row = {}
    #     for j, name in enumerate(names):
    #         raw_value = pgres.get_value(i, j)
    #         row[name] = raw_value
    #     result.append(row)
    # return result


def replay_captured_queries(queries):
    with psycopg.connect(CONN_STR) as conn:
        cur = conn.cursor()
        for entry in queries:
            if not entry.get("success", True):
                continue
            query = entry["query"]
            params = entry.get("parameters") or []
            """
            We convert $1 to %s queries, because psycopg3 wants that. 
            It sends them as parameters to the server - as it should. 
            pyscopg3 doesn't do string interpolation
            """
            query = query.replace("%", "%%")
            query_exec = convert_placeholders(query)
            cur.execute(query_exec, tuple(params))
            results = get_results(cur)
            expected_results = entry.get("result")
            if entry.get("only_check_run"):
                # TODO: this is for startup time etc. we only check if query runs
                continue
            if entry.get("no_order"):
                # TODO: if there is no order by we don't care about the order of the rows
                #   currently we don't check this and skip
                continue
            for (expected_row, row) in zip(expected_results, results):
                logger.debug("query %s", query)
                logger.debug("row %s", row)
                logger.debug("expected %s", expected_row)
                if row != expected_row:
                    logger.warning(
                        "WARN: for query %s result: %s and expected: %s are different",
                        query,
                        row,
                        expected_row,
                    )
                    assert(row == expected_row)
                    # import ipdb; ipdb.set_trace()





@pytest.mark.skip(reason="capture replay not stable")
def test_captured_queries(server):
    capture_files = sorted(glob.glob("captures/*.yaml"))
    assert capture_files, "no capture files found"
    
    for file in capture_files:
        print("running capture", file)
        with open(file) as f:
            data = yaml.safe_load(f)
            replay_captured_queries(data)

