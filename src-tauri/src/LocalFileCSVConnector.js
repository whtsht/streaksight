export function config() {
  return [{ name: 'filePath', type: 'file', default: '', nullable: false }];
}

export async function discovery(config) {
  const escapedPath = config.filePath.replace(/'/g, "''");

  return await streaksight.inferSchemaFromSQL(`
    SELECT * FROM read_csv_auto(
      '${escapedPath}'
    )
    LIMIT 100
  `);
}

export async function sync(name, config, _schema) {
  const escapedPath = config.filePath.replace(/'/g, "''");

  const sql = `
    CREATE TABLE IF NOT EXISTS "${name}" AS
    SELECT * FROM read_csv_auto('${escapedPath}')
  `;

  await streaksight.runSql(sql);
}
