const { core } = Deno;

function mapDuckDBType(duckdbType) {
  const typeUpper = duckdbType.toUpperCase();

  if (typeUpper.includes('VARCHAR') || typeUpper.includes('TEXT')) {
    return 'string';
  }
  if (
    typeUpper.includes('DOUBLE') ||
    typeUpper.includes('INTEGER') ||
    typeUpper.includes('BIGINT') ||
    typeUpper.includes('DECIMAL') ||
    typeUpper.includes('FLOAT') ||
    typeUpper.includes('TINYINT') ||
    typeUpper.includes('SMALLINT')
  ) {
    return 'number';
  }
  if (typeUpper.includes('BOOLEAN')) {
    return 'boolean';
  }
  if (typeUpper.includes('DATE') || typeUpper.includes('TIMESTAMP')) {
    return 'date';
  }

  return 'string';
}

async function inferSchemaFromSQL(sql) {
  const result = await core.ops.op_run_sql(`
    SELECT column_name, column_type
    FROM (DESCRIBE (${sql}))
  `);

  return {
    columns: result.map((row) => ({
      name: row.column_name,
      type: mapDuckDBType(row.column_type)
    }))
  };
}

globalThis.streaksight = {
  async readFile(path) {
    return await core.ops.op_read_file(path);
  },
  async writeFile(path, contents) {
    return await core.ops.op_write_file(path, contents);
  },
  async runSql(sql) {
    return await core.ops.op_run_sql(sql);
  },
  inferSchemaFromSQL
};
