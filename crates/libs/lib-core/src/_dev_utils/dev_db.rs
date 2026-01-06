use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

type Db = Pool<Postgres>;

// NOTE: Hardcode to prevent deployed system db update.
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

// sql files
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
	info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

	// -- Get the sql_dir
	// Note: This is because cargo test and cargo run won't give the same
	//       current_dir given the worspace layout.
	let current_dir = std::env::current_dir().unwrap();
	let v: Vec<_> = current_dir.components().collect();
	let path_comp = v.get(v.len().wrapping_sub(3));
	let base_dir = if Some(true) == path_comp.map(|c| c.as_os_str() == "crates") {
		v[..v.len() - 3].iter().collect::<PathBuf>()
	} else {
		current_dir.clone()
	};
	let sql_dir = base_dir.join(SQL_DIR);

	// -- Create the app_db/app_user with the postgres user.
	{
		let sql_recreate_db_file = sql_dir.join(SQL_RECREATE_DB_FILE_NAME);
		let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
		pexec(&root_db, &sql_recreate_db_file).await?;
	}

	// -- Get sql files.
	let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir)?
		.filter_map(|entry| entry.ok().map(|e| e.path()))
		.collect();
	paths.sort();

	// -- SQL Execute each file.
	let app_db = new_db_pool(PG_DEV_APP_URL).await?;

	for path in paths {
		let path_str = path.to_string_lossy();

		if path_str.ends_with(".sql")
			&& !path_str.ends_with(SQL_RECREATE_DB_FILE_NAME)
		{
			pexec(&app_db, &path).await?;
		}
	}

	// NOTE: Demo user data and passwords are set via SQL seed files (13-e2br3-seed.sql)

	Ok(())
}

async fn pexec(db: &Db, file: &Path) -> Result<(), sqlx::Error> {
	info!("{:<12} - pexec: {file:?}", "FOR-DEV-ONLY");

	// -- Read the file.
	let content = fs::read_to_string(file)?;

	// Split statements while respecting $$ and quoted strings.
	let sqls = split_sql(&content);

	for sql in sqls {
		sqlx::query(&sql).execute(db).await.map_err(|e| {
			println!("pexec error while running:\n{sql}");
			println!("cause:\n{e}");
			e
		})?;
	}

	Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
	PgPoolOptions::new()
		.max_connections(1)
		.acquire_timeout(Duration::from_millis(500))
		.connect(db_con_url)
		.await
}

fn split_sql(content: &str) -> Vec<String> {
	let mut statements = Vec::new();
	let mut buf = String::new();
	let mut in_dollar = false;
	let mut in_single = false;
	let mut in_line_comment = false;
	let mut in_block_comment = false;
	let mut chars = content.chars().peekable();

	while let Some(c) = chars.next() {
		let next = chars.peek().copied();

		if !in_dollar
			&& !in_single
			&& !in_block_comment
			&& c == '-'
			&& next == Some('-')
		{
			in_line_comment = true;
			buf.push(c);
			buf.push(chars.next().unwrap());
			continue;
		}

		if in_line_comment {
			if c == '\n' {
				in_line_comment = false;
			}
			buf.push(c);
			continue;
		}

		if !in_dollar
			&& !in_single
			&& !in_line_comment
			&& c == '/'
			&& next == Some('*')
		{
			in_block_comment = true;
			buf.push(c);
			buf.push(chars.next().unwrap());
			continue;
		}

		if in_block_comment {
			if c == '*' && next == Some('/') {
				in_block_comment = false;
				buf.push(c);
				buf.push(chars.next().unwrap());
				continue;
			}
			buf.push(c);
			continue;
		}

		if !in_dollar && c == '\'' {
			if chars.peek() == Some(&'\'') {
				// Escaped quote inside a string.
				buf.push(c);
				buf.push(chars.next().unwrap());
				continue;
			}
			in_single = !in_single;
			buf.push(c);
			continue;
		}

		if !in_single && c == '$' && chars.peek() == Some(&'$') {
			in_dollar = !in_dollar;
			buf.push(c);
			buf.push(chars.next().unwrap());
			continue;
		}

		if !in_dollar && !in_single && c == ';' {
			let stmt = buf.trim();
			if !stmt.is_empty() {
				statements.push(stmt.to_string());
			}
			buf.clear();
			continue;
		}

		buf.push(c);
	}

	if !buf.trim().is_empty() {
		statements.push(buf.trim().to_string());
	}

	statements
}
