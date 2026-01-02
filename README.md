# ⚡streaksight

A local-first, all-in-one data analytics platform.

## Overview

streaksight is a data analytics application designed to handle data connections, query building, and visualization entirely in a local environment.
### Goal: Minimize the time required to turn data into insights.

> [!NOTE]
> Highly experimental and unstable. Use at your own risk.

## Design principles

streaksight focuses on reducing the following three types of time:

- **Setup**  
  Distributed as a single binary with no external dependencies. Installing the application provides all the tools needed for data analysis.

- **Query execution**  
  Built on DuckDB and optimized for local analytics. It can efficiently process datasets ranging from millions to tens of millions of rows.

- **Data exploration**  
  A custom-built query builder enables interactive and intuitive query construction.

## Roadmap to Version 1

### Connectors

- [ ] Implement a small set of representative connectors from each [connector category](https://github.com/whtsht/streaksight/blob/main/CONNECTOR.md) to validate the overall design
  - [ ] LocalFile
  - [ ] S3
  - [ ] MySQL
  - [ ] Github
- [ ] Extend schemas into a unified catalog `catalog = Map<string, schema>`
- [ ] Support incremental imports to efficiently sync data changes

### Query Builder

- [ ] Enable data analysis across multiple data sources using JOIN operations
- [ ] Support more advanced aggregation and analytical operations (HAVING, WINDOW)
- [ ] Allow query results to be reused as nodes, similar to Redash’s query results
- [ ] Provide visibility into node structures and the generated SQL, primarily for debugging and transparency

### Visualization

- [ ] Allow users to create dashboards that combine multiple charts

### Other

- [ ] Save and load projects and configurations
- [ ] Export query results in CSV and JSON formats
- [ ] Provide a built-in help system so users can easily learn how to use the application when needed

## Beyond Version 1 (Draft)

- Allow users to add custom connectors and visualization plugins
- Integrate with cloud services for data sharing and scheduled executions
- Reverse ETL

## Technologies and libraries

### Backend

[![DuckDB](https://img.shields.io/badge/DuckDB-FFF000?style=for-the-badge&logo=duckdb&logoColor=000000)](https://duckdb.org/)
[![Tauri](https://img.shields.io/badge/Tauri-24C8D8?style=for-the-badge&logo=tauri&logoColor=ffffff)](https://tauri.app/)
[![Deno](https://img.shields.io/badge/Deno-000000?style=for-the-badge&logo=deno&logoColor=ffffff)](https://deno.com/)

### Frontend

[![Svelte](https://img.shields.io/badge/Svelte-FF3E00?style=for-the-badge&logo=svelte&logoColor=ffffff)](https://svelte.dev/)
[![shadcn-svelte](https://img.shields.io/badge/shadcn--svelte-000000?style=for-the-badge&logo=tailwindcss&logoColor=38BDF8)](https://shadcn-svelte.com/)
[![Apache ECharts](https://img.shields.io/badge/Apache%20ECharts-AA344D?style=for-the-badge&logo=apacheecharts&logoColor=ffffff)](https://echarts.apache.org/)
[![SvelteFlow](https://img.shields.io/badge/SvelteFlow-FF3E00?style=for-the-badge&logo=svelte&logoColor=ffffff)](https://svelteflow.dev/)

### ETL

- The design is inspired by [Singer](https://www.singer.io/)

## Development environment

### Requirements

- pnpm
- tauri
- DuckDB library
- DuckDB CLI (optional)

### Run the application

```bash
pnpm tauri dev
```
