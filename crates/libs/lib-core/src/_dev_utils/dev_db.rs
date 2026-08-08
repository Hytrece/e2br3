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
const DEV_SYSTEM_USER_ID: &str = "00000000-0000-0000-0000-000000000001";

// sql files
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const DB_DIR: &str = "db";

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
	let db_dir = base_dir.join(DB_DIR);

	// -- Create the app_db/app_user with the postgres user.
	{
		let sql_recreate_db_file =
			db_dir.join("admin").join(SQL_RECREATE_DB_FILE_NAME);
		let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
		pexec(&root_db, &sql_recreate_db_file).await?;
		for sql in [
			"ALTER DATABASE \"app_db\" OWNER TO \"app_user\"",
			"DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'e2br3_app_role') THEN CREATE ROLE e2br3_app_role NOLOGIN; END IF; END $$",
			"DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'e2br3_auditor_role') THEN CREATE ROLE e2br3_auditor_role NOLOGIN; END IF; END $$",
			"GRANT e2br3_app_role TO app_user WITH ADMIN OPTION",
			"GRANT e2br3_auditor_role TO app_user WITH ADMIN OPTION",
		] {
			sqlx::query(sql).execute(&root_db).await?;
		}
	}

	// -- SQL Execute each file.
	let app_db = new_db_pool(PG_DEV_APP_URL).await?;

	for group in ["bootstrap", "migrations", "seed"] {
		let mut paths: Vec<PathBuf> = fs::read_dir(db_dir.join(group))?
			.filter_map(|entry| entry.ok().map(|e| e.path()))
			.collect();
		paths.sort();

		for path in paths {
			if path.extension().is_some_and(|ext| ext == "sql") {
				pexec(&app_db, &path).await?;
			}
		}
	}

	// NOTE: Demo user data and passwords are set via SQL seed files in db/seed/.

	Ok(())
}

async fn pexec(db: &Db, file: &Path) -> Result<(), sqlx::Error> {
	info!("{:<12} - pexec: {file:?}", "FOR-DEV-ONLY");

	// -- Read the file.
	let content = fs::read_to_string(file)?;
	let content = render_dev_sql(&content);

	// Split statements while respecting $$ and quoted strings.
	let sqls = split_sql(&content);
	let is_admin_sql = file.file_name().and_then(|name| name.to_str())
		== Some(SQL_RECREATE_DB_FILE_NAME);
	let mut connection = db.acquire().await?;
	if !is_admin_sql {
		sqlx::query(
			"SELECT set_config('app.current_user_id', $1, false),
			        set_config('app.current_organization_id', $2, false),
			        set_config('app.platform_isolation_bypass', 'true', false)",
		)
		.bind(DEV_SYSTEM_USER_ID)
		.bind("00000000-0000-0000-0000-000000000000")
		.execute(&mut *connection)
		.await?;
	}

	for sql in sqls {
		if let Err(e) = sqlx::query(&sql).execute(&mut *connection).await {
			if should_ignore_role_error(&sql, &e) {
				println!(
					"pexec warning: skipping role creation due to permission error:\n{sql}\nreason:\n{e}"
				);
				continue;
			}

			if should_ignore_policy_role_error(&sql, &e) {
				println!(
					"pexec warning: skipping policy creation due to missing role:\n{sql}\nreason:\n{e}"
				);
				continue;
			}

			if should_ignore_grant_role_error(&sql, &e) {
				println!(
					"pexec warning: skipping grant due to missing role:\n{sql}\nreason:\n{e}"
				);
				continue;
			}

			println!("pexec error while running:\n{sql}");
			println!("cause:\n{e}");
			return Err(e);
		}
	}

	Ok(())
}

fn render_dev_sql(sql: &str) -> String {
	sql.lines()
		.filter(|line| !line.trim_start().starts_with('\\'))
		.collect::<Vec<_>>()
		.join("\n")
		.replace(":'app_db_user'", "'app_user'")
		.replace(":\"app_db_user\"", "\"app_user\"")
		.replace(":'app_db_name'", "'app_db'")
		.replace(":\"app_db_name\"", "\"app_db\"")
		.replace(":'app_user_password'", "'dev_only_pwd'")
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
	PgPoolOptions::new()
		.max_connections(2)
		.acquire_timeout(Duration::from_secs(5))
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

fn should_ignore_role_error(sql: &str, err: &sqlx::Error) -> bool {
	let has_create_role = sql.to_ascii_lowercase().contains("create role");
	if !has_create_role {
		return false;
	}

	match err {
		sqlx::Error::Database(db_err) => {
			matches!(db_err.code().as_deref(), Some("42501"))
		}
		_ => false,
	}
}

fn should_ignore_policy_role_error(sql: &str, err: &sqlx::Error) -> bool {
	let has_create_policy = sql.to_ascii_lowercase().contains("create policy");
	if !has_create_policy {
		return false;
	}

	match err {
		sqlx::Error::Database(db_err) => {
			matches!(db_err.code().as_deref(), Some("42704"))
		}
		_ => false,
	}
}

fn should_ignore_grant_role_error(sql: &str, err: &sqlx::Error) -> bool {
	let has_grant = sql.to_ascii_lowercase().contains("grant ");
	if !has_grant {
		return false;
	}

	match err {
		sqlx::Error::Database(db_err) => {
			matches!(db_err.code().as_deref(), Some("42704"))
		}
		_ => false,
	}
}

#[cfg(test)]
mod tests {
	use super::render_dev_sql;

	#[test]
	fn renders_psql_variables_for_raw_sqlx_execution() {
		let rendered = render_dev_sql(
			"\\if :{?app_db_user}\n\\
			\\else\n\\
			\\echo 'app_db_user psql variable is required'\n\\
			\\quit 1\n\\
			\\endif\n\\
			GRANT e2br3_app_role TO :\"app_db_user\";",
		);

		assert_eq!(rendered, "\t\t\tGRANT e2br3_app_role TO \"app_user\";");
	}
}
